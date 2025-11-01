//! Language model integration using Ollama

use crate::cache::LlmCache;
use crate::config::LlmConfig;
use crate::error::{LlmError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info};

/// Message role in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Response from text generation
#[derive(Debug, Clone)]
pub struct GenerationResponse {
    pub text: String,
    pub tokens_used: Option<u32>,
    pub model: String,
    pub finish_reason: Option<String>,
}

/// Embedding response
#[derive(Debug, Clone)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
    pub model: String,
}

/// Trait for language model operations
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Generate text from a conversation
    async fn generate(&self, messages: &[Message]) -> Result<GenerationResponse>;

    /// Generate embeddings for text
    async fn embed(&self, text: &str) -> Result<EmbeddingResponse>;

    /// List available models
    async fn list_models(&self) -> Result<Vec<String>>;

    /// Check if model is available
    async fn is_model_available(&self, model: &str) -> Result<bool>;
}

/// Ollama client implementation
pub struct OllamaClient {
    client: reqwest::Client,
    config: LlmConfig,
    cache: Option<Arc<LlmCache>>,
}

/// Ollama API request for generation
#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    options: OllamaOptions,
}

/// Ollama API options
#[derive(Debug, Serialize)]
struct OllamaOptions {
    num_predict: u32,
    temperature: f32,
}

/// Ollama API response for generation
#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    model: String,
    message: Message,
    done: bool,
    #[serde(default)]
    done_reason: Option<String>,
    #[serde(default)]
    eval_count: Option<u32>,
    #[serde(default)]
    #[allow(dead_code)]
    created_at: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    total_duration: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)]
    load_duration: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    #[allow(dead_code)]
    prompt_eval_duration: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)]
    eval_duration: Option<u64>,
}

/// Ollama API request for embeddings
#[derive(Debug, Serialize)]
struct OllamaEmbedRequest {
    model: String,
    prompt: String,
}

/// Ollama API response for embeddings
#[derive(Debug, Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

/// Ollama models list response
#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
    #[serde(default)]
    #[allow(dead_code)]
    size: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)]
    digest: Option<String>,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(config: LlmConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            cache: None,
        }
    }

    /// Create a new Ollama client with cache
    pub async fn new_with_cache(config: LlmConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .expect("Failed to create HTTP client");

        let cache = if config.cache.enabled {
            Some(Arc::new(LlmCache::new(config.cache.clone()).await?))
        } else {
            None
        };

        Ok(Self {
            client,
            config,
            cache,
        })
    }

    /// Get the base URL for API calls
    fn api_url(&self, endpoint: &str) -> String {
        format!(
            "{}/api/{}",
            self.config.ollama_url.trim_end_matches('/'),
            endpoint
        )
    }
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn generate(&self, messages: &[Message]) -> Result<GenerationResponse> {
        debug!("Generating text with {} messages", messages.len());

        // Try cache first if available
        if let Some(cache) = &self.cache {
            let messages_json =
                serde_json::to_string(&messages).unwrap_or_else(|_| format!("{:?}", messages));

            let system_prompt = messages
                .iter()
                .find(|m| m.role == Role::System)
                .map(|m| m.content.as_str());

            let cache_key = LlmCache::compute_cache_key(
                &messages_json,
                &self.config.text_model,
                self.config.temperature,
                self.config.max_tokens,
                system_prompt,
            );

            if let Ok(Some(cached_response)) = cache.get(&cache_key).await {
                debug!("Using cached response");
                return Ok(GenerationResponse {
                    text: cached_response,
                    tokens_used: None,
                    model: self.config.text_model.clone(),
                    finish_reason: Some("cached".to_string()),
                });
            }
        }

        let request = OllamaGenerateRequest {
            model: self.config.text_model.clone(),
            messages: messages.to_vec(),
            stream: self.config.stream,
            options: OllamaOptions {
                num_predict: self.config.max_tokens,
                temperature: self.config.temperature,
            },
        };

        let url = self.api_url("chat");
        debug!("Making request to: {}", url);

        let response = timeout(
            Duration::from_secs(self.config.timeout),
            self.client.post(&url).json(&request).send(),
        )
        .await
        .map_err(|_| LlmError::Timeout)?
        .map_err(|e| LlmError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Ollama API error: {}", error_text);
            return Err(LlmError::GenerationFailed(error_text).into());
        }

        let ollama_response: OllamaGenerateResponse = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        if !ollama_response.done {
            return Err(LlmError::InvalidResponse("Incomplete response".to_string()).into());
        }

        info!(
            "Generated {} tokens",
            ollama_response.eval_count.unwrap_or(0)
        );

        let response_text = ollama_response.message.content.clone();

        // Cache the response if cache is available
        if let Some(cache) = &self.cache {
            let messages_json =
                serde_json::to_string(&messages).unwrap_or_else(|_| format!("{:?}", messages));

            let system_prompt = messages
                .iter()
                .find(|m| m.role == Role::System)
                .map(|m| m.content.as_str());

            let cache_key = LlmCache::compute_cache_key(
                &messages_json,
                &self.config.text_model,
                self.config.temperature,
                self.config.max_tokens,
                system_prompt,
            );

            if let Err(e) = cache
                .set(
                    cache_key,
                    response_text.clone(),
                    ollama_response.model.clone(),
                    self.config.temperature,
                )
                .await
            {
                error!("Failed to cache response: {}", e);
            }
        }

        Ok(GenerationResponse {
            text: response_text,
            tokens_used: ollama_response.eval_count,
            model: ollama_response.model,
            finish_reason: ollama_response.done_reason,
        })
    }

    async fn embed(&self, text: &str) -> Result<EmbeddingResponse> {
        debug!("Generating embedding for text of length {}", text.len());

        let request = OllamaEmbedRequest {
            model: self.config.embedding_model.clone(),
            prompt: text.to_string(),
        };

        let url = self.api_url("embeddings");
        debug!("Making embedding request to: {}", url);

        let response = timeout(
            Duration::from_secs(self.config.timeout),
            self.client.post(&url).json(&request).send(),
        )
        .await
        .map_err(|_| LlmError::Timeout)?
        .map_err(|e| LlmError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Ollama embedding API error: {}", error_text);
            return Err(LlmError::EmbeddingFailed(error_text).into());
        }

        let ollama_response: OllamaEmbedResponse = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        info!(
            "Generated embedding with dimension {}",
            ollama_response.embedding.len()
        );

        Ok(EmbeddingResponse {
            embedding: ollama_response.embedding,
            model: self.config.embedding_model.clone(),
        })
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        debug!("Listing available models");

        let url = self.api_url("tags");

        let response = timeout(
            Duration::from_secs(self.config.timeout),
            self.client.get(&url).send(),
        )
        .await
        .map_err(|_| LlmError::Timeout)?
        .map_err(|e| LlmError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::GenerationFailed(error_text).into());
        }

        let models_response: OllamaModelsResponse = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        let models: Vec<String> = models_response.models.into_iter().map(|m| m.name).collect();

        info!("Found {} models", models.len());
        Ok(models)
    }

    async fn is_model_available(&self, model: &str) -> Result<bool> {
        let models = self.list_models().await?;
        Ok(models.iter().any(|m| m == model))
    }
}

