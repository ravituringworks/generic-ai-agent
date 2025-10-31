//! LLM Provider trait and common types
//!
//! This module defines the common interface that all LLM providers must implement.

use crate::error::Result;
use crate::llm::{EmbeddingResponse, GenerationResponse, Message};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Supported LLM providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Ollama,
    OpenAI,
    Anthropic,
    Google,
    AzureOpenAI,
    Groq,
    Together,
    Replicate,
    HuggingFace,
    Cohere,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::Ollama => write!(f, "ollama"),
            ProviderType::OpenAI => write!(f, "openai"),
            ProviderType::Anthropic => write!(f, "anthropic"),
            ProviderType::Google => write!(f, "google"),
            ProviderType::AzureOpenAI => write!(f, "azure-openai"),
            ProviderType::Groq => write!(f, "groq"),
            ProviderType::Together => write!(f, "together"),
            ProviderType::Replicate => write!(f, "replicate"),
            ProviderType::HuggingFace => write!(f, "huggingface"),
            ProviderType::Cohere => write!(f, "cohere"),
        }
    }
}

/// Configuration for a specific LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type
    pub provider: ProviderType,

    /// Provider name/identifier
    pub name: String,

    /// Priority for fallback (lower is higher priority)
    #[serde(default = "default_priority")]
    pub priority: u8,

    /// API key (usually from environment variable)
    pub api_key: Option<String>,

    /// Base URL for API calls
    pub base_url: Option<String>,

    /// Model name for text generation
    pub text_model: String,

    /// Model name for embeddings
    pub embedding_model: Option<String>,

    /// Maximum tokens for generation
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Temperature for generation
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Provider-specific options
    #[serde(default)]
    pub options: serde_json::Value,
}

fn default_priority() -> u8 {
    10
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_temperature() -> f32 {
    0.7
}

fn default_timeout() -> u64 {
    60
}

/// Trait that all LLM providers must implement
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> ProviderType;

    /// Get the provider name
    fn name(&self) -> &str;

    /// Generate text from a conversation
    async fn generate(&self, messages: &[Message]) -> Result<GenerationResponse>;

    /// Generate embeddings for text
    async fn embed(&self, text: &str) -> Result<EmbeddingResponse>;

    /// List available models
    async fn list_models(&self) -> Result<Vec<String>>;

    /// Check if a specific model is available
    async fn is_model_available(&self, model: &str) -> Result<bool>;

    /// Test if the provider is accessible (health check)
    async fn health_check(&self) -> Result<bool> {
        // Default implementation - try to list models
        match self.list_models().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get provider statistics (tokens used, cost, etc.)
    fn get_stats(&self) -> ProviderStats {
        ProviderStats::default()
    }
}

/// Provider usage statistics
#[derive(Debug, Clone, Default)]
pub struct ProviderStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_tokens_input: u64,
    pub total_tokens_output: u64,
    pub estimated_cost_usd: f64,
}

/// Fallback strategy for multi-provider setup
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FallbackStrategy {
    /// Try providers in priority order
    Priority,
    /// Distribute load across providers
    RoundRobin,
    /// Use cheapest provider first
    CostOptimized,
    /// Route based on task type
    TaskBased,
}

impl Default for FallbackStrategy {
    fn default() -> Self {
        Self::Priority
    }
}

/// Configuration for multi-provider fallback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// Enable fallback
    pub enabled: bool,

    /// Maximum retries per provider
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Delay between retries in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,

    /// Fallback strategy
    #[serde(default)]
    pub strategy: FallbackStrategy,
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    1000
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_retries: default_max_retries(),
            retry_delay_ms: default_retry_delay(),
            strategy: FallbackStrategy::Priority,
        }
    }
}
