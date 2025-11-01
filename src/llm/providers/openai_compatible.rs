//! OpenAI-compatible API implementation
//!
//! Provides a shared implementation for providers that use OpenAI-compatible APIs:
//! - OpenAI
//! - Azure OpenAI
//! - Groq
//! - Together AI
//! - And others

use crate::error::{LlmError, Result};
use crate::llm::provider::{LlmProvider, ProviderConfig, ProviderStats, ProviderType};
use crate::llm::providers::base::{HttpProviderClient, OpenAICompatible};
use crate::llm::{EmbeddingResponse, GenerationResponse, Message, Role};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// OpenAI-compatible chat completion request
#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub stream: bool,
}

/// OpenAI message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

impl From<&Message> for OpenAIMessage {
    fn from(msg: &Message) -> Self {
        Self {
            role: match msg.role {
                Role::System => "system".to_string(),
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
            },
            content: msg.content.clone(),
        }
    }
}

/// OpenAI chat completion response
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    #[serde(default)]
    pub usage: Option<UsageInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: OpenAIMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsageInfo {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// OpenAI embedding request
#[derive(Debug, Serialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: String,
}

/// OpenAI embedding response
#[derive(Debug, Deserialize)]
pub struct EmbeddingResponseData {
    pub data: Vec<EmbeddingData>,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingData {
    pub embedding: Vec<f32>,
    pub index: u32,
}

/// OpenAI models list response
#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<ModelData>,
}

#[derive(Debug, Deserialize)]
pub struct ModelData {
    pub id: String,
    #[serde(default)]
    pub created: Option<u64>,
    #[serde(default)]
    pub owned_by: Option<String>,
}

/// Generic OpenAI-compatible provider
pub struct OpenAICompatibleProvider<T: OpenAICompatible + Send + Sync> {
    adapter: T,
    client: HttpProviderClient,
    config: ProviderConfig,
    stats: ProviderStats,
}

impl<T: OpenAICompatible + Send + Sync> OpenAICompatibleProvider<T> {
    /// Create a new OpenAI-compatible provider
    pub fn new(adapter: T, config: ProviderConfig) -> Self {
        let client = HttpProviderClient::new(config.timeout);

        Self {
            adapter,
            client,
            config,
            stats: ProviderStats::default(),
        }
    }

    /// Build headers for requests
    fn build_headers(&self) -> Vec<(&str, String)> {
        let mut headers = self.adapter.auth_headers();

        // Add content type
        headers.push(("Content-Type", "application/json".to_string()));

        headers
    }
}

#[async_trait]
impl<T: OpenAICompatible + Send + Sync> LlmProvider for OpenAICompatibleProvider<T> {
    fn provider_type(&self) -> ProviderType {
        self.config.provider
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    async fn generate(&self, messages: &[Message]) -> Result<GenerationResponse> {
        debug!(
            "Generating with {} using {} messages",
            self.name(),
            messages.len()
        );

        let request = ChatCompletionRequest {
            model: self.config.text_model.clone(),
            messages: messages.iter().map(OpenAIMessage::from).collect(),
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            stream: false,
        };

        let url = self.adapter.build_url("chat/completions");
        let headers = self.build_headers();

        // Convert owned headers to borrowed
        let borrowed_headers: Vec<(&str, &str)> =
            headers.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response: ChatCompletionResponse = self
            .client
            .post_json(&url, &request, borrowed_headers)
            .await?;

        if response.choices.is_empty() {
            return Err(LlmError::InvalidResponse("No choices in response".to_string()).into());
        }

        let choice = &response.choices[0];
        let tokens_used = response.usage.map(|u| u.total_tokens);

        info!(
            "Generated {} tokens with {}",
            tokens_used.unwrap_or(0),
            response.model
        );

        Ok(GenerationResponse {
            text: choice.message.content.clone(),
            tokens_used,
            model: response.model,
            finish_reason: choice.finish_reason.clone(),
        })
    }

    async fn embed(&self, text: &str) -> Result<EmbeddingResponse> {
        debug!(
            "Generating embedding with {} for text length {}",
            self.name(),
            text.len()
        );

        let embedding_model = self.config.embedding_model.as_ref().ok_or_else(|| {
            LlmError::EmbeddingFailed("No embedding model configured".to_string())
        })?;

        let request = EmbeddingRequest {
            model: embedding_model.clone(),
            input: text.to_string(),
        };

        let url = self.adapter.build_url("embeddings");
        let headers = self.build_headers();

        let borrowed_headers: Vec<(&str, &str)> =
            headers.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response: EmbeddingResponseData = self
            .client
            .post_json(&url, &request, borrowed_headers)
            .await?;

        if response.data.is_empty() {
            return Err(LlmError::EmbeddingFailed("No embeddings in response".to_string()).into());
        }

        let embedding = response.data[0].embedding.clone();

        info!(
            "Generated embedding with dimension {} using {}",
            embedding.len(),
            response.model
        );

        Ok(EmbeddingResponse {
            embedding,
            model: response.model,
        })
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        debug!("Listing models from {}", self.name());

        let url = self.adapter.build_url("models");
        let headers = self.build_headers();

        let borrowed_headers: Vec<(&str, &str)> =
            headers.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response: ModelsResponse = self.client.get(&url, borrowed_headers).await?;

        let models: Vec<String> = response.data.into_iter().map(|m| m.id).collect();

        info!("Found {} models from {}", models.len(), self.name());
        Ok(models)
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

    struct TestAdapter {
        base_url: String,
        api_key: Option<String>,
    }

    impl OpenAICompatible for TestAdapter {
        fn base_url(&self) -> &str {
            &self.base_url
        }

        fn api_key(&self) -> Option<&str> {
            self.api_key.as_deref()
        }
    }

    #[test]
    fn test_message_conversion() {
        let msg = Message {
            role: Role::User,
            content: "Hello".to_string(),
        };

        let openai_msg = OpenAIMessage::from(&msg);
        assert_eq!(openai_msg.role, "user");
        assert_eq!(openai_msg.content, "Hello");
    }

    #[test]
    fn test_provider_creation() {
        let adapter = TestAdapter {
            base_url: "https://api.example.com/v1".to_string(),
            api_key: Some("test-key".to_string()),
        };

        let config = ProviderConfig {
            provider: ProviderType::OpenAI,
            name: "test".to_string(),
            priority: 1,
            api_key: Some("test-key".to_string()),
            base_url: Some("https://api.example.com/v1".to_string()),
            text_model: "gpt-4".to_string(),
            embedding_model: Some("text-embedding-ada-002".to_string()),
            max_tokens: 4096,
            temperature: 0.7,
            timeout: 60,
            options: serde_json::Value::Null,
        };

        let _provider = OpenAICompatibleProvider::new(adapter, config);
    }
}
