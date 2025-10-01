# Generic AI Agent

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A comprehensive, extensible AI agent framework built in Rust that integrates:

- **🧠 Ollama** - Local LLM inference for text generation and embeddings
- **💾 Vector Store** - Semantic memory and knowledge retrieval 
- **🛠️ MCP Client** - Model Context Protocol for calling external tools
- **⚡ Workflow Engine** - Orchestrates reasoning, memory, and tool usage

## ✨ Features

- **Memory System**: Persistent vector-based memory with semantic search
- **Tool Integration**: Call any MCP-compatible tools and built-in functions
- **Flexible Configuration**: YAML/JSON/TOML configuration with validation
- **Conversation Management**: Automatic history management and context preservation
- **Concurrent Operations**: Async/await throughout with proper error handling
- **Extensible Architecture**: Plugin-style components with trait-based design
- **Comprehensive Testing**: Unit tests, BDD tests, and integration examples

## 🚀 Quick Start

### Prerequisites

1. **Install Rust** (1.75 or later):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install and run Ollama**:
   ```bash
   # Install Ollama
   curl -fsSL https://ollama.ai/install.sh | sh
   
   # Pull required models
   ollama pull llama3.2
   ollama pull nomic-embed-text
   ```

3. **Clone and build**:
   ```bash
   git clone <your-repo-url>
   cd generic-ai-agent
   cargo build --release
   ```

### Basic Usage

```rust
use generic_ai_agent::{Agent, AgentConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize agent with default configuration
    let config = AgentConfig::default();
    let mut agent = Agent::new(config).await?;
    
    // Have a conversation
    let response = agent.process("Hello! What can you help me with?").await?;
    println!("Agent: {}", response);
    
    // Ask for system information (uses built-in tools)
    let response = agent.process("What's my system information?").await?;
    println!("Agent: {}", response);
    
    // Agent remembers context from previous interactions
    let response = agent.process("What did we talk about earlier?").await?;
    println!("Agent: {}", response);
    
    Ok(())
}
```

### Run the Interactive Example

```bash
cargo run --bin agent-example
```

This starts an interactive chat session with:
- Real-time conversation
- Memory demonstrations
- Tool usage examples
- Statistics monitoring

## 🏗️ Architecture

### Core Components

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│    Agent        │    │  Workflow Engine │    │   LLM Client    │
│                 │────│                  │────│                 │
│ • Orchestration │    │ • Step execution │    │ • Text generation│
│ • Configuration │    │ • Decision logic │    │ • Embeddings     │
│ • State mgmt    │    │ • Tool calling   │    │ • Model mgmt     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                        │                        │
         │                        │                        │
         ▼                        ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Memory Store    │    │    MCP Client    │    │   Built-in Tools│
│                 │    │                  │    │                 │
│ • Vector search │    │ • Server mgmt    │    │ • System info   │
│ • Embeddings    │    │ • Tool discovery │    │ • Extensible    │
│ • Persistence   │    │ • JSON-RPC calls │    │ • Async ready   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Workflow Processing

1. **Input Processing**: User message is received and added to conversation history
2. **Memory Retrieval**: Relevant memories are retrieved using embedding similarity
3. **Tool Analysis**: Available tools are analyzed for relevance to the query
4. **LLM Generation**: Context is prepared and sent to the language model
5. **Response Assembly**: Final response is assembled from LLM output, tool results, and memory
6. **Memory Storage**: Conversation is stored in vector memory for future retrieval

## 📋 Configuration

### Basic Configuration

```toml
[llm]
ollama_url = "http://localhost:11434"
text_model = "llama3.2"
embedding_model = "nomic-embed-text"
max_tokens = 4096
temperature = 0.7

[memory]
database_url = "sqlite:memory.db"
embedding_dimension = 768
max_search_results = 10
similarity_threshold = 0.7

[agent]
name = "My AI Assistant"
system_prompt = "You are a helpful AI assistant..."
max_history_length = 20
use_memory = true
use_tools = true
```

