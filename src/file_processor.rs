use anyhow::{anyhow, Result};
use base64::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::types::Attachment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedAttachment {
    pub name: String,
    pub content_type: String,
    pub content: String,
    pub is_image: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResult {
    pub processed_attachments: Vec<ProcessedAttachment>,
    pub context_prompt: String,
}

/// Process file attachments for AI model consumption
pub async fn process_file_attachments(
    client: &Client,
    attachments: &[Attachment],
) -> Result<ProcessResult> {
    let mut processed_attachments = Vec::new();
    let mut context_parts = Vec::new();

    for attachment in attachments {
        match process_file_attachment(client, attachment).await {
            Ok(processed) => {
                if processed.is_image {
                    context_parts.push(format!("[Image: {}]", processed.name));
                } else {
                    let content_preview = if processed.content.len() > 2000 {
                        format!("{}... (truncated)", &processed.content[..2000])
                    } else {
                        processed.content.clone()
                    };

                    let context_entry = if !content_preview.is_empty() {
                        format!(
                            "[File: {} ({})]\n{}",
                            processed.name, processed.content_type, content_preview
                        )
                    } else {
                        format!("[File: {} ({})]", processed.name, processed.content_type)
                    };
                    
                    context_parts.push(context_entry);
                }
                processed_attachments.push(processed);
            }
            Err(error) => {
                tracing::error!("Failed to process attachment {}: {}", attachment.name, error);
                context_parts.push(format!("[File: {} - Processing failed]", attachment.name));
            }
        }
    }

    let context_prompt = if !context_parts.is_empty() {
        format!(
            "\n\n--- Attached Files ---\n{}\n--- End of Files ---\n\nPlease analyze the attached files and respond to the user's query.",
            context_parts.join("\n\n")
        )
    } else {
        String::new()
    };

    Ok(ProcessResult {
        processed_attachments,
        context_prompt,
    })
}

/// Process a single file attachment
async fn process_file_attachment(
    client: &Client,
    attachment: &Attachment,
) -> Result<ProcessedAttachment> {
    let is_image = attachment.content_type.starts_with("image/");

    // For images, we'll just pass the URL (models like OpenRouter handle image URLs directly)
    if is_image {
        return Ok(ProcessedAttachment {
            name: attachment.name.clone(),
            content_type: attachment.content_type.clone(),
            content: attachment.url.clone(), // Pass URL for image processing
            is_image: true,
        });
    }

    // For text-based files, try to fetch and read content
    let content = if attachment.url.starts_with("data:") {
        // Decode data URL inline (data:[mime][;base64],payload)
        decode_data_url(&attachment.url)?
    } else if attachment.url.starts_with("http") {
        // Fetch from HTTP URL
        fetch_url_content(client, &attachment.url).await?
    } else {
        // Local file path or unsupported scheme
        return Err(anyhow!("Unsupported URL scheme: {}", attachment.url));
    };

    Ok(ProcessedAttachment {
        name: attachment.name.clone(),
        content_type: attachment.content_type.clone(),
        content,
        is_image: false,
    })
}

fn decode_data_url(data_url: &str) -> Result<String> {
    let comma_idx = data_url
        .find(',')
        .ok_or_else(|| anyhow!("Invalid data URL format"))?;
    
    let meta = &data_url[5..comma_idx]; // strip 'data:'
    let payload = &data_url[comma_idx + 1..];
    
    let is_base64 = meta.contains(";base64");
    
    if is_base64 {
        let decoded_bytes = BASE64_STANDARD
            .decode(payload)
            .map_err(|e| anyhow!("Failed to decode base64: {}", e))?;
        
        String::from_utf8(decoded_bytes)
            .map_err(|e| anyhow!("Failed to convert to UTF-8: {}", e))
    } else {
        // URL decoded content
        urlencoding::decode(payload)
            .map(|s| s.into_owned())
            .map_err(|e| anyhow!("Failed to URL decode: {}", e))
    }
}

async fn fetch_url_content(client: &Client, url: &str) -> Result<String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to fetch URL: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!("HTTP error {}: {}", response.status(), url));
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("");

    if !is_text_content_type(content_type) {
        return Err(anyhow!("Non-text content type: {}", content_type));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

    // Limit file size to prevent memory issues
    if bytes.len() > 10 * 1024 * 1024 {
        // 10MB limit
        return Err(anyhow!("File too large: {} bytes", bytes.len()));
    }

    String::from_utf8(bytes.to_vec())
        .map_err(|e| anyhow!("Failed to convert to UTF-8: {}", e))
}

fn is_text_content_type(content_type: &str) -> bool {
    let lower = content_type.to_lowercase();
    lower.starts_with("text/")
        || lower.contains("application/json")
        || lower.contains("application/javascript")
        || lower.contains("application/xml")
        || lower.contains("application/yaml")
        || lower.contains("application/x-yaml")
}

/// Check if a provider supports multimodal input (images)
pub fn supports_multimodal(provider: &str, model: &str) -> bool {
    match provider.to_lowercase().as_str() {
        "openai" => {
            model.contains("gpt-4") || model.contains("gpt-4o") || model.contains("gpt-4-vision")
        }
        "anthropic" => {
            model.contains("claude-3") || model.contains("claude-3.5")
        }
        "openrouter" => {
            // OpenRouter supports many multimodal models
            model.contains("gpt-4")
                || model.contains("claude-3")
                || model.contains("llava")
                || model.contains("vision")
        }
        _ => false,
    }
}

/// Create messages with file context for non-multimodal models
pub fn create_messages_with_file_context(
    original_messages: &[crate::types::ChatMessage],
    context_prompt: &str,
) -> Vec<crate::types::ChatMessage> {
    if context_prompt.is_empty() {
        return original_messages.to_vec();
    }

    let mut messages = Vec::new();

    // Add all messages except the last user message
    for (i, message) in original_messages.iter().enumerate() {
        if i == original_messages.len() - 1 && matches!(message.role, crate::types::MessageRole::User) {
            // Modify the last user message to include file context
            messages.push(crate::types::ChatMessage {
                role: message.role.clone(),
                content: format!("{}{}", message.content, context_prompt),
            });
        } else {
            messages.push(message.clone());
        }
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_multimodal() {
        assert!(supports_multimodal("openai", "gpt-4o"));
        assert!(supports_multimodal("anthropic", "claude-3.5-sonnet"));
        assert!(!supports_multimodal("openai", "gpt-3.5-turbo"));
        assert!(!supports_multimodal("unknown", "some-model"));
    }

    #[test]
    fn test_is_text_content_type() {
        assert!(is_text_content_type("text/plain"));
        assert!(is_text_content_type("application/json"));
        assert!(is_text_content_type("text/html; charset=utf-8"));
        assert!(!is_text_content_type("image/jpeg"));
        assert!(!is_text_content_type("application/pdf"));
    }

    #[test]
    fn test_decode_data_url() {
        // Test base64 data URL
        let data_url = "data:text/plain;base64,SGVsbG8gV29ybGQ=";
        let result = decode_data_url(data_url).unwrap();
        assert_eq!(result, "Hello World");

        // Test URL encoded data URL
        let data_url = "data:text/plain,Hello%20World";
        let result = decode_data_url(data_url).unwrap();
        assert_eq!(result, "Hello World");
    }
}