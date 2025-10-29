//! Configuration management for the AI agent

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::a2a::A2AConfig;

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
    
    /// Snapshot storage directory
    pub snapshot_storage_dir: Option<String>,
    
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
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            enable_suspend_resume: false,
            snapshot_storage_dir: None,
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
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
            _ => return Err(anyhow::anyhow!("Unsupported config file format")),
        };
        Ok(config)
    }
    
    /// Save configuration to a file
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = match path.as_ref().extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::to_string_pretty(self)?,
            Some("toml") => toml::to_string(self)?,
            Some("yaml") | Some("yml") => serde_yaml::to_string(self)?,
            _ => return Err(anyhow::anyhow!("Unsupported config file format")),
        };
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate URLs
        if !self.llm.ollama_url.starts_with("http") {
            return Err(anyhow::anyhow!("Invalid Ollama URL: {}", self.llm.ollama_url));
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
            return Err(anyhow::anyhow!("Embedding dimension must be greater than 0"));
        }
        
        if self.memory.similarity_threshold < 0.0 || self.memory.similarity_threshold > 1.0 {
            return Err(anyhow::anyhow!("Similarity threshold must be between 0.0 and 1.0"));
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
    use tempfile::NamedTempFile;
    
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