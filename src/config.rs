//! Configuration management for the AI agent

use crate::a2a::A2AConfig;
use crate::cache::LlmCacheConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration for the AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// LLM configuration
    pub llm: LlmConfig,

    /// Memory/vector store configuration
    pub memory: MemoryConfig,

    /// MCP server configurations
    pub mcp: McpConfig,

    /// Agent-to-Agent communication configuration
    pub a2a: A2AConfig,

    /// Agent behavior settings
    pub agent: AgentBehaviorConfig,

    /// Workflow configuration
    pub workflow: WorkflowConfig,

    /// Learning configuration
    #[serde(default)]
    pub learning: LearningConfig,

    /// Evaluation configuration
    #[serde(default)]
    pub evaluation: EvaluationConfig,
}

/// Language model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Ollama server URL
    pub ollama_url: String,

    /// Default model for text generation
    pub text_model: String,

    /// Model for embeddings
    pub embedding_model: String,

    /// Maximum tokens for generation
    pub max_tokens: u32,

    /// Temperature for generation
    pub temperature: f32,

    /// Request timeout in seconds
    pub timeout: u64,

    /// Enable streaming responses
    pub stream: bool,

    /// Task-specific model configurations
    #[serde(default)]
    pub task_models: HashMap<String, TaskModelConfig>,

    /// LLM response cache configuration
    #[serde(default)]
    pub cache: LlmCacheConfig,
}

/// Task-specific model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskModelConfig {
    /// Model name for this task
    pub model: String,

    /// Maximum tokens for this task
    #[serde(default)]
    pub max_tokens: Option<u32>,

    /// Temperature for this task
    #[serde(default)]
    pub temperature: Option<f32>,

    /// Custom system prompt for this task
    #[serde(default)]
    pub system_prompt: Option<String>,

    /// Task description/keywords for matching
    #[serde(default)]
    pub keywords: Vec<String>,
}

/// Memory and vector store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Vector store type ("sqlite", "faiss")
    pub store_type: String,

    /// Database file path (for SQLite)
    pub database_url: Option<String>,

    /// Embedding dimension
    pub embedding_dimension: usize,

    /// Maximum number of search results
    pub max_search_results: usize,

    /// Similarity threshold for retrieval
    pub similarity_threshold: f32,

    /// Enable persistent storage
    pub persistent: bool,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Map of server name to server configuration
    pub servers: HashMap<String, McpServerConfig>,

    /// Default timeout for tool calls
    pub default_timeout: u64,

    /// Maximum concurrent tool calls
    pub max_concurrent_calls: usize,

    /// Enable tool call caching
    pub enable_caching: bool,
}

/// Individual MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server transport type ("http", "websocket", "stdio")
    pub transport: String,

    /// Server endpoint URL (for http/websocket)
    pub url: Option<String>,

    /// Command to start server (for stdio)
    pub command: Option<Vec<String>>,

    /// Environment variables
    pub env: Option<HashMap<String, String>>,

    /// Connection timeout
    pub timeout: Option<u64>,

    /// Authentication token
    pub auth_token: Option<String>,

    /// Enable/disable this server
    pub enabled: bool,
}

/// Workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Enable workflow suspend/resume functionality
    pub enable_suspend_resume: bool,

    /// Enable automatic checkpointing
    pub auto_checkpoint: bool,

    /// Checkpoint interval (in steps)
    pub checkpoint_interval: usize,

    /// Maximum number of snapshots to keep
    pub max_snapshots: usize,

    /// Snapshot retention period in days
    pub snapshot_retention_days: i64,

    /// Enable workflow step debugging
    pub debug_steps: bool,
}

/// Agent behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBehaviorConfig {
    /// Agent's name/identity
    pub name: String,

    /// System prompt for the agent
    pub system_prompt: String,

    /// Maximum conversation history length
    pub max_history_length: usize,

    /// Enable memory retrieval
    pub use_memory: bool,

    /// Enable tool calling
    pub use_tools: bool,

    /// Maximum thinking steps for complex queries
    pub max_thinking_steps: usize,

    /// Enable verbose logging
    pub verbose: bool,

    /// Enable reflection after tasks
    #[serde(default)]
    pub enable_reflection: bool,

    /// Enable option evaluation before execution
    #[serde(default)]
    pub enable_option_evaluation: bool,

    /// Minimum quality score to extract best practice
    #[serde(default = "default_min_quality_threshold")]
    pub min_quality_for_best_practice: f32,
}

