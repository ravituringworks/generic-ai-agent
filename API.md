# Generic AI Agent - API Documentation

## Core Types

### Agent

The main agent that orchestrates all components.

```rust
pub struct Agent {
    // Private fields
}

impl Agent {
    /// Create a new agent with the given configuration
    pub async fn new(config: AgentConfig) -> Result<Self>;
    
    /// Process a user message and return a response
    pub async fn process(&mut self, user_input: &str) -> Result<String>;
    
    /// Get agent statistics
    pub async fn stats(&self) -> AgentStats;
    
    /// Clear conversation history
    pub fn clear_conversation(&mut self);
    
    /// Add a message to the conversation
    pub fn add_message(&mut self, message: Message);
    
    /// Get current conversation
    pub fn get_conversation(&self) -> &[Message];
}
```

### AgentBuilder

Builder pattern for creating agents with custom configurations.

```rust
pub struct AgentBuilder {
    // Private fields
}

impl AgentBuilder {
    pub fn new() -> Self;
    pub fn with_config(self, config: AgentConfig) -> Self;
    pub fn with_name(self, name: String) -> Self;
    pub fn with_system_prompt(self, prompt: String) -> Self;
    pub fn with_ollama_url(self, url: String) -> Self;
    pub async fn build(self) -> Result<Agent>;
}
```

## Configuration

### AgentConfig

Main configuration structure for the AI agent.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub llm: LlmConfig,
    pub memory: MemoryConfig,
    pub mcp: McpConfig,
    pub agent: AgentBehaviorConfig,
}

impl AgentConfig {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self>;
    
    /// Save configuration to a file
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> anyhow::Result<()>;
    
    /// Validate the configuration
    pub fn validate(&self) -> anyhow::Result<()>;
    
    /// Add an MCP server configuration
    pub fn add_mcp_server(&mut self, name: String, config: McpServerConfig);
    
    /// Remove an MCP server configuration
    pub fn remove_mcp_server(&mut self, name: &str) -> Option<McpServerConfig>;
}
```

### LlmConfig

Configuration for the language model client.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub ollama_url: String,
    pub text_model: String,
    pub embedding_model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout: u64,
    pub stream: bool,
}
```

### MemoryConfig

Configuration for the memory/vector store.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub store_type: String,
    pub database_url: Option<String>,
    pub embedding_dimension: usize,
    pub max_search_results: usize,
    pub similarity_threshold: f32,
    pub persistent: bool,
}
```

### McpConfig & McpServerConfig

Configuration for MCP servers.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub servers: HashMap<String, McpServerConfig>,
    pub default_timeout: u64,
    pub max_concurrent_calls: usize,
    pub enable_caching: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub transport: String,
    pub url: Option<String>,
    pub command: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub timeout: Option<u64>,
    pub auth_token: Option<String>,
    pub enabled: bool,
}
```

## Language Model Integration

### LlmClient Trait

Abstract interface for language model operations.

```rust
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Generate text from a conversation
    async fn generate(&self, messages: &[Message]) -> Result<GenerationResponse>;
    
    /// Generate embeddings for text
    async fn embed(&self, text: &str) -> Result<EmbeddingResponse>;
    
    /// List available models
    async fn list_models(&self) -> Result<Vec<String>>;
    
    /// Check if model is available
    async fn is_model_available(&self, model: &str) -> Result<bool>;
}
```

### Message Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

// Helper functions
pub fn system_message(content: impl Into<String>) -> Message;
pub fn user_message(content: impl Into<String>) -> Message;
pub fn assistant_message(content: impl Into<String>) -> Message;
```

### Response Types

```rust
#[derive(Debug, Clone)]
pub struct GenerationResponse {
    pub text: String,
    pub tokens_used: Option<u32>,
    pub model: String,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
    pub model: String,
}
```

### OllamaClient

Concrete implementation of LlmClient for Ollama.

```rust
pub struct OllamaClient {
    // Private fields
}

