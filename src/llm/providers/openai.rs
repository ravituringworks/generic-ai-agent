//! OpenAI provider implementation

use crate::llm::provider::{LlmProvider, ProviderConfig, ProviderType};
use crate::llm::providers::base::OpenAICompatible;
use crate::llm::providers::openai_compatible::OpenAICompatibleProvider;
use std::sync::Arc;

/// OpenAI adapter
pub struct OpenAIAdapter {
    base_url: String,
    api_key: Option<String>,
}

impl OpenAIAdapter {
    pub fn new(api_key: Option<String>, base_url: Option<String>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            api_key,
        }
    }
}

impl OpenAICompatible for OpenAIAdapter {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
}

/// OpenAI provider
pub type OpenAIProvider = OpenAICompatibleProvider<OpenAIAdapter>;

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn create(config: ProviderConfig) -> Arc<dyn LlmProvider> {
        let adapter = OpenAIAdapter::new(config.api_key.clone(), config.base_url.clone());
        Arc::new(OpenAICompatibleProvider::new(adapter, config))
    }

    /// Create from environment variable
    pub fn from_env(
        text_model: String,
        embedding_model: Option<String>,
    ) -> std::result::Result<Arc<dyn LlmProvider>, String> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY environment variable not set")?;

        let config = ProviderConfig {
            provider: ProviderType::OpenAI,
            name: "openai".to_string(),
            priority: 10,
            api_key: Some(api_key),
            base_url: None,
            text_model,
            embedding_model,
            max_tokens: 4096,
            temperature: 0.7,
            timeout: 120,
            options: serde_json::Value::Null,
        };

        Ok(Self::create(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let adapter = OpenAIAdapter::new(Some("test-key".to_string()), None);
        assert_eq!(adapter.base_url(), "https://api.openai.com/v1");
        assert_eq!(adapter.api_key(), Some("test-key"));
    }

    #[test]
    fn test_custom_base_url() {
        let adapter = OpenAIAdapter::new(
            Some("test-key".to_string()),
            Some("https://custom.endpoint.com/v1".to_string()),
        );
        assert_eq!(adapter.base_url(), "https://custom.endpoint.com/v1");
    }
}
