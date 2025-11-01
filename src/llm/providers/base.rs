//! Base HTTP provider implementation
//!
//! Provides common HTTP client functionality for cloud-based LLM providers

use crate::error::{LlmError, Result};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use tracing::{debug, error};

/// Base HTTP client for LLM providers
pub struct HttpProviderClient {
    client: Client,
    timeout: Duration,
}

impl HttpProviderClient {
    /// Create a new HTTP provider client
    pub fn new(timeout_secs: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Execute a POST request with JSON body
    pub async fn post_json<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
        headers: Vec<(&str, &str)>,
    ) -> Result<R> {
        debug!("Making POST request to: {}", url);

        let mut request = self.client.post(url).json(body);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = tokio::time::timeout(self.timeout, request.send())
            .await
            .map_err(|_| LlmError::Timeout)?
            .map_err(|e| LlmError::ConnectionFailed(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Execute a GET request
    pub async fn get<R: DeserializeOwned>(
        &self,
        url: &str,
        headers: Vec<(&str, &str)>,
    ) -> Result<R> {
        debug!("Making GET request to: {}", url);

        let mut request = self.client.get(url);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = tokio::time::timeout(self.timeout, request.send())
            .await
            .map_err(|_| LlmError::Timeout)?
            .map_err(|e| LlmError::ConnectionFailed(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Handle HTTP response and deserialize
    async fn handle_response<R: DeserializeOwned>(&self, response: Response) -> Result<R> {
        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| format!("HTTP {} error", status));

            error!("API error ({}): {}", status, error_text);

            return Err(match status.as_u16() {
                401 => LlmError::Unauthorized,
                429 => LlmError::RateLimited,
                500..=599 => LlmError::ServerError(error_text),
                _ => LlmError::GenerationFailed(error_text),
            }
            .into());
        }

        response
            .json()
            .await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()).into())
    }

    /// Get the underlying reqwest client
    pub fn client(&self) -> &Client {
        &self.client
    }
}

/// Common trait for OpenAI-compatible API adapters
pub trait OpenAICompatible {
    /// Get the base URL for the provider
    fn base_url(&self) -> &str;

    /// Get the API key
    fn api_key(&self) -> Option<&str>;

    /// Get additional headers specific to the provider
    fn additional_headers(&self) -> Vec<(&str, &str)> {
        Vec::new()
    }

    /// Build auth headers
    fn auth_headers(&self) -> Vec<(&str, String)> {
        let mut headers = Vec::new();
        if let Some(key) = self.api_key() {
            headers.push(("Authorization", format!("Bearer {}", key)));
        }
        headers
    }

    /// Transform endpoint path (for provider-specific routing)
    fn transform_endpoint(&self, endpoint: &str) -> String {
        endpoint.to_string()
    }

    /// Build full URL for an endpoint
    fn build_url(&self, endpoint: &str) -> String {
        let base = self.base_url().trim_end_matches('/');
        let transformed = self.transform_endpoint(endpoint);
        let path = transformed.trim_start_matches('/');
        format!("{}/{}", base, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProvider {
        base_url: String,
        api_key: Option<String>,
    }

    impl OpenAICompatible for TestProvider {
        fn base_url(&self) -> &str {
            &self.base_url
        }

        fn api_key(&self) -> Option<&str> {
            self.api_key.as_deref()
        }
    }

    #[test]
    fn test_url_building() {
        let provider = TestProvider {
            base_url: "https://api.example.com/v1".to_string(),
            api_key: Some("test-key".to_string()),
        };
        let base_url = provider.base_url().to_string();

        assert_eq!(
            provider.build_url("chat/completions"),
            "https://api.example.com/v1/chat/completions"
        );

        assert_eq!(
            provider.build_url("/embeddings"),
            "https://api.example.com/v1/embeddings"
        );
    }

    #[test]
    fn test_auth_headers() {
        let provider = TestProvider {
            base_url: "https://api.example.com".to_string(),
            api_key: Some("test-key".to_string()),
        };

        let headers = provider.auth_headers();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "Authorization");
        assert_eq!(headers[0].1, "Bearer test-key");
    }
}