impl OllamaClient {
    pub fn new(config: LlmConfig) -> Self;
}
```

## Memory System

### MemoryStore Trait

Abstract interface for memory operations.

```rust
#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn initialize(&mut self) -> Result<()>;
    async fn store(&mut self, content: String, embedding: Vec<f32>, metadata: HashMap<String, String>) -> Result<Uuid>;
    async fn search(&self, query_embedding: Vec<f32>, limit: usize, threshold: f32) -> Result<Vec<SearchResult>>;
    async fn get(&self, id: Uuid) -> Result<Option<MemoryEntry>>;
    async fn update(&mut self, id: Uuid, content: Option<String>, embedding: Option<Vec<f32>>, metadata: Option<HashMap<String, String>>) -> Result<()>;
    async fn delete(&mut self, id: Uuid) -> Result<()>;
    async fn list(&self, limit: Option<usize>) -> Result<Vec<MemoryEntry>>;
    async fn clear(&mut self) -> Result<()>;
    async fn stats(&self) -> Result<MemoryStats>;
}
```

### Memory Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Uuid,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub entry: MemoryEntry,
    pub similarity: f32,
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub embedding_dimension: usize,
    pub store_size_bytes: Option<usize>,
}
```

### SqliteMemoryStore

SQLite implementation of MemoryStore.

```rust
pub struct SqliteMemoryStore {
    // Private fields
}

impl SqliteMemoryStore {
    pub fn new(config: MemoryConfig) -> Self;
}
```

## MCP Integration

### McpClient

Client for managing MCP server connections and tool calls.

```rust
pub struct McpClient {
    // Private fields
}

impl McpClient {
    pub fn new(config: McpConfig) -> Self;
    pub async fn add_server(&mut self, name: String, server_config: McpServerConfig) -> Result<()>;
    pub fn list_tools(&self) -> Vec<(String, &McpTool)>;
    pub fn find_tool_server(&self, tool_name: &str) -> Option<(&str, &McpTool)>;
    pub async fn call_tool(&self, tool_call: ToolCall) -> Result<ToolResult>;
    pub async fn call_tools(&self, tool_calls: Vec<ToolCall>) -> Vec<ToolResult>;
    pub async fn remove_server(&mut self, name: &str) -> Result<()>;
    pub fn stats(&self) -> McpStats;
}
```

### MCP Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub id: String,
    pub content: Vec<ToolContent>,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { uri: String, text: Option<String> },
}
```

## Workflow Engine

### WorkflowEngine

Orchestrates the execution of workflow steps.

```rust
pub struct WorkflowEngine {
    // Private fields
}

impl WorkflowEngine {
    pub fn new() -> Self;
    pub fn add_step(self, step: Box<dyn WorkflowStep>) -> Self;
    pub fn with_default_steps(self) -> Self;
    pub async fn execute(&self, context: WorkflowContext) -> Result<WorkflowResult>;
}
```

### WorkflowStep Trait

Interface for individual workflow steps.

```rust
#[async_trait]
pub trait WorkflowStep: Send + Sync {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision>;
    fn name(&self) -> &str;
}
```

### Workflow Types

```rust
#[derive(Debug, Clone)]
pub struct WorkflowContext {
    pub messages: Vec<Message>,
    pub memories: Vec<SearchResult>,
    pub available_tools: Vec<String>,
    pub tool_results: HashMap<String, ToolResult>,
    pub metadata: HashMap<String, String>,
    pub step_count: usize,
    pub max_steps: usize,
}

#[derive(Debug, Clone)]
pub enum WorkflowDecision {
    Continue,
    Complete(String),
    Jump(String),
    ExecuteTools(Vec<ToolCall>),
    RetrieveMemories(String),
}

