//! Anthropic Claude provider implementation
//!
//! Anthropic uses a different message format than OpenAI, so we implement
//! LlmProvider directly rather than using the OpenAI-compatible base.

use crate::error::{LlmError, Result};
use crate::llm::provider::{LlmProvider, ProviderConfig, ProviderStats, ProviderType};
use crate::llm::providers::base::HttpProviderClient;
use crate::llm::{EmbeddingResponse, GenerationResponse, Message, Role};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Anthropic message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

impl From<&Message> for AnthropicMessage {
    fn from(msg: &Message) -> Self {
        Self {
            role: match msg.role {
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
                Role::System => "user".to_string(), // System messages handled separately
            },
            content: msg.content.clone(),
        }
    }
}

/// Anthropic API request
#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

/// Anthropic API response
#[derive(Debug, Deserialize)]
pub struct AnthropicResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub usage: UsageInfo,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct UsageInfo {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Anthropic models list response
#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub id: String,
}

/// Anthropic Claude provider
pub struct AnthropicProvider {
    client: HttpProviderClient,
    config: ProviderConfig,
    stats: ProviderStats,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn create(config: ProviderConfig) -> Arc<dyn LlmProvider> {
        let client = HttpProviderClient::new(config.timeout);
        Arc::new(Self {
            client,
            config,
            stats: ProviderStats::default(),
        })
    }

    /// Create from environment variable
    pub fn from_env(
        text_model: String,
        embedding_model: Option<String>,
    ) -> std::result::Result<Arc<dyn LlmProvider>, String> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;

        let config = ProviderConfig {
            provider: ProviderType::Anthropic,
            name: "anthropic".to_string(),
            priority: 10,
            api_key: Some(api_key),
            base_url: Some("https://api.anthropic.com".to_string()),
            text_model,
            embedding_model,
            max_tokens: 4096,
            temperature: 0.7,
            timeout: 120,
            options: serde_json::Value::Null,
        };

        Ok(Self::create(config))
    }

    fn base_url(&self) -> String {
        self.config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.anthropic.com".to_string())
    }

    fn build_headers(&self) -> Vec<(&str, String)> {
        let mut headers = vec![
            ("Content-Type", "application/json".to_string()),
            ("anthropic-version", "2023-06-01".to_string()),
        ];

        if let Some(api_key) = &self.config.api_key {
            headers.push(("x-api-key", api_key.clone()));
        }

        headers
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Anthropic
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    async fn generate(&self, messages: &[Message]) -> Result<GenerationResponse> {
        debug!(
            "Generating with Anthropic using {} messages",
            messages.len()
        );

        // Extract system message if present
        let system_message = messages
            .iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.clone());

        // Convert non-system messages
        let anthropic_messages: Vec<AnthropicMessage> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(AnthropicMessage::from)
            .collect();

        if anthropic_messages.is_empty() {
            return Err(LlmError::InvalidResponse("No messages to send".to_string()).into());
        }

        let request = AnthropicRequest {
            model: self.config.text_model.clone(),
            messages: anthropic_messages,
            max_tokens: self.config.max_tokens,
            system: system_message,
            temperature: Some(self.config.temperature),
        };

        let url = format!("{}/v1/messages", self.base_url().trim_end_matches('/'));
        let headers = self.build_headers();

        let borrowed_headers: Vec<(&str, &str)> = headers
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect();

        let response: AnthropicResponse = self
            .client
            .post_json(&url, &request, borrowed_headers)
            .await?;

        if response.content.is_empty() {
            return Err(LlmError::InvalidResponse("No content in response".to_string()).into());
        }

        let text = response
            .content
            .iter()
            .map(|block| block.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let total_tokens = response.usage.input_tokens + response.usage.output_tokens;

        info!("Generated {} tokens with {}", total_tokens, response.model);

        Ok(GenerationResponse {
            text,
            tokens_used: Some(total_tokens),
            model: response.model,
            finish_reason: response.stop_reason,
        })
    }

    async fn embed(&self, _text: &str) -> Result<EmbeddingResponse> {
        // Anthropic doesn't provide embeddings directly
        // Users should use Voyage AI or another embedding provider
        Err(LlmError::EmbeddingFailed(
            "Anthropic does not provide native embeddings. Use Voyage AI or another provider."
                .to_string(),
        )
        .into())
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        debug!("Listing models from Anthropic");

        // Anthropic doesn't have a public models endpoint, return known models
        Ok(vec![
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
            "claude-2.1".to_string(),
            "claude-2.0".to_string(),
            "claude-instant-1.2".to_string(),
        ])
    }

    async fn is_model_available(&self, model: &str) -> Result<bool> {
        let models = self.list_models().await?;
        Ok(models.iter().any(|m| m == model))
    }

    fn get_stats(&self) -> ProviderStats {
        self.stats.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_conversion() {
        let msg = Message {
            role: Role::User,
            content: "Hello".to_string(),
        };

        let anthropic_msg = AnthropicMessage::from(&msg);
        assert_eq!(anthropic_msg.role, "user");
        assert_eq!(anthropic_msg.content, "Hello");
    }

    #[test]
    fn test_provider_creation() {
        let config = ProviderConfig {
            provider: ProviderType::Anthropic,
            name: "test".to_string(),
            priority: 1,
            api_key: Some("test-key".to_string()),
            base_url: Some("https://api.anthropic.com".to_string()),
            text_model: "claude-3-opus-20240229".to_string(),
            embedding_model: None,
            max_tokens: 4096,
            temperature: 0.7,
            timeout: 60,
            options: serde_json::Value::Null,
        };

        let provider = AnthropicProvider::create(config);
        assert_eq!(provider.name(), "test");
        assert_eq!(provider.provider_type(), ProviderType::Anthropic);
    }

    #[tokio::test]
    async fn test_list_models() {
        let config = ProviderConfig {
            provider: ProviderType::Anthropic,
            name: "test".to_string(),
            priority: 1,
            api_key: Some("test-key".to_string()),
            base_url: Some("https://api.anthropic.com".to_string()),
            text_model: "claude-3-opus-20240229".to_string(),
            embedding_model: None,
            max_tokens: 4096,
            temperature: 0.7,
            timeout: 60,
            options: serde_json::Value::Null,
        };

        let provider = AnthropicProvider::create(config);
        let models = provider.list_models().await.unwrap();
        assert!(!models.is_empty());
        assert!(models.contains(&"claude-3-opus-20240229".to_string()));
    }
}