/// Helper function to create a system message
pub fn system_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::System,
        content: content.into(),
    }
}

/// Helper function to create a user message
pub fn user_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::User,
        content: content.into(),
    }
}

/// Helper function to create an assistant message
pub fn assistant_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::Assistant,
        content: content.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{
        automock,
        predicate::{self, *},
    };

    #[automock]
    #[async_trait]
    #[allow(dead_code)]
    pub trait MockLlmClient: Send + Sync {
        async fn generate(&self, messages: &[Message]) -> Result<GenerationResponse>;
        async fn embed(&self, text: &str) -> Result<EmbeddingResponse>;
        async fn list_models(&self) -> Result<Vec<String>>;
        async fn is_model_available(&self, model: &str) -> Result<bool>;
    }

    #[test]
    fn test_message_creation() {
        let system_msg = system_message("You are a helpful assistant");
        assert_eq!(system_msg.role, Role::System);
        assert_eq!(system_msg.content, "You are a helpful assistant");

        let user_msg = user_message("Hello");
        assert_eq!(user_msg.role, Role::User);
        assert_eq!(user_msg.content, "Hello");

        let assistant_msg = assistant_message("Hi there!");
        assert_eq!(assistant_msg.role, Role::Assistant);
        assert_eq!(assistant_msg.content, "Hi there!");
    }

    #[test]
    fn test_ollama_client_creation() {
        let config = LlmConfig::default();
        let client = OllamaClient::new(config);
        assert_eq!(client.config.text_model, "llama3.2");
    }

    #[test]
    fn test_api_url_generation() {
        let config = LlmConfig::default();
        let client = OllamaClient::new(config);

        assert_eq!(client.api_url("chat"), "http://localhost:11434/api/chat");
        assert_eq!(
            client.api_url("embeddings"),
            "http://localhost:11434/api/embeddings"
        );
    }

    #[tokio::test]
    async fn test_mock_llm_client() {
        let mut mock_client = MockMockLlmClient::new();

        mock_client
            .expect_generate()
            .with(predicate::always())
            .times(1)
            .returning(|_| {
                Ok(GenerationResponse {
                    text: "Hello! How can I help you?".to_string(),
                    tokens_used: Some(10),
                    model: "test-model".to_string(),
                    finish_reason: Some("stop".to_string()),
                })
            });

        let messages = vec![user_message("Hello")];
        let response = mock_client.generate(&messages).await.unwrap();

        assert_eq!(response.text, "Hello! How can I help you?");
        assert_eq!(response.tokens_used, Some(10));
    }
}
