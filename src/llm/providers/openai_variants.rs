//! OpenAI-compatible provider variants (Groq, Together AI, Azure OpenAI)

use crate::llm::provider::{LlmProvider, ProviderConfig, ProviderType};
use crate::llm::providers::base::OpenAICompatible;
use crate::llm::providers::openai_compatible::OpenAICompatibleProvider;
use std::sync::Arc;

// ============================================================================
// Groq Provider
// ============================================================================

/// Groq adapter (fast inference with LPU)
pub struct GroqAdapter {
    api_key: Option<String>,
}

impl GroqAdapter {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

impl OpenAICompatible for GroqAdapter {
    fn base_url(&self) -> &str {
        "https://api.groq.com/openai/v1"
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
}

/// Groq provider
pub type GroqProvider = OpenAICompatibleProvider<GroqAdapter>;

impl GroqProvider {
    pub fn create(config: ProviderConfig) -> Arc<dyn LlmProvider> {
        let adapter = GroqAdapter::new(config.api_key.clone());
        Arc::new(OpenAICompatibleProvider::new(adapter, config))
    }

    pub fn from_env(text_model: String) -> std::result::Result<Arc<dyn LlmProvider>, String> {
        let api_key = std::env::var("GROQ_API_KEY")
            .map_err(|_| "GROQ_API_KEY environment variable not set")?;

        let config = ProviderConfig {
            provider: ProviderType::Groq,
            name: "groq".to_string(),
            priority: 10,
            api_key: Some(api_key),
            base_url: Some("https://api.groq.com/openai/v1".to_string()),
            text_model,
            embedding_model: None, // Groq doesn't offer embeddings
            max_tokens: 8192,
            temperature: 0.7,
            timeout: 60,
            options: serde_json::Value::Null,
        };

        Ok(Self::create(config))
    }
}

// ============================================================================
// Together AI Provider
// ============================================================================

/// Together AI adapter
pub struct TogetherAdapter {
    api_key: Option<String>,
}

impl TogetherAdapter {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

impl OpenAICompatible for TogetherAdapter {
    fn base_url(&self) -> &str {
        "https://api.together.xyz/v1"
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
}

/// Together AI provider
pub type TogetherProvider = OpenAICompatibleProvider<TogetherAdapter>;

impl TogetherProvider {
    pub fn create(config: ProviderConfig) -> Arc<dyn LlmProvider> {
        let adapter = TogetherAdapter::new(config.api_key.clone());
        Arc::new(OpenAICompatibleProvider::new(adapter, config))
    }

    pub fn from_env(
        text_model: String,
        embedding_model: Option<String>,
    ) -> std::result::Result<Arc<dyn LlmProvider>, String> {
        let api_key = std::env::var("TOGETHER_API_KEY")
            .map_err(|_| "TOGETHER_API_KEY environment variable not set")?;

        let config = ProviderConfig {
            provider: ProviderType::Together,
            name: "together".to_string(),
            priority: 10,
            api_key: Some(api_key),
            base_url: Some("https://api.together.xyz/v1".to_string()),
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

// ============================================================================
// Azure OpenAI Provider
// ============================================================================

/// Azure OpenAI adapter
pub struct AzureOpenAIAdapter {
    endpoint: String,
    api_key: Option<String>,
    api_version: String,
    deployment_name: String,
}

impl AzureOpenAIAdapter {
    pub fn new(
        endpoint: String,
        deployment_name: String,
        api_key: Option<String>,
        api_version: Option<String>,
    ) -> Self {
        Self {
            endpoint,
            api_key,
            api_version: api_version.unwrap_or_else(|| "2024-02-15-preview".to_string()),
            deployment_name,
        }
    }
}

impl OpenAICompatible for AzureOpenAIAdapter {
    fn base_url(&self) -> &str {
        &self.endpoint
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    fn auth_headers(&self) -> Vec<(&str, String)> {
        let mut headers = Vec::new();
        if let Some(key) = self.api_key.as_ref() {
            headers.push(("api-key", key.clone()));
        }
        headers
    }

    fn transform_endpoint(&self, endpoint: &str) -> String {
        // Azure uses deployment-based routing
        match endpoint {
            "chat/completions" => {
                format!(
                    "openai/deployments/{}/chat/completions?api-version={}",
                    self.deployment_name, self.api_version
                )
            }
            "embeddings" => {
                format!(
                    "openai/deployments/{}/embeddings?api-version={}",
                    self.deployment_name, self.api_version
                )
            }
            "models" => format!("openai/models?api-version={}", self.api_version),
            _ => format!("{}?api-version={}", endpoint, self.api_version),
        }
    }
}

/// Azure OpenAI provider
pub type AzureOpenAIProvider = OpenAICompatibleProvider<AzureOpenAIAdapter>;

impl AzureOpenAIProvider {
    pub fn create(config: ProviderConfig) -> Arc<dyn LlmProvider> {
        // Extract deployment name from options or use model name as fallback
        let deployment_name = config
            .options
            .get("deployment_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&config.text_model)
            .to_string();

        let api_version = config
            .options
            .get("api_version")
            .and_then(|v| v.as_str())
            .map(String::from);

        let endpoint = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://your-resource.openai.azure.com".to_string());

        let adapter = AzureOpenAIAdapter::new(
            endpoint,
            deployment_name,
            config.api_key.clone(),
            api_version,
        );

        Arc::new(OpenAICompatibleProvider::new(adapter, config))
    }

    pub fn from_env(
        endpoint: String,
        deployment_name: String,
        text_model: String,
    ) -> std::result::Result<Arc<dyn LlmProvider>, String> {
        let api_key = std::env::var("AZURE_OPENAI_API_KEY")
            .map_err(|_| "AZURE_OPENAI_API_KEY environment variable not set")?;

        let mut options = serde_json::Map::new();
        options.insert(
            "deployment_name".to_string(),
            serde_json::Value::String(deployment_name),
        );

        let config = ProviderConfig {
            provider: ProviderType::AzureOpenAI,
            name: "azure-openai".to_string(),
            priority: 10,
            api_key: Some(api_key),
            base_url: Some(endpoint),
            text_model,
            embedding_model: None,
            max_tokens: 4096,
            temperature: 0.7,
            timeout: 120,
            options: serde_json::Value::Object(options),
        };

        Ok(Self::create(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groq_adapter() {
        let adapter = GroqAdapter::new(Some("test-key".to_string()));
        assert_eq!(adapter.base_url(), "https://api.groq.com/openai/v1");
        assert_eq!(adapter.api_key(), Some("test-key"));
    }

    #[test]
    fn test_together_adapter() {
        let adapter = TogetherAdapter::new(Some("test-key".to_string()));
        assert_eq!(adapter.base_url(), "https://api.together.xyz/v1");
    }

    #[test]
    fn test_azure_endpoint_transform() {
        let adapter = AzureOpenAIAdapter::new(
            "https://my-resource.openai.azure.com".to_string(),
            "gpt-4-deployment".to_string(),
            Some("test-key".to_string()),
            Some("2024-02-15-preview".to_string()),
        );

        let chat_endpoint = adapter.transform_endpoint("chat/completions");
        assert!(chat_endpoint.contains("gpt-4-deployment"));
        assert!(chat_endpoint.contains("2024-02-15-preview"));
    }
}
