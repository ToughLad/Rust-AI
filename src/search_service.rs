use anyhow::{anyhow, Result};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
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
#[allow(dead_code)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct BraveRequest {
    q: String,
    count: u8,
    search_lang: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BraveResponse {
    web: Option<BraveWeb>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BraveWeb {
    results: Vec<BraveResult>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BraveResult {
    title: String,
    url: String,
    description: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SearxngResult {
    title: String,
    url: String,
    content: Option<String>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct SearchService {
    config: Config,
    client: Client,
    cache: SearchCache,
}

#[allow(dead_code)]
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
            Regex::new(r"\b(who|what|when|where|which).*(win|won|winning|winner|elected|announced|released|launched)\b").unwrap(),
            Regex::new(r"\b(who\s+is|who\s+won|who\s+will|what\s+is\s+happening)\b").unwrap(),
        ];

        patterns.iter().any(|pattern| pattern.is_match(&lower_query))
    }

    /// Perform web search using available providers
    pub async fn perform_web_search(&self, query: &str) -> Result<SearchResponse> {
        // If search is not enabled, return disabled response
        if !self.config.search.enabled {
            return Ok(SearchResponse {
                query: query.to_string(),
                results: Vec::new(),
                provider: "disabled".to_string(),
                took_ms: 0,
            });
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, SearchConfig, TavilyConfig, BraveConfig, SearxngConfig};
    
    fn create_test_config(enabled: bool) -> Config {
        let mut config = Config::from_env();
        config.search = SearchConfig {
            enabled,
            cache_duration: 300, // 5 minutes
            tavily: TavilyConfig {
                api_key: "test_tavily_key".to_string(),
                base_url: "https://api.tavily.com".to_string(),
            },
            brave: BraveConfig {
                api_key: "test_brave_key".to_string(),
                base_url: "https://api.search.brave.com".to_string(),
            },
            searxng: SearxngConfig {
                base_url: "http://localhost:8090".to_string(),
                enabled: true,
            },
        };
        config
    }
    
    #[test]
    fn test_search_service_new() {
        let config = create_test_config(true);
        let service = SearchService::new(config.clone());
        
        assert_eq!(service.config.search.enabled, true);
        assert_eq!(service.config.search.cache_duration, 300);
        assert_eq!(service.config.search.tavily.api_key, "test_tavily_key");
    }
    
    #[test]
    fn test_needs_internet_search() {
        let config = create_test_config(true);
        let service = SearchService::new(config);
        
        // Should NOT need search for basic queries
        assert!(!service.needs_internet_search("hello"));
        assert!(!service.needs_internet_search("write a poem"));
        assert!(!service.needs_internet_search("explain recursion"));
        assert!(!service.needs_internet_search("how to sort an array"));
        
        // Should need search for current/recent information
        assert!(service.needs_internet_search("what's the weather today"));
        assert!(service.needs_internet_search("latest news"));
        assert!(service.needs_internet_search("current stock price"));
        assert!(service.needs_internet_search("today's date"));
        assert!(service.needs_internet_search("recent developments"));
        
        // Should need search for specific factual queries
        assert!(service.needs_internet_search("population of Tokyo 2024"));
        assert!(service.needs_internet_search("who won the election"));
        assert!(service.needs_internet_search("latest iPhone release"));
        
        // Edge cases
        assert!(!service.needs_internet_search(""));
        assert!(!service.needs_internet_search("   "));
    }
    
    #[test]
    fn test_needs_internet_search_keywords() {
        let config = create_test_config(true);
        let service = SearchService::new(config);
        
        // Test specific keywords that should trigger search
        let search_keywords = [
            "today", "current", "latest", "recent", "now", "2024", "2025",
            "news", "weather", "stock", "price", "who is", "what is happening"
        ];
        
        for keyword in search_keywords {
            assert!(
                service.needs_internet_search(keyword),
                "Keyword '{}' should trigger internet search", keyword
            );
        }
        
        // Test keywords that should NOT trigger search
        let no_search_keywords = [
            "explain", "how to", "write", "create", "help", "define",
            "algorithm", "programming", "code", "function"
        ];
        
        for keyword in no_search_keywords {
            assert!(
                !service.needs_internet_search(keyword),
                "Keyword '{}' should NOT trigger internet search", keyword
            );
        }
    }
    
    #[test]
    fn test_needs_internet_search_case_insensitive() {
        let config = create_test_config(true);
        let service = SearchService::new(config);
        
        // Should be case insensitive
        assert!(service.needs_internet_search("TODAY"));
        assert!(service.needs_internet_search("Today"));
        assert!(service.needs_internet_search("tOdAy"));
        assert!(service.needs_internet_search("LATEST NEWS"));
        assert!(service.needs_internet_search("Latest News"));
    }
    
    #[tokio::test]
    async fn test_perform_web_search_disabled() {
        let config = create_test_config(false);
        let service = SearchService::new(config);
        
        let result = service.perform_web_search("test query").await;
        assert!(result.is_ok());
        
        let search_response = result.unwrap();
        assert_eq!(search_response.query, "test query");
        assert!(search_response.results.is_empty());
        assert_eq!(search_response.provider, "disabled");
    }
    
    #[test]
    fn test_cleanup_cache() {
        let config = create_test_config(true);
        let service = SearchService::new(config);
        
        // Should not panic when called
        service.cleanup_cache();
        
        // Test multiple calls
        service.cleanup_cache();
        service.cleanup_cache();
    }
    
    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            title: "Test Title".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Test snippet here".to_string(),
            score: Some(0.85),
        };
        
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SearchResult = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.title, result.title);
        assert_eq!(deserialized.url, result.url);
        assert_eq!(deserialized.snippet, result.snippet);
        assert_eq!(deserialized.score, result.score);
    }
    
    #[test]
    fn test_search_response_serialization() {
        let response = SearchResponse {
            query: "test query".to_string(),
            results: vec![
                SearchResult {
                    title: "Result 1".to_string(),
                    url: "https://example1.com".to_string(),
                    snippet: "Snippet 1".to_string(),
                    score: Some(0.9),
                },
                SearchResult {
                    title: "Result 2".to_string(),
                    url: "https://example2.com".to_string(),
                    snippet: "Snippet 2".to_string(),
                    score: Some(0.8),
                },
            ],
            provider: "tavily".to_string(),
            took_ms: 200,
        };
        
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: SearchResponse = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.query, response.query);
        assert_eq!(deserialized.results.len(), 2);
        assert_eq!(deserialized.provider, response.provider);
        assert_eq!(deserialized.took_ms, response.took_ms);
        assert_eq!(deserialized.results[0].title, "Result 1");
        assert_eq!(deserialized.results[1].url, "https://example2.com");
    }
    
    #[test]
    fn test_tavily_request_serialization() {
        let request = TavilyRequest {
            api_key: "test_key_123".to_string(),
            query: "test search query".to_string(),
            search_depth: "basic".to_string(),
            include_answer: true,
            include_raw_content: false,
            max_results: 5,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: TavilyRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.api_key, request.api_key);
        assert_eq!(deserialized.query, request.query);
        assert_eq!(deserialized.max_results, request.max_results);
    }
    
    #[test]
    fn test_brave_request_serialization() {
        let request = BraveRequest {
            q: "test query".to_string(),
            count: 10,
            search_lang: "en".to_string(),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: BraveRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.q, request.q);
        assert_eq!(deserialized.count, request.count);
        assert_eq!(deserialized.search_lang, request.search_lang);
    }
    
    #[test]
    fn test_search_patterns() {
        let config = create_test_config(true);
        let service = SearchService::new(config);
        
        // Test various query patterns that should trigger search
        let search_queries = vec![
            "What's happening in Ukraine today?",
            "Latest Apple stock price",
            "Current weather in New York",
            "Who won the Super Bowl 2024?",
            "Recent COVID-19 updates",
            "Today's exchange rate USD to EUR",
            "Breaking news today",
        ];
        
        for query in search_queries {
            assert!(
                service.needs_internet_search(query),
                "Query '{}' should trigger search", query
            );
        }
        
        // Test queries that should NOT trigger search
        let no_search_queries = vec![
            "Explain machine learning",
            "How to implement binary search",
            "Write a Python function to sort a list",
            "What is recursion?",
            "Help me understand databases",
            "Create a REST API design",
            "Explain the MVC pattern",
        ];
        
        for query in no_search_queries {
            assert!(
                !service.needs_internet_search(query),
                "Query '{}' should NOT trigger search", query
            );
        }
    }
}