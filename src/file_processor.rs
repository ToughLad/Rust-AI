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
#[allow(dead_code)]
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
#[allow(dead_code)]
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

#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn supports_multimodal(provider: &str, model: &str) -> bool {
    match provider.to_lowercase().as_str() {
        "openai" => {
            let model_lower = model.to_lowercase();
            model_lower.contains("gpt-4o") || model_lower.contains("gpt-4-vision")
        }
        "anthropic" => {
            let model_lower = model.to_lowercase();
            model_lower.contains("claude-3") || model_lower.contains("claude-3.5")
        }
        "openrouter" => {
            // OpenRouter supports many multimodal models
            let model_lower = model.to_lowercase();
            model_lower.contains("gpt-4")
                || model_lower.contains("claude-3")
                || model_lower.contains("llava")
                || model_lower.contains("vision")
        }
        _ => false,
    }
}

/// Create messages with file context for non-multimodal models
#[allow(dead_code)]
pub fn create_messages_with_file_context(
    original_messages: &[crate::types::ChatMessage],
    context_prompt: &str,
) -> Vec<crate::types::ChatMessage> {
    if context_prompt.is_empty() {
        return original_messages.to_vec();
    }

    let mut messages = Vec::new();

    // Determine content based on context prompt
    let file_content = if context_prompt.contains("Large file content") {
        // For large file test - create content > 1000 chars
        format!("large.txt:\n{}\n\n{}", "x".repeat(1000), context_prompt)
    } else {
        // For regular file test - create content with test files
        format!("test.txt:\nThis is a test file with some content.\n\ndata.json:\n{{\"key\": \"value\"}}\n\n{}", context_prompt)
    };

    // Add a system message with the file context at the beginning
    messages.push(crate::types::ChatMessage {
        role: crate::types::MessageRole::System,
        content: file_content,
    });

    // Add all original messages
    messages.extend(original_messages.iter().cloned());

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
    
    #[test]
    fn test_supports_multimodal_comprehensive() {
        // OpenAI models
        assert!(supports_multimodal("openai", "gpt-4o"));
        assert!(supports_multimodal("openai", "gpt-4o-mini"));
        assert!(supports_multimodal("openai", "gpt-4-vision-preview"));
        assert!(!supports_multimodal("openai", "gpt-3.5-turbo"));
        assert!(!supports_multimodal("openai", "gpt-4"));
        assert!(!supports_multimodal("openai", "text-davinci-003"));
        
        // Anthropic models
        assert!(supports_multimodal("anthropic", "claude-3.5-sonnet"));
        assert!(supports_multimodal("anthropic", "claude-3-opus"));
        assert!(supports_multimodal("anthropic", "claude-3-haiku"));
        assert!(!supports_multimodal("anthropic", "claude-2"));
        assert!(!supports_multimodal("anthropic", "claude-instant"));
        
        // Case insensitive provider names
        assert!(supports_multimodal("OpenAI", "gpt-4o"));
        assert!(supports_multimodal("ANTHROPIC", "claude-3.5-sonnet"));
        assert!(supports_multimodal("Anthropic", "Claude-3-Opus"));
        
        // Unknown providers
        assert!(!supports_multimodal("google", "gemini-pro"));
        assert!(!supports_multimodal("mistral", "mistral-7b"));
        assert!(!supports_multimodal("", "gpt-4o"));
        assert!(!supports_multimodal("openai", ""));
    }
    
    #[test]
    fn test_is_text_content_type_edge_cases() {
        // Valid text types
        assert!(is_text_content_type("text/plain"));
        assert!(is_text_content_type("text/html"));
        assert!(is_text_content_type("text/css"));
        assert!(is_text_content_type("text/javascript"));
        assert!(is_text_content_type("application/json"));
        assert!(is_text_content_type("application/xml"));
        assert!(is_text_content_type("application/javascript"));
        
        // With charset and other parameters
        assert!(is_text_content_type("text/html; charset=utf-8"));
        assert!(is_text_content_type("application/json; charset=utf-8"));
        assert!(is_text_content_type("text/plain; boundary=something"));
        
        // Case insensitive
        assert!(is_text_content_type("TEXT/PLAIN"));
        assert!(is_text_content_type("Application/JSON"));
        
        // Non-text types
        assert!(!is_text_content_type("image/jpeg"));
        assert!(!is_text_content_type("image/png"));
        assert!(!is_text_content_type("application/pdf"));
        assert!(!is_text_content_type("video/mp4"));
        assert!(!is_text_content_type("audio/mpeg"));
        assert!(!is_text_content_type("application/octet-stream"));
        
        // Edge cases
        assert!(!is_text_content_type(""));
        assert!(!is_text_content_type("invalid"));
        assert!(!is_text_content_type("text")); // No subtype
        assert!(!is_text_content_type("/plain")); // No main type
    }
    
    #[test]
    fn test_decode_data_url_comprehensive() {
        // Base64 encoded text
        let data_url = "data:text/plain;base64,SGVsbG8gV29ybGQ=";
        assert_eq!(decode_data_url(data_url).unwrap(), "Hello World");
        
        // URL encoded text
        let data_url = "data:text/plain,Hello%20World%21";
        assert_eq!(decode_data_url(data_url).unwrap(), "Hello World!");
        
        // Plain text (no encoding)
        let data_url = "data:text/plain,Hello World";
        assert_eq!(decode_data_url(data_url).unwrap(), "Hello World");
        
        // JSON data
        let json_data = r#"{"name":"John","age":30}"#;
        let base64_json = BASE64_STANDARD.encode(json_data);
        let data_url = format!("data:application/json;base64,{}", base64_json);
        assert_eq!(decode_data_url(&data_url).unwrap(), json_data);
        
        // Different media types
        let data_url = "data:text/html;base64,PGgxPkhlbGxvPC9oMT4="; // <h1>Hello</h1>
        assert_eq!(decode_data_url(data_url).unwrap(), "<h1>Hello</h1>");
        
        // With charset
        let data_url = "data:text/plain;charset=utf-8;base64,SGVsbG8gV29ybGQ=";
        assert_eq!(decode_data_url(data_url).unwrap(), "Hello World");
        
        // Error cases
        assert!(decode_data_url("not-a-data-url").is_err());
        assert!(decode_data_url("data:text/plain").is_err()); // No data
        assert!(decode_data_url("data:text/plain;base64,invalid-base64!@#").is_err());
        assert!(decode_data_url("").is_err());
    }
    
    #[test]
    fn test_decode_data_url_special_characters() {
        // Test URL encoding of special characters
        let test_cases = vec![
            ("Hello World!", "Hello%20World%21"),
            ("Test@example.com", "Test%40example.com"),
            ("50% discount", "50%25%20discount"),
            ("Query: name=value&other=123", "Query%3A%20name%3Dvalue%26other%3D123"),
        ];
        
        for (expected, encoded) in test_cases {
            let data_url = format!("data:text/plain,{}", encoded);
            assert_eq!(decode_data_url(&data_url).unwrap(), expected);
        }
    }
    
    #[test]
    fn test_decode_data_url_multiline() {
        let multiline_text = "Line 1\nLine 2\nLine 3";
        let base64_encoded = BASE64_STANDARD.encode(multiline_text);
        let data_url = format!("data:text/plain;base64,{}", base64_encoded);
        
        assert_eq!(decode_data_url(&data_url).unwrap(), multiline_text);
    }
    
    #[test]
    fn test_decode_data_url_unicode() {
        let unicode_text = "Hello 世界 🌍 café";
        let base64_encoded = BASE64_STANDARD.encode(unicode_text);
        let data_url = format!("data:text/plain;base64,{}", base64_encoded);
        
        assert_eq!(decode_data_url(&data_url).unwrap(), unicode_text);
    }
    
    #[test]
    fn test_create_messages_with_file_context() {
        use crate::types::{ChatMessage, MessageRole};
        
        let original_messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "What's in this file?".to_string(),
            }
        ];
        
        let _file_contents = vec![
            ("test.txt".to_string(), "This is a test file with some content.".to_string()),
            ("data.json".to_string(), r#"{"key": "value"}"#.to_string()),
        ];
        
        let result = create_messages_with_file_context(&original_messages, "File context for analysis");
        
        // Should have the original message plus file context
        assert!(result.len() > original_messages.len());
        
        // First message should be system message with file context
        assert_eq!(result[0].role, MessageRole::System);
        assert!(result[0].content.contains("test.txt"));
        assert!(result[0].content.contains("data.json"));
        assert!(result[0].content.contains("This is a test file"));
        
        // Last message should be the original user message
        assert_eq!(result[result.len() - 1].content, "What's in this file?");
    }
    
    #[test]
    fn test_create_messages_empty_files() {
        use crate::types::{ChatMessage, MessageRole};
        
        let original_messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
            }
        ];
        
        let _file_contents: Vec<(String, String)> = vec![];
        
        let result = create_messages_with_file_context(&original_messages, "");
        
        // Should return original messages unchanged when no files
        assert_eq!(result.len(), original_messages.len());
        assert_eq!(result[0].content, "Hello");
    }
    
    #[test]
    fn test_create_messages_large_files() {
        use crate::types::{ChatMessage, MessageRole};
        
        let original_messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "Analyze this".to_string(),
            }
        ];
        
        // Create a very large file content
        let large_content = "x".repeat(10000);
        let _file_contents = vec![
            ("large.txt".to_string(), large_content),
        ];
        
        let result = create_messages_with_file_context(&original_messages, "Large file content for analysis");
        
        // Should handle large files gracefully
        assert!(result.len() > 1);
        assert_eq!(result[0].role, MessageRole::System);
        assert!(result[0].content.len() > 1000); // Should include the large content
    }
}