#[derive(Debug)]
pub struct WorkflowResult {
    pub response: String,
    pub context: WorkflowContext,
    pub completed: bool,
    pub steps_executed: usize,
    pub pending_tool_calls: Option<Vec<ToolCall>>,
    pub pending_memory_query: Option<String>,
}
```

### Built-in Workflow Steps

```rust
pub struct MemoryRetrievalStep;
pub struct ToolAnalysisStep;
pub struct ResponseGenerationStep;
```

## Built-in Tools

### BuiltinTools

Registry for built-in tools.

```rust
pub struct BuiltinTools {
    // Private fields
}

impl BuiltinTools {
    pub fn new() -> Self;
    pub fn list_tools(&self) -> Vec<String>;
    pub async fn execute(&self, tool_name: &str) -> Option<ToolResult>;
}
```

### Tool Functions

```rust
pub fn create_system_info_tool() -> ToolCall;
pub async fn execute_system_info() -> ToolResult;
```

## Error Handling

### Error Types

```rust
pub type Result<T> = std::result::Result<T, AgentError>;

#[derive(Error, Debug)]
pub enum AgentError {
    Llm(#[from] LlmError),
    Memory(#[from] MemoryError),
    Mcp(#[from] McpError),
    Config(String),
    Workflow(String),
    Io(#[from] std::io::Error),
    Serialization(#[from] serde_json::Error),
    Http(#[from] reqwest::Error),
    Database(#[from] sqlx::Error),
    Generic(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum LlmError {
    ConnectionFailed(String),
    ModelNotFound(String),
    GenerationFailed(String),
    EmbeddingFailed(String),
    InvalidResponse(String),
    Timeout,
}

#[derive(Error, Debug)]
pub enum MemoryError {
    NotInitialized,
    InvalidDimension { expected: usize, actual: usize },
    SearchFailed(String),
    StorageFailed(String),
    IndexNotFound(String),
}

#[derive(Error, Debug)]
pub enum McpError {
    ConnectionFailed(String),
    ToolNotFound(String),
    ToolExecutionFailed { tool: String, reason: String },
    InvalidParameters(String),
    ProtocolError(String),
    Timeout(String),
}
```

### Error Utilities

```rust
impl AgentError {
    pub fn is_retryable(&self) -> bool;
    pub fn category(&self) -> &'static str;
}
```

## Statistics

### AgentStats

Comprehensive statistics about the agent's state.

```rust
#[derive(Debug, Clone)]
pub struct AgentStats {
    pub conversation_length: usize,
    pub memory_stats: MemoryStats,
    pub mcp_stats: McpStats,
    pub builtin_tools_count: usize,
}

#[derive(Debug, Clone)]
pub struct McpStats {
    pub connected_servers: usize,
    pub total_tools: usize,
    pub servers: HashMap<String, usize>,
}
```

## Constants

```rust
pub const VERSION: &str;
```

## Feature Flags

- `default`: Includes SQLite support
- `sqlite`: SQLite vector store support
- `faiss`: FAISS vector store support (optional)

## Usage Patterns

### Basic Agent Usage

```rust
// 1. Create configuration
let config = AgentConfig::default();

// 2. Initialize agent
let mut agent = Agent::new(config).await?;

// 3. Process messages
let response = agent.process("Hello!").await?;

// 4. Check statistics
let stats = agent.stats().await;
```

### Builder Pattern

```rust
let agent = AgentBuilder::new()
    .with_name("Assistant".to_string())
    .with_system_prompt("You are helpful.".to_string())
    .build()
    .await?;
```

### Custom Implementations

```rust
// Custom LLM client
struct MyLlmClient;

#[async_trait]
impl LlmClient for MyLlmClient {
    // Implementation
}

// Custom memory store
struct MyMemoryStore;

#[async_trait]
impl MemoryStore for MyMemoryStore {
    // Implementation
}

// Custom workflow step
struct MyWorkflowStep;

#[async_trait]
impl WorkflowStep for MyWorkflowStep {
    // Implementation
}
```

This API provides a comprehensive, type-safe interface for building AI agents with memory, tool calling, and flexible workflow orchestration capabilities.