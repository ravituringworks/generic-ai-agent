//! Google Gemini provider implementation

use crate::error::{LlmError, Result};
use crate::llm::provider::{LlmProvider, ProviderConfig, ProviderStats, ProviderType};
use crate::llm::providers::base::HttpProviderClient;
use crate::llm::{EmbeddingResponse, GenerationResponse, Message, Role};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Gemini message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiContent {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiPart {
    pub text: String,
}

impl From<&Message> for GeminiContent {
    fn from(msg: &Message) -> Self {
        Self {
            role: match msg.role {
                Role::User => "user".to_string(),
                Role::Assistant => "model".to_string(),
                Role::System => "user".to_string(), // System messages as user
            },
            parts: vec![GeminiPart {
                text: msg.content.clone(),
            }],
        }
    }
}

/// Gemini API request
#[derive(Debug, Serialize)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Serialize)]
pub struct GenerationConfig {
    pub temperature: f32,
    pub max_output_tokens: u32,
}

/// Gemini API response
#[derive(Debug, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
    #[serde(default)]
    pub usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub content: GeminiContent,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsageMetadata {
    pub prompt_token_count: u32,
    pub candidates_token_count: u32,
    pub total_token_count: u32,
}

/// Gemini embedding request
#[derive(Debug, Serialize)]
pub struct GeminiEmbedRequest {
    pub content: GeminiContent,
}

/// Gemini embedding response
#[derive(Debug, Deserialize)]
pub struct GeminiEmbedResponse {
    pub embedding: EmbeddingData,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingData {
    pub values: Vec<f32>,
}

/// Google Gemini provider
pub struct GoogleProvider {
    client: HttpProviderClient,
    config: ProviderConfig,
    stats: ProviderStats,
}

impl GoogleProvider {
    /// Create a new Google Gemini provider
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
        let api_key = std::env::var("GOOGLE_API_KEY")
            .map_err(|_| "GOOGLE_API_KEY environment variable not set")?;

        let config = ProviderConfig {
            provider: ProviderType::Google,
            name: "google".to_string(),
            priority: 10,
            api_key: Some(api_key),
            base_url: Some("https://generativelanguage.googleapis.com/v1beta".to_string()),
            text_model,
            embedding_model,
            max_tokens: 2048,
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
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string())
    }

    fn api_key(&self) -> Option<&str> {
        self.config.api_key.as_deref()
    }
}

#[async_trait]
impl LlmProvider for GoogleProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Google
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    async fn generate(&self, messages: &[Message]) -> Result<GenerationResponse> {
        debug!(
            "Generating with Google Gemini using {} messages",
            messages.len()
        );

        // Convert messages to Gemini format
        let contents: Vec<GeminiContent> = messages.iter().map(GeminiContent::from).collect();

        if contents.is_empty() {
            return Err(LlmError::InvalidResponse("No messages to send".to_string()).into());
        }

        let request = GeminiRequest {
            contents,
            generation_config: Some(GenerationConfig {
                temperature: self.config.temperature,
                max_output_tokens: self.config.max_tokens,
            }),
        };

        let api_key = self
            .api_key()
            .ok_or_else(|| LlmError::Unauthorized)?;

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url().trim_end_matches('/'),
            self.config.text_model,
            api_key
        );

        let headers = vec![("Content-Type", "application/json")];

        let response: GeminiResponse = self.client.post_json(&url, &request, headers).await?;

        if response.candidates.is_empty() {
            return Err(LlmError::InvalidResponse("No candidates in response".to_string()).into());
        }

        let candidate = &response.candidates[0];
        let text = candidate
            .content
            .parts
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let tokens_used = response
            .usage_metadata
            .map(|u| u.total_token_count);

        info!(
            "Generated {} tokens with {}",
            tokens_used.unwrap_or(0),
            self.config.text_model
        );

        Ok(GenerationResponse {
            text,
            tokens_used,
            model: self.config.text_model.clone(),
            finish_reason: candidate.finish_reason.clone(),
        })
    }

    async fn embed(&self, text: &str) -> Result<EmbeddingResponse> {
        debug!("Generating embedding with Google for text length {}", text.len());

        let embedding_model = self
            .config
            .embedding_model
            .as_ref()
            .ok_or_else(|| LlmError::EmbeddingFailed("No embedding model configured".to_string()))?;

        let request = GeminiEmbedRequest {
            content: GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: text.to_string(),
                }],
            },
        };

        let api_key = self
            .api_key()
            .ok_or_else(|| LlmError::Unauthorized)?;

        let url = format!(
            "{}/models/{}:embedContent?key={}",
            self.base_url().trim_end_matches('/'),
            embedding_model,
            api_key
        );

        let headers = vec![("Content-Type", "application/json")];

        let response: GeminiEmbedResponse = self.client.post_json(&url, &request, headers).await?;

        info!(
            "Generated embedding with dimension {}",
            response.embedding.values.len()
        );

        Ok(EmbeddingResponse {
            embedding: response.embedding.values,
            model: embedding_model.clone(),
        })
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        debug!("Listing models from Google Gemini");

        // Return known Gemini models
        Ok(vec![
            "gemini-pro".to_string(),
            "gemini-pro-vision".to_string(),
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
            "embedding-001".to_string(),
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

        let gemini_content = GeminiContent::from(&msg);
        assert_eq!(gemini_content.role, "user");
        assert_eq!(gemini_content.parts[0].text, "Hello");
    }

    #[test]
    fn test_provider_creation() {
        let config = ProviderConfig {
            provider: ProviderType::Google,
            name: "test".to_string(),
            priority: 1,
            api_key: Some("test-key".to_string()),
            base_url: Some("https://generativelanguage.googleapis.com/v1beta".to_string()),
            text_model: "gemini-pro".to_string(),
            embedding_model: Some("embedding-001".to_string()),
            max_tokens: 2048,
            temperature: 0.7,
            timeout: 60,
            options: serde_json::Value::Null,
        };

        let provider = GoogleProvider::create(config);
        assert_eq!(provider.name(), "test");
        assert_eq!(provider.provider_type(), ProviderType::Google);
    }

    #[tokio::test]
    async fn test_list_models() {
        let config = ProviderConfig {
            provider: ProviderType::Google,
            name: "test".to_string(),
            priority: 1,
            api_key: Some("test-key".to_string()),
            base_url: None,
            text_model: "gemini-pro".to_string(),
            embedding_model: None,
            max_tokens: 2048,
            temperature: 0.7,
            timeout: 60,
            options: serde_json::Value::Null,
        };

        let provider = GoogleProvider::create(config);
        let models = provider.list_models().await.unwrap();
        assert!(!models.is_empty());
        assert!(models.contains(&"gemini-pro".to_string()));
    }
}
