//! Generic AI Agent Library
//!
//! A comprehensive AI agent framework that integrates:
//! - Ollama for language model interactions
//! - Vector store for semantic search and memory
//! - MCP (Model Context Protocol) client for tool calling
//! - A2A (Agent-to-Agent) communication for external agent integration
//!
//! # Example
//!
//! ```rust,no_run
//! use the_agency::{Agent, AgentConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = AgentConfig::default();
//!     let mut agent = Agent::new(config).await?;
//!     
//!     let response = agent
//!         .process("What's the weather like today?")
//!         .await?;
//!     
//!     println!("Agent response: {}", response);
//!     Ok(())
//! }
//! ```

pub mod a2a;
pub mod agent;
pub mod api;
pub mod cache;
pub mod config;
pub mod error;
pub mod knowledge;
pub mod llm;
pub mod mcp;
pub mod memory;
pub mod organization;
pub mod saga;
pub mod tools;
pub mod ui_workflow_storage;
pub mod unified_storage;
pub mod workflow;

// Re-export main types
pub use a2a::{
    A2AClient, A2AConfig, A2AManager, A2AMessage, A2AResponse, A2AStats, AgentCapabilities,
    AgentId, AgentRegistration, AgentStatus, HttpA2AClient, MessageHandler, MessagePayload,
    MessagePriority, MessageType, ProtocolType, ResponseStatus,
};
pub use agent::{Agent, AgentBuilder};
pub use cache::{CacheStats, LlmCache, LlmCacheConfig};
pub use config::{AgentConfig, LlmConfig, McpConfig, MemoryConfig};
pub use error::{AgentError, Result};
pub use knowledge::{
    AdaptiveKnowledgeManager, ConsolidatedKnowledge, ContentChunker, DocumentFormat,
    IngestionConfig, IngestionResult, KnowledgeChunk, KnowledgeConsolidator, KnowledgeSource,
    KnowledgeStats, ManagementResult,
};
pub use mcp::{McpClient, McpTool, ToolCall, ToolResult};
pub use memory::{MemoryStore, VectorStore};
pub use organization::{
    AgentStatus as OrgAgentStatus, CollaborativeWorkspace, Organization, OrganizationAgent,
    OrganizationRole, RoleCategory, TaskPriority, TaskStatus, WorkspaceTask,
};
pub use saga::{
    SagaContext, SagaOrchestrator, SagaResult, SagaStep, SagaStepState, SagaWorkflowStep,
};
pub use unified_storage::{
    CleanupStats, EvalDataset, EvalScore, InMemoryUnifiedStorage, MemoryMessage, MemoryThread,
    MessageRole, ResourceId, ResumeCondition, RetentionPolicy, StorageManager, StorageStats,
    SuspendReason, SuspendedWorkflow, TraceData, TraceEvent, TraceFilters, TraceStatus,
    UnifiedStorage,
};
pub use workflow::{WorkflowContext, WorkflowEngine, WorkflowStep};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(VERSION.starts_with(char::is_numeric));
    }
}