### MCP Server Configuration

```toml
[mcp.servers.filesystem]
transport = "stdio"
command = ["mcp-server-filesystem", "/path/to/workspace"]
enabled = true

[mcp.servers.web_search]
transport = "http"
url = "http://localhost:8080/mcp"
auth_token = "your-token"
enabled = true
```

### Programmatic Configuration

```rust
use generic_ai_agent::{AgentBuilder, config::*};

let agent = AgentBuilder::new()
    .with_name("Custom Assistant".to_string())
    .with_system_prompt("You are an expert in Rust programming.".to_string())
    .with_ollama_url("http://custom-ollama:11434".to_string())
    .build()
    .await?;
```

## 🔧 MCP Integration

The agent supports the Model Context Protocol (MCP) for calling external tools:

### Adding MCP Servers

```rust
use generic_ai_agent::config::{McpServerConfig};

let mut config = AgentConfig::default();

// Add filesystem server
config.add_mcp_server("filesystem".to_string(), McpServerConfig {
    transport: "stdio".to_string(),
    command: Some(vec!["mcp-server-filesystem".to_string(), "/workspace".to_string()]),
    enabled: true,
    ..Default::default()
});

// Add web search server  
config.add_mcp_server("search".to_string(), McpServerConfig {
    transport: "http".to_string(),
    url: Some("http://localhost:8080/mcp".to_string()),
    auth_token: Some("token".to_string()),
    enabled: true,
    ..Default::default()
});

let agent = Agent::new(config).await?;
```

### Available MCP Servers

The agent can work with any MCP-compatible server:

- **Filesystem**: File operations and workspace management
- **Web Search**: Internet search and retrieval
- **Database**: SQL query execution
- **API Clients**: REST API interactions
- **Custom Tools**: Your own MCP implementations

## 💾 Memory System

The agent includes a sophisticated memory system for context retention:

### Vector Storage

```rust
// Memory is automatically managed, but you can access it:
let stats = agent.stats().await;
println!("Total memories: {}", stats.memory_stats.total_memories);

// Memories are automatically created from conversations
agent.process("Remember that I prefer TypeScript over JavaScript").await?;
agent.process("What do you know about my programming preferences?").await?;
```

### Custom Memory Operations

```rust
use generic_ai_agent::memory::{MemoryStore, SqliteMemoryStore};

// Direct memory access
let mut store = SqliteMemoryStore::new(config.memory);
store.initialize().await?;

// Store custom memory
let embedding = llm_client.embed("Important information").await?.embedding;
let memory_id = store.store(
    "Important information".to_string(),
    embedding,
    HashMap::from([("type".to_string(), "note".to_string())])
).await?;

// Search memories
let results = store.search(query_embedding, 10, 0.7).await?;
```

## 🛠️ Built-in Tools

The agent comes with several built-in tools:

### System Information Tool

```rust
// Automatically called when user asks about system info
agent.process("What's my system information?").await?;
// Returns: OS, architecture, and other system details
```

### Adding Custom Tools

```rust
use generic_ai_agent::tools::BuiltinTools;
use generic_ai_agent::mcp::{ToolResult, ToolContent};

// Custom tools can be added by extending the BuiltinTools struct
// or by implementing MCP servers
```

## 🧪 Testing

### Run Unit Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test memory

# Run with output
cargo test -- --nocapture
```

### Run BDD Tests

```bash
# Install cucumber runner
cargo install cucumber_rust

# Run behavior tests
cargo test --test bdd_steps
```

### Integration Tests

```bash
# Run the example with test data
cargo run --bin agent-example -- --demo-mode

# Or run specific integration scenarios
cargo test integration_tests
```

## 📊 Monitoring and Debugging

### Agent Statistics

```rust
let stats = agent.stats().await;
println!("Conversation length: {}", stats.conversation_length);
println!("Total memories: {}", stats.memory_stats.total_memories);
println!("Connected MCP servers: {}", stats.mcp_stats.connected_servers);
println!("Available tools: {}", stats.builtin_tools_count + stats.mcp_stats.total_tools);
```

### Logging

```rust
use tracing_subscriber;

