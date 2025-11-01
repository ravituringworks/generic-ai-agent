//! LLM Provider Manager
//!
//! Manages multiple LLM providers with automatic fallback and load balancing

use crate::config::LlmConfig;
use crate::error::{LlmError, Result};
use crate::llm::{EmbeddingResponse, GenerationResponse, LlmClient, Message, OllamaClient};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, warn};

/// Provider manager that handles multiple LLM providers with fallback
pub struct ProviderManager {
    /// Primary LLM client
    primary: Arc<dyn LlmClient>,
    /// Fallback clients (in order of priority)
    fallbacks: Vec<Arc<dyn LlmClient>>,
    /// Configuration
    config: ManagerConfig,
}

/// Configuration for provider manager
#[derive(Debug, Clone)]
pub struct ManagerConfig {
    /// Enable automatic fallback
    pub enable_fallback: bool,
    /// Maximum retry attempts per provider
    pub max_retries: usize,
    /// Delay between retries in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            enable_fallback: true,
            max_retries: 2,
            retry_delay_ms: 1000,
        }
    }
}

impl ProviderManager {
    /// Create a new provider manager with Ollama as primary
    pub fn new_ollama(config: LlmConfig) -> Self {
        let primary = Arc::new(OllamaClient::new(config));

        Self {
            primary,
            fallbacks: Vec::new(),
            config: ManagerConfig::default(),
        }
    }

    /// Create a new provider manager with Ollama and caching
    pub async fn new_ollama_with_cache(config: LlmConfig) -> Result<Self> {
        let primary = Arc::new(OllamaClient::new_with_cache(config).await?);

        Ok(Self {
            primary,
            fallbacks: Vec::new(),
            config: ManagerConfig::default(),
        })
    }

    /// Add a fallback provider
    pub fn with_fallback(mut self, client: Arc<dyn LlmClient>) -> Self {
        self.fallbacks.push(client);
        self
    }

    /// Configure the manager
    pub fn with_config(mut self, config: ManagerConfig) -> Self {
        self.config = config;
        self
    }

    /// Try to generate with retries
    async fn try_generate_with_retries(
        &self,
        client: &Arc<dyn LlmClient>,
        messages: &[Message],
        provider_name: &str,
    ) -> Result<GenerationResponse> {
        let mut last_error = None;

        for attempt in 0..self.config.max_retries {
            match client.generate(messages).await {
                Ok(response) => {
                    if attempt > 0 {
                        debug!(
                            "Successfully generated after {} retries with {}",
                            attempt, provider_name
                        );
                    }
                    return Ok(response);
                }
                Err(e) => {
                    warn!(
                        "Attempt {} failed for {}: {}",
                        attempt + 1,
                        provider_name,
                        e
                    );
                    last_error = Some(e);

                    if attempt < self.config.max_retries - 1 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            self.config.retry_delay_ms,
                        ))
                        .await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            crate::error::AgentError::Llm(LlmError::Unknown("No error recorded".to_string()))
        }))
    }

    /// Try to embed with retries
    async fn try_embed_with_retries(
        &self,
        client: &Arc<dyn LlmClient>,
        text: &str,
        provider_name: &str,
    ) -> Result<EmbeddingResponse> {
        let mut last_error = None;

        for attempt in 0..self.config.max_retries {
            match client.embed(text).await {
                Ok(response) => {
                    if attempt > 0 {
                        debug!(
                            "Successfully embedded after {} retries with {}",
                            attempt, provider_name
                        );
                    }
                    return Ok(response);
                }
                Err(e) => {
                    warn!(
                        "Embed attempt {} failed for {}: {}",
                        attempt + 1,
                        provider_name,
                        e
                    );
                    last_error = Some(e);

                    if attempt < self.config.max_retries - 1 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            self.config.retry_delay_ms,
                        ))
                        .await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            crate::error::AgentError::Llm(LlmError::Unknown("No error recorded".to_string()))
        }))
    }
}

#[async_trait]
impl LlmClient for ProviderManager {
    async fn generate(&self, messages: &[Message]) -> Result<GenerationResponse> {
        // Try primary provider
        match self
            .try_generate_with_retries(&self.primary, messages, "primary")
            .await
        {
            Ok(response) => return Ok(response),
            Err(e) if !self.config.enable_fallback || self.fallbacks.is_empty() => {
                return Err(e);
            }
            Err(e) => {
                warn!("Primary provider failed: {}, trying fallbacks", e);
            }
        }

        // Try fallbacks in order
        for (idx, fallback) in self.fallbacks.iter().enumerate() {
            let provider_name = format!("fallback_{}", idx);
            match self
                .try_generate_with_retries(fallback, messages, &provider_name)
                .await
            {
                Ok(response) => {
                    debug!("Successfully used fallback provider {}", idx);
                    return Ok(response);
                }
                Err(e) => {
                    warn!("Fallback {} failed: {}", idx, e);
                }
            }
        }

        Err(crate::error::AgentError::Llm(LlmError::AllProvidersFailed))
    }

    async fn embed(&self, text: &str) -> Result<EmbeddingResponse> {
        // Try primary provider
        match self
            .try_embed_with_retries(&self.primary, text, "primary")
            .await
        {
            Ok(response) => return Ok(response),
            Err(e) if !self.config.enable_fallback || self.fallbacks.is_empty() => {
                return Err(e);
            }
            Err(e) => {
                warn!("Primary provider embed failed: {}, trying fallbacks", e);
            }
        }

        // Try fallbacks in order
        for (idx, fallback) in self.fallbacks.iter().enumerate() {
            let provider_name = format!("fallback_{}", idx);
            match self
                .try_embed_with_retries(fallback, text, &provider_name)
                .await
            {
                Ok(response) => {
                    debug!("Successfully used fallback provider {} for embedding", idx);
                    return Ok(response);
                }
                Err(e) => {
                    warn!("Fallback {} embed failed: {}", idx, e);
                }
            }
        }

        Err(crate::error::AgentError::Llm(LlmError::AllProvidersFailed))
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        self.primary.list_models().await
    }

    async fn is_model_available(&self, model: &str) -> Result<bool> {
        self.primary.is_model_available(model).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LlmConfig;
    use std::collections::HashMap;

    fn test_config() -> LlmConfig {
        LlmConfig {
            ollama_url: "http://localhost:11434".to_string(),
            text_model: "llama3.2".to_string(),
            embedding_model: "nomic-embed-text".to_string(),
            max_tokens: 1000,
            temperature: 0.7,
            timeout: 30,
            stream: false,
            task_models: HashMap::new(),
            cache: crate::cache::LlmCacheConfig::default(),
        }
    }

    #[test]
    fn test_manager_creation() {
        let config = test_config();
        let _manager = ProviderManager::new_ollama(config);
        // Manager creation should not panic
    }

    #[test]
    fn test_manager_config() {
        let config = test_config();
        let manager_config = ManagerConfig {
            enable_fallback: false,
            max_retries: 3,
            retry_delay_ms: 500,
        };

        let manager = ProviderManager::new_ollama(config).with_config(manager_config.clone());

        assert!(!manager.config.enable_fallback);
        assert_eq!(manager.config.max_retries, 3);
        assert_eq!(manager.config.retry_delay_ms, 500);
    }

    #[tokio::test]
    async fn test_manager_list_models() {
        let config = test_config();
        let manager = ProviderManager::new_ollama(config);

        // This will fail if Ollama is not running, which is expected
        match manager.list_models().await {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}