fn default_min_quality_threshold() -> f32 {
    0.8
}

/// Learning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    /// Soft limit for best practices (triggers consolidation)
    #[serde(default = "default_max_best_practices")]
    pub soft_limit_best_practices: usize,

    /// Hard limit for best practices (triggers pruning)
    #[serde(default = "default_hard_limit_best_practices")]
    pub hard_limit_best_practices: usize,

    /// Maximum failure lessons to store per role
    #[serde(default = "default_max_failure_lessons")]
    pub max_failure_lessons_per_role: usize,

    /// Knowledge relevance threshold for retrieval
    #[serde(default = "default_knowledge_threshold")]
    pub knowledge_relevance_threshold: f32,

    /// Enable automatic consolidation
    #[serde(default = "default_true")]
    pub enable_auto_consolidation: bool,

    /// Similarity threshold for consolidation (0.0-1.0)
    #[serde(default = "default_consolidation_threshold")]
    pub consolidation_similarity_threshold: f32,

    /// Minimum reuse count to keep practice
    #[serde(default = "default_min_reuse_count")]
    pub min_reuse_count_to_keep: u32,

    /// Minimum quality score to keep practice
    #[serde(default = "default_knowledge_threshold")]
    pub min_quality_score_to_keep: f32,

    /// Max age in days for unused practices
    #[serde(default = "default_max_age_days")]
    pub max_age_days_if_unused: i64,

    /// Enable performance metric tracking
    #[serde(default = "default_true")]
    pub track_metrics: bool,

    /// Metric tracking window in days
    #[serde(default = "default_metric_window")]
    pub metric_window_days: i64,

    /// Enable reflection after every task
    #[serde(default = "default_true")]
    pub reflection_after_every_task: bool,

    /// Reflection depth level
    #[serde(default = "default_reflection_depth")]
    pub reflection_depth: String,

    /// Number of relevant memories to retrieve
    #[serde(default = "default_memory_retrieval_count")]
    pub memory_retrieval_count: usize,

    /// External knowledge sources configuration
    #[serde(default)]
    pub external_sources: ExternalSourcesConfig,
}

fn default_max_best_practices() -> usize {
    100
}

fn default_hard_limit_best_practices() -> usize {
    500
}

fn default_max_failure_lessons() -> usize {
    50
}

fn default_knowledge_threshold() -> f32 {
    0.7
}

fn default_consolidation_threshold() -> f32 {
    0.85
}

fn default_min_reuse_count() -> u32 {
    2
}

fn default_max_age_days() -> i64 {
    90
}

fn default_true() -> bool {
    true
}

fn default_metric_window() -> i64 {
    30
}

fn default_reflection_depth() -> String {
    "detailed".to_string()
}

fn default_memory_retrieval_count() -> usize {
    5
}

/// External knowledge sources configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSourcesConfig {
    /// Enable web-based learning
    #[serde(default)]
    pub enable_web_learning: bool,

    /// Enable document ingestion
    #[serde(default)]
    pub enable_document_ingestion: bool,

    /// Max crawl depth for web scraping
    #[serde(default = "default_crawl_depth")]
    pub max_crawl_depth: u32,

    /// Max pages per domain
    #[serde(default = "default_max_pages")]
    pub max_pages_per_domain: usize,

    /// Chunk size for text splitting
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,

    /// Chunk overlap for context preservation
    #[serde(default = "default_chunk_overlap")]
    pub chunk_overlap: usize,

    /// Minimum content quality score
    #[serde(default = "default_min_quality")]
    pub min_content_quality_score: f32,

    /// Allowed domains for web scraping
    #[serde(default)]
    pub allowed_domains: Vec<String>,

    /// Trusted sources for credibility boost
    #[serde(default)]
    pub trusted_sources: Vec<String>,
}