// Initialize logging
tracing_subscriber::fmt::init();

// Logs are automatically generated for:
// - LLM requests/responses
// - Memory operations
// - Tool calls
// - Workflow decisions
```

## 🔍 Examples

### Basic Conversation

```rust
let mut agent = Agent::new(AgentConfig::default()).await?;
let response = agent.process("Hello! Tell me about Rust programming.").await?;
println!("{}", response);
```

### Memory-Enhanced Conversation

```rust
// First interaction - store information
agent.process("My name is Alice and I work at Acme Corp").await?;

// Later interaction - retrieve information  
let response = agent.process("What do you remember about me?").await?;
// Agent will recall name and workplace
```

### Tool Usage

```rust
// Request system information
let response = agent.process("Can you show me my system details?").await?;
// Agent automatically calls system_info tool
```

### Multi-step Reasoning

```rust
let response = agent.process(
    "I need system info and also want you to remember that I'm learning Rust"
).await?;
// Agent will:
// 1. Call system_info tool
// 2. Store learning preference in memory  
// 3. Combine both in response
```

## ⚙️ Advanced Usage

### Custom Workflow Steps

```rust
use generic_ai_agent::workflow::{WorkflowStep, WorkflowDecision, WorkflowContext};
use async_trait::async_trait;

struct CustomStep;

#[async_trait]
impl WorkflowStep for CustomStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        // Custom logic here
        Ok(WorkflowDecision::Continue)
    }
    
    fn name(&self) -> &str {
        "custom_step"
    }
}

let workflow = WorkflowEngine::new()
    .add_step(Box::new(CustomStep))
    .add_step(Box::new(MemoryRetrievalStep))
    .add_step(Box::new(ResponseGenerationStep));
```

### Custom Memory Store

```rust
use generic_ai_agent::memory::{MemoryStore, MemoryEntry, SearchResult};

struct CustomMemoryStore {
    // Your implementation
}

#[async_trait]
impl MemoryStore for CustomMemoryStore {
    async fn initialize(&mut self) -> Result<()> { /* ... */ }
    async fn store(&mut self, content: String, embedding: Vec<f32>, metadata: HashMap<String, String>) -> Result<Uuid> { /* ... */ }
    async fn search(&self, query_embedding: Vec<f32>, limit: usize, threshold: f32) -> Result<Vec<SearchResult>> { /* ... */ }
    // ... other methods
}
```

### Custom LLM Client

```rust
use generic_ai_agent::llm::{LlmClient, Message, GenerationResponse, EmbeddingResponse};

struct CustomLlmClient;

#[async_trait]
impl LlmClient for CustomLlmClient {
    async fn generate(&self, messages: &[Message]) -> Result<GenerationResponse> { /* ... */ }
    async fn embed(&self, text: &str) -> Result<EmbeddingResponse> { /* ... */ }
    async fn list_models(&self) -> Result<Vec<String>> { /* ... */ }
    async fn is_model_available(&self, model: &str) -> Result<bool> { /* ... */ }
}
```

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone repository
git clone <repo-url>
cd generic-ai-agent

# Install development dependencies
cargo install cargo-watch cargo-tarpaulin

# Run tests in watch mode
cargo watch -x test

# Check code coverage
cargo tarpaulin --out Html
```

### Code Quality

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy -- -D warnings

# Run all quality checks
make check  # If using provided Makefile
```

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Ollama](https://ollama.ai/) - Local LLM inference
- [MCP](https://modelcontextprotocol.io/) - Model Context Protocol
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [Tokio](https://tokio.rs/) - Async runtime
- [Serde](https://serde.rs/) - Serialization framework

## 📞 Support

- 📖 [Documentation](https://docs.rs/generic-ai-agent)
- 🐛 [Issue Tracker](https://github.com/your-repo/issues)
- 💬 [Discussions](https://github.com/your-repo/discussions)

---

Built with ❤️ in Rust