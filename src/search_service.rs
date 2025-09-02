use anyhow::{anyhow, Result};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;

use crate::config::Config;
use crate::types::{SearchResult, SearchResponse};

// Simple in-memory cache for search results
type SearchCache = Arc<Mutex<HashMap<String, (SearchResponse, Instant)>>>;

#[derive(Debug, Serialize, Deserialize)]
struct TavilyRequest {
    api_key: String,
    query: String,
    search_depth: String,
    include_answer: bool,
    include_raw_content: bool,
    max_results: u8,
}

#[derive(Debug, Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Debug, Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct BraveRequest {
    q: String,
    count: u8,
    search_lang: String,
}

#[derive(Debug, Deserialize)]
struct BraveResponse {
    web: Option<BraveWeb>,
}

#[derive(Debug, Deserialize)]
struct BraveWeb {
    results: Vec<BraveResult>,
}

#[derive(Debug, Deserialize)]
struct BraveResult {
    title: String,
    url: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct SearxngResult {
    title: String,
    url: String,
    content: Option<String>,
}

#[derive(Clone)]
pub struct SearchService {
    config: Config,
    client: Client,
    cache: SearchCache,
}

impl SearchService {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(3500))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Detect if a query needs internet search
    pub fn needs_internet_search(&self, query: &str) -> bool {
        if !self.config.search.enabled {
            return false;
        }

        let lower_query = query.to_lowercase();

        // Patterns that indicate need for current information
        let patterns = vec![
            Regex::new(r"\b(current|today|now|latest|recent|live|real[\s-]?time)\b").unwrap(),
            Regex::new(r"\b(price|cost|worth|value|rate|stock|market)\b").unwrap(),
            Regex::new(r"\b(weather|temperature|forecast|climate)\b").unwrap(),
            Regex::new(r"\b(news|happening|event|update|announcement)\b").unwrap(),
            Regex::new(r"\b(score|game|match|tournament|competition)\b").unwrap(),
            Regex::new(r"\b20(2[4-9]|[3-9]\d)\b").unwrap(), // Years 2024 and beyond
            Regex::new(r"\b(january|february|march|april|may|june|july|august|september|october|november|december)\s+\d{1,2},?\s*20(2[4-9]|[3-9]\d)\b").unwrap(),
            Regex::new(r"\bwhat\s+(is|are|was|were)\s+the\b").unwrap(),
            Regex::new(r"\bhow\s+(much|many|long|far|old)\s+(is|are|does|do)\b").unwrap(),
            Regex::new(r"\b(who|what|when|where|which)\s+.*\s+(win|won|winning|winner|elected|announced|released|launched)\b").unwrap(),
        ];

        patterns.iter().any(|pattern| pattern.is_match(&lower_query))
    }

    /// Perform web search using available providers
    pub async fn perform_web_search(&self, query: &str) -> Result<SearchResponse> {
        // Check cache first
        let cache_key = format!("search:{}", query);
        if let Ok(cache) = self.cache.lock() {
            if let Some((cached_response, cached_at)) = cache.get(&cache_key) {
                if cached_at.elapsed() < Duration::from_secs(self.config.search.cache_duration) {
                    return Ok(cached_response.clone());
                }
            }
        }

        let mut results = Vec::new();
        let mut provider = "none";

        // Try Tavily first
        if results.is_empty() && !self.config.search.tavily.api_key.is_empty() {
            match self.search_tavily(query).await {
                Ok(tavily_results) if !tavily_results.is_empty() => {
                    results = tavily_results;
                    provider = "tavily";
                }
                Err(_) => {
                    tracing::warn!("Tavily provider failed, trying next...");
                }
                _ => {}
            }
        }

        // Try Brave if Tavily didn't work
        if results.is_empty() && !self.config.search.brave.api_key.is_empty() {
            match self.search_brave(query).await {
                Ok(brave_results) if !brave_results.is_empty() => {
                    results = brave_results;
                    provider = "brave";
                }
                Err(_) => {
                    tracing::warn!("Brave provider failed, trying next...");
                }
                _ => {}
            }
        }

        // Fall back to SearXNG only if API providers failed
        if results.is_empty() && self.config.search.searxng.enabled {
            match self.search_searxng(query).await {
                Ok(searxng_results) if !searxng_results.is_empty() => {
                    results = searxng_results;
                    provider = "searxng";
                }
                Err(_) => {
                    tracing::warn!("SearXNG provider failed");
                }
                _ => {}
            }
        }

        let response = SearchResponse {
            query: query.to_string(),
            results,
            provider: provider.to_string(),
            took_ms: 0, // TODO: Measure actual time
        };

        // Cache the response
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(cache_key, (response.clone(), Instant::now()));
        }

