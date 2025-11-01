//! Error handling for the AI agent

use thiserror::Error;

/// Result type alias for the AI agent
pub type Result<T> = std::result::Result<T, AgentError>;

/// Main error type for the AI agent
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),

    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),

    #[error("MCP error: {0}")]
    Mcp(#[from] McpError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Workflow error: {0}")]
    Workflow(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),

    #[error("A2A error: {0}")]
    A2A(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

/// Errors related to language model operations
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Embedding failed: {0}")]
    EmbeddingFailed(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Timeout: operation took too long")]
    Timeout,

    #[error("All providers failed")]
    AllProvidersFailed,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Errors related to memory/vector store operations
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Store not initialized")]
    NotInitialized,

    #[error("Invalid embedding dimension: expected {expected}, got {actual}")]
    InvalidDimension { expected: usize, actual: usize },

    #[error("Search failed: {0}")]
    SearchFailed(String),

    #[error("Storage failed: {0}")]
    StorageFailed(String),

    #[error("Index not found: {0}")]
    IndexNotFound(String),
}

/// Errors related to MCP operations
#[derive(Error, Debug)]
pub enum McpError {
    #[error("Server connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool execution failed: {tool}: {reason}")]
    ToolExecutionFailed { tool: String, reason: String },

    #[error("Invalid tool parameters: {0}")]
    InvalidParameters(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

impl AgentError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AgentError::Llm(LlmError::Timeout)
                | AgentError::Llm(LlmError::ConnectionFailed(_))
                | AgentError::Mcp(McpError::ConnectionFailed(_))
                | AgentError::Mcp(McpError::Timeout(_))
                | AgentError::Http(_)
        )
    }

    /// Get error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            AgentError::Llm(_) => "llm",
            AgentError::Memory(_) => "memory",
            AgentError::Mcp(_) => "mcp",
            AgentError::Config(_) => "config",
            AgentError::Workflow(_) => "workflow",
            AgentError::Io(_) => "io",
            AgentError::Serialization(_) => "serialization",
            AgentError::Http(_) => "http",
            AgentError::Database(_) => "database",
            AgentError::Generic(_) => "generic",
            AgentError::A2A(_) => "a2a",
            AgentError::Network(_) => "network",
            AgentError::NotFound(_) => "not_found",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryable() {
        let timeout_error = AgentError::Llm(LlmError::Timeout);
        assert!(timeout_error.is_retryable());

        let config_error = AgentError::Config("invalid config".to_string());
        assert!(!config_error.is_retryable());
    }

    #[test]
    fn test_error_category() {
        let llm_error = AgentError::Llm(LlmError::Timeout);
        assert_eq!(llm_error.category(), "llm");

        let memory_error = AgentError::Memory(MemoryError::NotInitialized);
        assert_eq!(memory_error.category(), "memory");
    }
}