fn default_crawl_depth() -> u32 {
    2
}

fn default_max_pages() -> usize {
    50
}

fn default_chunk_size() -> usize {
    1000
}

fn default_chunk_overlap() -> usize {
    200
}

fn default_min_quality() -> f32 {
    0.6
}

impl Default for ExternalSourcesConfig {
    fn default() -> Self {
        Self {
            enable_web_learning: false,
            enable_document_ingestion: false,
            max_crawl_depth: default_crawl_depth(),
            max_pages_per_domain: default_max_pages(),
            chunk_size: default_chunk_size(),
            chunk_overlap: default_chunk_overlap(),
            min_content_quality_score: default_min_quality(),
            allowed_domains: vec![],
            trusted_sources: vec![],
        }
    }
}

/// Evaluation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    /// Enable cross-agent peer review
    #[serde(default = "default_true")]
    pub enable_cross_review: bool,

    /// Minimum review score to approve artifact
    #[serde(default = "default_min_review_score")]
    pub min_review_score: f32,

    /// Enable self-evaluation
    #[serde(default = "default_true")]
    pub enable_self_evaluation: bool,

    /// Enable structured feedback (scores + comments)
    #[serde(default = "default_true")]
    pub enable_structured_feedback: bool,
}

fn default_min_review_score() -> f32 {
    0.7
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            soft_limit_best_practices: default_max_best_practices(),
            hard_limit_best_practices: default_hard_limit_best_practices(),
            max_failure_lessons_per_role: default_max_failure_lessons(),
            knowledge_relevance_threshold: default_knowledge_threshold(),
            enable_auto_consolidation: true,
            consolidation_similarity_threshold: default_consolidation_threshold(),
            min_reuse_count_to_keep: default_min_reuse_count(),
            min_quality_score_to_keep: default_knowledge_threshold(),
            max_age_days_if_unused: default_max_age_days(),
            track_metrics: true,
            metric_window_days: default_metric_window(),
            reflection_after_every_task: true,
            reflection_depth: default_reflection_depth(),
            memory_retrieval_count: default_memory_retrieval_count(),
            external_sources: ExternalSourcesConfig::default(),
        }
    }
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            enable_cross_review: true,
            min_review_score: default_min_review_score(),
            enable_self_evaluation: true,
            enable_structured_feedback: true,
        }
    }
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            enable_suspend_resume: false,
            auto_checkpoint: false,
            checkpoint_interval: 5,
            max_snapshots: 10,
            snapshot_retention_days: 7,
            debug_steps: false,
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            memory: MemoryConfig::default(),
            mcp: McpConfig::default(),
            a2a: A2AConfig::default(),
            agent: AgentBehaviorConfig::default(),
            workflow: WorkflowConfig::default(),
            learning: LearningConfig::default(),
            evaluation: EvaluationConfig::default(),
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".to_string(),
            text_model: "llama3.2".to_string(),
            embedding_model: "nomic-embed-text".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            timeout: 30,
            stream: false,
            task_models: HashMap::new(),
            cache: LlmCacheConfig::default(),
        }
    }
}

impl LlmConfig {
    /// Get the appropriate model configuration for a given task
    pub fn get_task_model(&self, task: &str) -> TaskModelConfig {
        // Try to find a matching task model by exact name
        if let Some(task_config) = self.task_models.get(task) {
            return task_config.clone();
        }

        // Try to find a matching task model by keywords
        let task_lower = task.to_lowercase();
        for (_, config) in &self.task_models {
            for keyword in &config.keywords {
                if task_lower.contains(&keyword.to_lowercase()) {
                    return config.clone();
                }
            }
        }

        // Return default configuration
        TaskModelConfig {
            model: self.text_model.clone(),
            max_tokens: Some(self.max_tokens),
            temperature: Some(self.temperature),
            system_prompt: None,
            keywords: vec![],
        }
    }

    /// Add a task-specific model configuration
    pub fn add_task_model(&mut self, task_name: String, config: TaskModelConfig) {
        self.task_models.insert(task_name, config);
    }