        Ok(response)
    }

    async fn search_tavily(&self, query: &str) -> Result<Vec<SearchResult>> {
        let request = TavilyRequest {
            api_key: self.config.search.tavily.api_key.clone(),
            query: query.to_string(),
            search_depth: "basic".to_string(),
            include_answer: true,
            include_raw_content: false,
            max_results: 5,
        };

        let response = timeout(
            Duration::from_millis(3500),
            self.client
                .post(format!("{}/search", self.config.search.tavily.base_url))
                .header("Content-Type", "application/json")
                .header("api-key", &self.config.search.tavily.api_key)
                .json(&request)
                .send(),
        )
        .await
        .map_err(|_| anyhow!("Tavily request timeout"))?
        .map_err(|e| anyhow!("Tavily request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Tavily API error: {}", response.status()));
        }

        let tavily_response: TavilyResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Tavily response: {}", e))?;

        Ok(tavily_response
            .results
            .into_iter()
            .map(|result| SearchResult {
                title: result.title,
                url: result.url,
                snippet: result.content,
                score: None,
            })
            .collect())
    }

    async fn search_brave(&self, query: &str) -> Result<Vec<SearchResult>> {
        let params = BraveRequest {
            q: query.to_string(),
            count: 5,
            search_lang: "en".to_string(),
        };

        let response = timeout(
            Duration::from_millis(3500),
            self.client
                .get(format!("{}/v1/web/search", self.config.search.brave.base_url))
                .header("X-Subscription-Token", &self.config.search.brave.api_key)
                .query(&params)
                .send(),
        )
        .await
        .map_err(|_| anyhow!("Brave request timeout"))?
        .map_err(|e| anyhow!("Brave request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Brave API error: {}", response.status()));
        }

        let brave_response: BraveResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Brave response: {}", e))?;

        Ok(brave_response
            .web
            .unwrap_or_else(|| BraveWeb { results: vec![] })
            .results
            .into_iter()
            .map(|result| SearchResult {
                title: result.title,
                url: result.url,
                snippet: result.description,
                score: None,
            })
            .collect())
    }

    async fn search_searxng(&self, query: &str) -> Result<Vec<SearchResult>> {
        let mut params = HashMap::new();
        params.insert("q", query);
        params.insert("format", "json");
        params.insert("safesearch", "1");
        params.insert("pageno", "1");

        let response = timeout(
            Duration::from_millis(5000), // Slightly longer timeout for SearXNG
            self.client
                .get(format!("{}/search", self.config.search.searxng.base_url))
                .query(&params)
                .send(),
        )
        .await
        .map_err(|_| anyhow!("SearXNG request timeout"))?
        .map_err(|e| anyhow!("SearXNG request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("SearXNG error: {}", response.status()));
        }

        let searxng_results: Vec<SearxngResult> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse SearXNG response: {}", e))?;

        Ok(searxng_results
            .into_iter()
            .take(5)
            .map(|result| SearchResult {
                title: result.title,
                url: result.url,
                snippet: result.content.unwrap_or_else(|| "No content available".to_string()),
                score: None,
            })
            .collect())
    }

    /// Clear expired entries from cache
    pub fn cleanup_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            let cache_duration = Duration::from_secs(self.config.search.cache_duration);
            cache.retain(|_, (_, cached_at)| cached_at.elapsed() < cache_duration);
        }
    }
}