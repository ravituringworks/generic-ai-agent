//! Generic AI Agent Library
//! 
//! A comprehensive AI agent framework that integrates:
//! - Ollama for language model interactions
//! - Vector store for semantic search and memory
//! - MCP (Model Context Protocol) client for tool calling
//! 
//! # Example
//! 
//! ```rust,no_run
//! use generic_ai_agent::{Agent, AgentConfig};
//! 
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = AgentConfig::default();
//!     let agent = Agent::new(config).await?;
//!     
//!     let response = agent
//!         .process("What's the weather like today?")
//!         .await?;
//!     
//!     println!("Agent response: {}", response);
//!     Ok(())
//! }
//! ```

pub mod agent;
pub mod config;
pub mod error;
pub mod llm;
pub mod memory;
pub mod mcp;
pub mod tools;
pub mod unified_storage;
pub mod workflow;

// Re-export main types
pub use agent::{Agent, AgentBuilder};
pub use config::{AgentConfig, LlmConfig, MemoryConfig, McpConfig};
pub use error::{AgentError, Result};
pub use memory::{MemoryStore, VectorStore};
pub use mcp::{McpClient, McpTool, ToolCall, ToolResult};
pub use unified_storage::{
    UnifiedStorage, InMemoryUnifiedStorage, StorageManager, ResourceId,
    SuspendedWorkflow, MemoryThread, MemoryMessage, TraceData, EvalDataset, EvalScore,
    StorageStats, RetentionPolicy, CleanupStats, TraceFilters,
    SuspendReason, ResumeCondition, MessageRole, TraceStatus, TraceEvent
};
pub use workflow::{WorkflowContext, WorkflowStep, WorkflowEngine};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}