    /// Remove a task-specific model configuration
    pub fn remove_task_model(&mut self, task_name: &str) -> Option<TaskModelConfig> {
        self.task_models.remove(task_name)
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            store_type: "sqlite".to_string(),
            database_url: Some("sqlite:memory.db".to_string()),
            embedding_dimension: 768,
            max_search_results: 10,
            similarity_threshold: 0.7,
            persistent: true,
        }
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            servers: HashMap::new(),
            default_timeout: 30,
            max_concurrent_calls: 5,
            enable_caching: true,
        }
    }
}

impl Default for AgentBehaviorConfig {
    fn default() -> Self {
        Self {
            name: "Generic AI Agent".to_string(),
            system_prompt: "You are a helpful AI assistant with access to various tools and a memory system. Use your capabilities to assist users effectively.".to_string(),
            max_history_length: 20,
            use_memory: true,
            use_tools: true,
            max_thinking_steps: 5,
            verbose: false,
            enable_reflection: false,
            enable_option_evaluation: false,
            min_quality_for_best_practice: default_min_quality_threshold(),
        }
    }
}

impl AgentConfig {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let config = match path.as_ref().extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str(&content)?,
            Some("toml") => toml::from_str(&content)?,
            Some("yaml") | Some("yml") => serde_yml::from_str(&content)?,
            _ => return Err(anyhow::anyhow!("Unsupported config file format")),
        };
        Ok(config)
    }

    /// Save configuration to a file
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = match path.as_ref().extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::to_string_pretty(self)?,
            Some("toml") => toml::to_string(self)?,
            Some("yaml") | Some("yml") => serde_yml::to_string(self)?,
            _ => return Err(anyhow::anyhow!("Unsupported config file format")),
        };
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate URLs
        if !self.llm.ollama_url.starts_with("http") {
            return Err(anyhow::anyhow!(
                "Invalid Ollama URL: {}",
                self.llm.ollama_url
            ));
        }

        // Validate models
        if self.llm.text_model.is_empty() {
            return Err(anyhow::anyhow!("Text model name cannot be empty"));
        }

        if self.llm.embedding_model.is_empty() {
            return Err(anyhow::anyhow!("Embedding model name cannot be empty"));
        }

        // Validate memory config
        if self.memory.embedding_dimension == 0 {
            return Err(anyhow::anyhow!(
                "Embedding dimension must be greater than 0"
            ));
        }

        if self.memory.similarity_threshold < 0.0 || self.memory.similarity_threshold > 1.0 {
            return Err(anyhow::anyhow!(
                "Similarity threshold must be between 0.0 and 1.0"
            ));
        }

        // Validate agent config
        if self.agent.name.is_empty() {
            return Err(anyhow::anyhow!("Agent name cannot be empty"));
        }

        if self.agent.max_history_length == 0 {
            return Err(anyhow::anyhow!("Max history length must be greater than 0"));
        }

        Ok(())
    }

    /// Add an MCP server configuration
    pub fn add_mcp_server(&mut self, name: String, config: McpServerConfig) {
        self.mcp.servers.insert(name, config);
    }

    /// Remove an MCP server configuration
    pub fn remove_mcp_server(&mut self, name: &str) -> Option<McpServerConfig> {
        self.mcp.servers.remove(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AgentConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.llm.ollama_url, "http://localhost:11434");
        assert_eq!(config.agent.name, "Generic AI Agent");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AgentConfig::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid Ollama URL should fail
        config.llm.ollama_url = "invalid-url".to_string();
        assert!(config.validate().is_err());

        // Reset and test empty model
        config = AgentConfig::default();
        config.llm.text_model = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_mcp_server_management() {
        let mut config = AgentConfig::default();

        let server_config = McpServerConfig {
            transport: "http".to_string(),
            url: Some("http://localhost:8000".to_string()),
            command: None,
            env: None,
            timeout: Some(30),
            auth_token: None,
            enabled: true,
        };

        config.add_mcp_server("test-server".to_string(), server_config.clone());
        assert!(config.mcp.servers.contains_key("test-server"));

        let removed = config.remove_mcp_server("test-server");
        assert!(removed.is_some());
        assert!(!config.mcp.servers.contains_key("test-server"));
    }
}
