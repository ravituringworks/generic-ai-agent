# The Agency

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A comprehensive, extensible AI agent framework built in Rust that integrates:

- **🤖 Multi-Provider LLMs** - OpenAI, Anthropic, Google, Groq, Together AI, Azure OpenAI, Ollama
- **💾 Vector Store** - Semantic memory and knowledge retrieval
- **🛠️ MCP Client** - Model Context Protocol for calling external tools
- **⚡ Workflow Engine** - Orchestrates reasoning, memory, and tool usage
- **🌐 A2A Communication** - Agent-to-Agent communication for multi-agent systems
- **🔄 State Management** - Pause, resume, and persistent agent state
- **🗃️ Unified Storage** - Centralized data management across components
- **🧠 Knowledge Management** - Organizational learning and external knowledge ingestion
- **⚡ Saga Workflows** - Distributed transaction patterns for complex operations

## ✨ Features

### Core Capabilities

- **Multi-Provider LLMs**: Support for 7+ LLM providers with automatic fallback
  - Local: Ollama
  - Cloud: OpenAI, Anthropic Claude, Google Gemini
  - Fast: Groq (LPU acceleration)
  - Enterprise: Azure OpenAI
  - Open Source: Together AI (50+ models)
- **Task-Based LLM**: Configure different models for different task types (code, creative, math, etc.)
- **Memory System**: Persistent vector-based memory with semantic search
- **Document RAG**: PDF processing with table extraction and semantic indexing
- **Tool Integration**: Call any MCP-compatible tools and built-in functions
- **Flexible Configuration**: YAML/JSON/TOML configuration with validation
- **Conversation Management**: Automatic history management and context preservation
- **Concurrent Operations**: Async/await throughout with proper error handling
- **Extensible Architecture**: Plugin-style components with trait-based design
- **Specialized Agents**: Domain-specific agents like Robotics Scientist for research tasks
- **Knowledge Management**: Persistent learning, external knowledge ingestion, and organizational memory
- **Saga Workflows**: Distributed transaction patterns for complex multi-agent operations
- **Comprehensive Testing**: Unit tests, BDD tests, and integration examples

### Advanced Features

- **🌐 Agent-to-Agent Communication**: Multi-protocol support (HTTP, WebSocket, Redis, RabbitMQ)
- **🔍 Service Discovery**: Capability-based agent discovery and health monitoring
- **🔒 Security**: Authentication, encryption, rate limiting, and access control
- **⏸️ State Management**: Pause, resume, and checkpoint agent execution
- **🗄️ Unified Storage**: Centralized data management with multiple backend support
- **📊 Real-time Collaboration**: Multi-agent workflows and task distribution
- **🔄 Load Balancing**: Automatic request distribution across agent networks
- **🧠 Organizational Learning**: Knowledge capture from every task with persistent memory
- **🌐 External Knowledge**: Web scraping, document ingestion, and content consolidation
- **⚡ Saga Transactions**: Distributed workflows with compensation and rollback

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
   git clone https://github.com/ravituringworks/the-agency.git
   cd the-agency
   cargo build --release
   ```

### Basic Usage

```rust
use the_agency::{Agent, AgentConfig};

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

### Multi-Provider LLM Support

The Agency supports 7+ LLM providers with a unified interface:

```rust
use the_agency::llm::providers::{
    OpenAIProvider, AnthropicProvider, GoogleProvider,
    GroqProvider, TogetherProvider, AzureOpenAIProvider,
};
use the_agency::llm::{user_message, provider::LlmProvider};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use OpenAI
    let openai = OpenAIProvider::from_env(
        "gpt-4".to_string(),
        Some("text-embedding-ada-002".to_string())
    )?;
    
    let messages = vec![user_message("What is Rust?")];
    let response = openai.generate(&messages).await?;
    println!("OpenAI: {}", response.text);
    
    // Use Anthropic Claude
    let claude = AnthropicProvider::from_env(
        "claude-3-opus-20240229".to_string(),
        None
    )?;
    let response = claude.generate(&messages).await?;
    println!("Claude: {}", response.text);
    
    // Use Groq for fast inference
    let groq = GroqProvider::from_env("llama3-70b-8192".to_string())?;
    let response = groq.generate(&messages).await?;
    println!("Groq: {}", response.text);
    
    Ok(())
}
```

Configure with environment variables:

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GOOGLE_API_KEY="..."
export GROQ_API_KEY="gsk_..."
export TOGETHER_API_KEY="..."
export AZURE_OPENAI_API_KEY="..."
export AZURE_OPENAI_ENDPOINT="https://your-resource.openai.azure.com"
```

See `examples/multi_provider_usage.rs` for complete examples and `docs/MULTI_PROVIDER_ARCHITECTURE.md` for architecture details.

### Multi-Agent Communication

```rust
use the_agency::{Agent, AgentConfig, AgentId};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create AI Agent with A2A enabled
    let mut config = AgentConfig::default();
    config.a2a.discovery.enabled = true;
    config.a2a.agent_id = AgentId::new("ai_network", "assistant");
    
    let agent = Agent::new(config).await?;
    
    // Start A2A communication
    agent.start_a2a().await?;
    
    // Discover specialized agents
    let specialists = agent.discover_agents("data_analysis").await?;
    println!("Found {} specialist agents", specialists.len());
    
    // Send task to another agent
    let target_agent = AgentId::new("ai_network", "data_specialist");
    let response = agent.send_to_agent(
        target_agent, 
        "Analyze sales data for Q4 trends"
    ).await?;
    
    println!("Specialist response: {}", response);
    
    Ok(())
}
```

### Knowledge Management & Organizational Learning

Agents learn from every task execution and can ingest external knowledge:

```rust
use the_agency::knowledge::{KnowledgeEntry, KnowledgeManager};

// Agents automatically capture knowledge from tasks
let knowledge = KnowledgeEntry {
    task_title: "Implement RL Algorithm".to_string(),
    task_description: "Develop PPO implementation for robotic control".to_string(),
    agent_role: "ResearchEngineerRL".to_string(),
    approach: "Used stable-baselines3 with custom environment".to_string(),
    outcome: "Achieved 85% success rate in simulation".to_string(),
    insights: vec![
        "Hyperparameter tuning critical for convergence".to_string(),
        "Environment reward shaping improved learning".to_string(),
    ],
    timestamp: chrono::Utc::now(),
};

// Store knowledge for future use
let manager = KnowledgeManager::new(config).await?;
manager.store_knowledge(knowledge).await?;

// Agents learn from past experiences
let similar_tasks = manager.query_similar_experiences("RL implementation", 5).await?;
```

**Features:**
- **Persistent Learning**: Knowledge captured from every task execution
- **Context-Aware Execution**: Agents query past experiences for enhanced task performance
- **External Knowledge Ingestion**: Web scraping, document parsing, and content consolidation
- **Organizational Memory**: Cross-agent knowledge sharing and best practices
- **Quality Management**: Automatic consolidation and deduplication of knowledge

### Saga Workflows for Distributed Transactions

Handle complex multi-step operations with automatic rollback and compensation:

```rust
use the_agency::saga::{Saga, SagaStep, SagaContext};

// Define saga steps
let step1 = SagaStep::new("validate_input", validate_input);
let step2 = SagaStep::new("process_payment", process_payment);
let step3 = SagaStep::new("update_inventory", update_inventory);
let step4 = SagaStep::new("send_notification", send_notification);

// Create compensating actions for rollback
let compensation1 = SagaStep::new("rollback_validation", rollback_validation);
let compensation2 = SagaStep::new("refund_payment", refund_payment);
let compensation3 = SagaStep::new("restore_inventory", restore_inventory);

// Build saga with compensations
let saga = Saga::new("order_processing")
    .add_step(step1, Some(compensation1))?
    .add_step(step2, Some(compensation2))?
    .add_step(step3, Some(compensation3))?
    .add_step(step4, None)?; // No compensation needed for notification

// Execute saga
let context = SagaContext::new();
let result = saga.execute(context).await;

// Automatic rollback on failure
if result.is_err() {
    // Compensations executed in reverse order
    saga.rollback().await?;
}
```

**Features:**
- **Distributed Transactions**: Multi-step operations across services
- **Automatic Compensation**: Rollback failed operations with custom logic
- **Fault Tolerance**: Graceful handling of partial failures
- **State Persistence**: Saga state saved for recovery
- **Timeout Management**: Configurable timeouts and retry policies

### Document RAG (Retrieval-Augmented Generation)

Process and query PDF documents with table extraction:

```rust
use the_agency::{
    memory::{MemoryStore, SqliteMemoryStore},
    llm::{OllamaClient, LlmClient, user_message},
    config::MemoryConfig,
};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize memory store
    let memory_config = MemoryConfig {
        store_type: "sqlite".to_string(),
        database_url: Some("sqlite:documents.db".to_string()),
        embedding_dimension: 768,
        max_search_results: 5,
        similarity_threshold: 0.6,
        persistent: true,
    };
    
    let mut memory_store = SqliteMemoryStore::new(memory_config);
    memory_store.initialize().await?;
    
    // Initialize LLM client
    let llm_config = the_agency::config::LlmConfig {
        ollama_url: "http://localhost:11434".to_string(),
        text_model: "llama3.2".to_string(),
        embedding_model: "nomic-embed-text".to_string(),
        max_tokens: 4096,
        temperature: 0.7,
        timeout: 60,
        stream: false,
    };
    
    let llm_client = OllamaClient::new(llm_config);
    
    // Index a PDF document
    let pdf_path = Path::new("document.pdf");
    let document_text = extract_pdf_text(pdf_path).await?;
    
    // Create embeddings and store document sections
    let sections = split_into_sections(&document_text);
    for (i, section) in sections.iter().enumerate() {
        let embedding = llm_client.embed(section).await?.embedding;
        let metadata = HashMap::from([
            ("document".to_string(), "document.pdf".to_string()),
            ("section".to_string(), i.to_string()),
            ("type".to_string(), "text".to_string()),
        ]);
        memory_store.store(section.clone(), embedding, metadata).await?;
    }
    
    // Query the document
    let question = "What are the main findings?";
    let question_embedding = llm_client.embed(question).await?.embedding;
    
    // Retrieve relevant sections
    let search_results = memory_store
        .search(question_embedding, 3, 0.6)
        .await?;
    
    // Prepare context
    let context: Vec<String> = search_results
        .iter()
        .map(|result| result.entry.content.clone())
        .collect();
    
    let context_text = context.join("\n\n");
    
    // Generate answer using RAG
    let system_prompt = "You are an AI assistant that answers questions based on provided document context. Use only the information from the context to answer questions.";
    
    let messages = vec![
        the_agency::llm::system_message(system_prompt),
        user_message(&format!("Context:\n{}\n\nQuestion: {}", context_text, question)),
    ];
    
    let response = llm_client.generate(&messages).await?;
    println!("Answer: {}", response.text);
    
    Ok(())
}

async fn extract_pdf_text(pdf_path: &Path) -> anyhow::Result<String> {
    // Use pdf-extract or similar library
    // This is a simplified example
    Ok("Sample PDF text content...".to_string())
}

fn split_into_sections(text: &str) -> Vec<String> {
    // Split document into logical sections
    text.split("\n\n")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
```

### Advanced PDF RAG with Table Extraction

Run the comprehensive PDF table extraction example:

```bash
# Install PDF processing dependencies
cargo build --features pdf

# Run the interactive PDF RAG example
cargo run --example pdf_rag_with_tables --features pdf
```

This example demonstrates:

- **Real PDF Text Extraction**: Uses `pdf-extract` library to parse actual PDF files
- **Table Detection & Parsing**: Identifies and structures tables from PDF content
- **Semantic Indexing**: Creates embeddings for sections, tables, and abstracts
- **Multi-modal Search**: Searches across text and tabular data
- **Interactive Q&A**: Ask questions about the document with context-aware answers

Features include:

- Automatic section detection and parsing
- Table structure recognition with headers and data rows
- Abstract and reference extraction
- Context-aware question answering
- Similarity-based content retrieval
- Support for academic papers and technical documents

### Run the Interactive Example

```bash
cargo run --bin agent-example
```

This starts an interactive chat session with:

- Real-time conversation
- Memory demonstrations
- Tool usage examples
- Statistics monitoring

## 🌐 REST API & Daemon

The Agency can run as a background daemon with a REST API for integration with other applications.

### Starting the Daemon

```bash
# Build the daemon
cargo build --release --bin agency-daemon

# Run the daemon
./target/release/agency-daemon

# Or in development mode
cargo run --bin agency-daemon
```

The daemon starts an HTTP server on `http://127.0.0.1:8080` with full OpenAPI documentation.

### API Examples

Run the interactive examples script to see the API in action:

```bash
# Run all examples (curl, Python, JavaScript)
./scripts/run-api-examples.sh

# Run specific language examples
./scripts/run-api-examples.sh --curl
./scripts/run-api-examples.sh --python
./scripts/run-api-examples.sh --javascript

# Keep daemon running after examples
./scripts/run-api-examples.sh --keep-running
```

#### cURL Examples

```bash
# Health check
curl http://127.0.0.1:8080/health

# Process a message
curl -X POST http://127.0.0.1:8080/api/v1/agent/process \
  -H "Content-Type: application/json" \
  -d '{"message": "What is Rust?", "max_steps": 5}'

# Create a workflow
curl -X POST http://127.0.0.1:8080/api/v1/workflows \
  -H "Content-Type: application/json" \
  -d '{
    "workflow_id": "my-workflow",
    "initial_message": "Process this task",
    "max_steps": 20
  }'

# List workflow snapshots
curl http://127.0.0.1:8080/api/v1/workflows/snapshots
```

#### Python Example

```python
import requests

class AgencyClient:
    def __init__(self, base_url="http://127.0.0.1:8080"):
        self.base_url = base_url
    
    def process(self, message, max_steps=None):
        data = {"message": message}
        if max_steps:
            data["max_steps"] = max_steps
        response = requests.post(
            f"{self.base_url}/api/v1/agent/process",
            json=data
        )
        return response.json()

# Usage
client = AgencyClient()
result = client.process("Tell me about Python")
print(result["response"])
```

#### JavaScript/Node.js Example

```javascript
class AgencyClient {
    constructor(baseUrl = 'http://127.0.0.1:8080') {
        this.baseUrl = baseUrl;
    }
    
    async process(message, maxSteps = null) {
        const body = { message };
        if (maxSteps) body.max_steps = maxSteps;
        
        const response = await fetch(
            `${this.baseUrl}/api/v1/agent/process`,
            {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(body)
            }
        );
        return response.json();
    }
}

// Usage
const client = new AgencyClient();
const result = await client.process('Tell me about JavaScript');
console.log(result.response);
```

### API Documentation

View the interactive Swagger UI documentation:

```bash
# Open the local Swagger UI
open docs/swagger-ui.html

# Or access the OpenAPI spec directly
curl http://127.0.0.1:8080/api-docs/openapi.json
```

See [API Documentation](docs/API_DOCUMENTATION.md) for complete API reference.

### System Service Installation

Run The Agency as a system service on Linux or Windows:

**Linux (systemd)**:
```bash
# Install as systemd service
sudo ./scripts/install-linux.sh

# Manage the service
sudo systemctl start the-agency
sudo systemctl status the-agency
sudo systemctl stop the-agency
```

**Windows Service**:
```powershell
# Install as Windows service (Run as Administrator)
.\scripts\install-windows.ps1 -Install

# Start the service
Start-Service "The Agency"

# Check status
Get-Service "The Agency"

# Uninstall
.\scripts\install-windows.ps1 -Uninstall
```

**Docker**:
```bash
# Build image
docker build -t the-agency .

# Run container
docker run -d -p 8080:8080 \
  -v $(pwd)/data:/app/data \
  --name the-agency \
  the-agency
```

**Kubernetes**:
```bash
# Deploy to Kubernetes
./scripts/deploy-k8s.sh

# Check deployment
kubectl -n the-agency get pods

# Access via ingress
curl https://the-agency.example.com/health
```

See [Deployment Guide](docs/DEPLOYMENT.md) for detailed deployment instructions.

## 🏗️ Architecture

### Core Components

```text
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────-┐
│    Agent        │    │  Workflow Engine │    │   LLM Client     │
│                 │────│                  │────│                  │
│ • Orchestration │    │ • Step execution │    │ • Text generation│
│ • Configuration │    │ • Decision logic │    │ • Embeddings     │
│ • State mgmt    │    │ • Tool calling   │    │ • Model mgmt     │
│ • A2A mgmt      │    │ • Multi-agent    │    │ • Load balancing │
└─────────────────┘    └──────────────────┘    └─────────────────-┘
          │                        │                        │
          │                        │                        │
          ▼                        ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Memory Store    │    │  MCP Client      │    │ Built-in Tools  │
│                 │    │                  │    │                 │
│ • Vector search │    │ • Server mgmt    │    │ • System info   │
│ • Embeddings    │    │ • Tool discovery │    │ • Extensible    │
│ • Persistence   │    │ • JSON-RPC calls │    │ • Async ready   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
          │                        │                        │
          │                        │                        │
          ▼                        ▼                        ▼
┌───────────────-──┐   ┌──────────────────┐    ┌─────────────────┐
│ Knowledge Mgmt   │   │  Saga Workflows  │    │ Unified Storage │
│                  │   │                  │    │                 │
│ • Learning       │   │ • Transactions   │    │ • Multi-backend │
│ • External ingest│   │ • Compensation   │    │ • Persistence   │
│ • Consolidation  │   │ • Rollback       │    │ • Resource mgmt │
└─────────────────-┘   └──────────────────┘    └─────────────────┘
                                    │
                                    ▼
                      ┌─────────────────────────────┐
                      │    A2A Communication        │
                      │                             │
                      │ • Multi-protocol support    │
                      │ • Service discovery         │
                      │ • Agent coordination        │
                      │ • Security & auth           │
                      │ • Load balancing            │
                      └─────────────────────────────┘
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
# LLM Provider Configuration (supports multiple providers)
[llm]
ollama_url = "http://localhost:11434"
text_model = "llama3.2"
embedding_model = "nomic-embed-text"
max_tokens = 4096
temperature = 0.7

# Multi-Provider Fallback (optional)
[llm.fallback]
enabled = true
strategy = "sequential"  # or "parallel"
max_retries = 2
retry_delay_ms = 1000
priority = ["ollama", "groq", "openai", "anthropic"]

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

[a2a]
# Agent identity
namespace = "ai_network"
name = "assistant"

# Service discovery
[a2a.discovery]
enabled = true
registry_type = "consul"
registry_url = "http://localhost:8500"
heartbeat_interval = "30s"
ttl = "90s"

# Protocol configuration
[a2a.protocols.http]
enabled = true
endpoint = "http://localhost:8080"
timeout = "30s"
retry_attempts = 3

# Security settings
[a2a.security]
enable_authentication = true
enable_encryption = true
api_key = "your-api-key"
rate_limit_per_minute = 100
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
use the_agency::{AgentBuilder, config::*};

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
use the_agency::config::{McpServerConfig};

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

## 🌐 Agent-to-Agent Communication

The A2A system enables sophisticated multi-agent architectures where specialized AI agents collaborate:

### Multi-Protocol Support

```rust
use the_agency::{a2a::*, AgentConfig};

// HTTP communication
let http_client = HttpA2AClient::new(A2AConfig {
    agent_id: AgentId::new("network", "coordinator"),
    protocols: HashMap::from([
        (ProtocolType::Http, ProtocolConfig {
            enabled: true,
            endpoint: "http://localhost:8080".to_string(),
            timeout: Duration::from_secs(30),
            retry_attempts: 3,
            connection_pool_size: 10,
            settings: HashMap::new(),
        })
    ]),
    ..Default::default()
})?;

// Start communication
http_client.start().await?;
```

### Service Discovery

```rust
// Register agent capabilities
let capabilities = AgentCapabilities {
    services: vec![
        "natural_language_processing".to_string(),
        "data_analysis".to_string(),
    ],
    protocols: vec!["http".to_string()],
    message_types: vec!["text".to_string(), "task".to_string()],
    metadata: HashMap::from([
        ("model".to_string(), "gpt-4".to_string()),
        ("max_tokens".to_string(), "4096".to_string()),
    ]),
};

client.register(capabilities).await?;

// Discover other agents
let data_agents = client.discover_agents("data_analysis").await?;
for agent in data_agents {
    println!("Found agent: {}", agent.agent_id.to_string());
    println!("Status: {:?}", agent.status);
}
```

### Message Patterns

```rust
// Request-Response pattern
let payload = MessagePayload::Task {
    task_id: "analysis-001".to_string(),
    operation: "analyze_sentiment".to_string(),
    parameters: HashMap::from([
        ("text".to_string(), "Customer feedback data".to_string()),
        ("language".to_string(), "en".to_string()),
    ]),
};

let response = client.request(target_agent, payload).await?;

// Event broadcasting
let event = MessagePayload::Event {
    event_type: "data_updated".to_string(),
    data: serde_json::json!({
        "dataset_id": "sales-2023",
        "changes": 1247
    }),
};

let agents = client.discover_agents("data_processor").await?;
let agent_ids: Vec<AgentId> = agents.into_iter().map(|a| a.agent_id).collect();
client.broadcast(agent_ids, event).await?;
```

### Multi-Agent Workflows

```rust
// Collaborative problem solving
async fn solve_complex_problem(query: &str) -> Result<String> {
    let coordinator = HttpA2AClient::new(config)?;
    coordinator.start().await?;
    
    // Discover specialist agents
    let math_agents = coordinator.discover_agents("mathematics").await?;
    let research_agents = coordinator.discover_agents("research").await?;
    
    let mut solutions = Vec::new();
    
    // Delegate to specialists based on query content
    if query.contains("calculate") {
        if let Some(math_agent) = math_agents.first() {
            let math_result = coordinator.request(
                math_agent.agent_id.clone(),
                MessagePayload::Query {
                    query_id: uuid::Uuid::new_v4().to_string(),
                    query_type: "mathematical_analysis".to_string(),
                    parameters: HashMap::from([
                        ("query".to_string(), query.to_string())
                    ]),
                }
            ).await?;
            solutions.push(format!("Math: {:?}", math_result.payload));
        }
    }
    
    if query.contains("research") {
        if let Some(research_agent) = research_agents.first() {
            let research_result = coordinator.request(
                research_agent.agent_id.clone(),
                MessagePayload::Query {
                    query_id: uuid::Uuid::new_v4().to_string(),
                    query_type: "factual_research".to_string(),
                    parameters: HashMap::from([
                        ("query".to_string(), query.to_string()),
                        ("sources".to_string(), "academic,web".to_string()),
                    ]),
                }
            ).await?;
            solutions.push(format!("Research: {:?}", research_result.payload));
        }
    }
    
    Ok(solutions.join("\n\n"))
}
```

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
use the_agency::memory::{MemoryStore, SqliteMemoryStore};

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
use the_agency::tools::BuiltinTools;
use the_agency::mcp::{ToolResult, ToolContent};

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

// A2A Communication statistics
if let Some(a2a_stats) = stats.a2a_stats {
    println!("A2A messages sent: {}", a2a_stats.messages_sent);
    println!("A2A messages received: {}", a2a_stats.messages_received);
    println!("Connected agents: {}", a2a_stats.connected_agents);
    println!("Failed communications: {}", a2a_stats.failed_messages);
    println!("Active protocols: {:?}", a2a_stats.active_protocols);
}
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

### State Management

```rust
// Pause agent execution and save state
agent.pause().await?;
let checkpoint = agent.save_checkpoint().await?;
println!("Agent paused, checkpoint: {}", checkpoint.id);

// Resume from checkpoint later
let mut restored_agent = Agent::restore_from_checkpoint(checkpoint).await?;
restored_agent.resume().await?;

// Continue conversation with full context preserved
let response = restored_agent.process("What were we discussing?").await?;
```

### Multi-Agent Collaboration

```rust
// Create specialized agents
let coordinator = create_agent("coordinator", vec!["orchestration"]).await?;
let analyst = create_agent("data_analyst", vec!["data_analysis"]).await?;
let researcher = create_agent("researcher", vec!["web_research"]).await?;

// Start A2A communication
coordinator.start_a2a().await?;
analyst.start_a2a().await?;
researcher.start_a2a().await?;

// Coordinator delegates tasks
let analysis_task = "Analyze Q4 sales trends";
let research_task = "Research market conditions in Q4";

// Parallel execution
let (analysis_result, research_result) = tokio::join!(
    coordinator.send_to_agent(analyst.agent_id(), analysis_task),
    coordinator.send_to_agent(researcher.agent_id(), research_task)
);

// Combine results
let final_report = format!(
    "Analysis: {}\n\nResearch: {}", 
    analysis_result?, 
    research_result?
);
```

## ⚙️ Advanced Usage

### Custom Workflow Steps

```rust
use the_agency::workflow::{WorkflowStep, WorkflowDecision, WorkflowContext};
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
use the_agency::memory::{MemoryStore, MemoryEntry, SearchResult};

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
use the_agency::llm::{LlmClient, Message, GenerationResponse, EmbeddingResponse};

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

**We welcome contributions!** Before contributing, please:

1. **Contact us first**: Email [rboddipalli@turingworks.com](mailto:rboddipalli@turingworks.com) to discuss your ideas
2. **Read the guide**: See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for detailed guidelines
3. **Follow the process**: We'll help you through the development and review process

### Quick Start for Contributors

```bash
# Clone repository
git clone https://github.com/ravituringworks/the-agency.git
cd the-agency

# Install dependencies and run tests
cargo build
cargo test

# Format and lint code
cargo fmt
cargo clippy
```

Areas we're especially looking for help:

- Document RAG enhancements
- Performance optimizations  
- Real-world integration examples
- Documentation and tutorials

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Copyright © 2025 Ravindra Boddipalli / [Turing Works](https://turingworks.com)

## 🙏 Acknowledgments

- [Ollama](https://ollama.ai/) - Local LLM inference
- [OpenAI](https://openai.com/) - GPT models
- [Anthropic](https://anthropic.com/) - Claude models
- [Google AI](https://ai.google.dev/) - Gemini models
- [Groq](https://groq.com/) - Fast LPU inference
- [Together AI](https://together.ai/) - Open source model hosting
- [MCP](https://modelcontextprotocol.io/) - Model Context Protocol
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [Tokio](https://tokio.rs/) - Async runtime
- [Serde](https://serde.rs/) - Serialization framework

## 📞 Support & Contact

### 📬 Primary Contact

- **Maintainer**: Ravindra Boddipalli
- **Email**: [rboddipalli@turingworks.com](mailto:rboddipalli@turingworks.com)
- **Company**: [Turing Works](https://turingworks.com)

### 📚 Documentation & Resources

- 📜 [API Documentation](https://docs.rs/the-agency)
- 🤖 [Multi-Provider LLM Architecture](docs/MULTI_PROVIDER_ARCHITECTURE.md)
- 🌐 [A2A Communication Guide](docs/A2A_COMMUNICATION.md)
- 🔄 [State Management Guide](docs/PAUSE_EXECUTION.md)
- 🗄️ [Unified Storage Guide](docs/UNIFIED_STORAGE_README.md)
- 📋 [API Reference](docs/API.md)
- ⏯️ [Suspend/Resume Guide](docs/SUSPEND_RESUME.md)
- 🏢 [Multi-Agent Organization Example](docs/ORGANIZATION.md)
- 🧠 [Knowledge Management Guide](docs/KNOWLEDGE_MANAGEMENT_SUMMARY.md)
- 🌐 [External Knowledge Learning Example](docs/EXTERNAL_KNOWLEDGE_LEARNING.md)
- ⚡ [Saga Workflows Guide](docs/SAGA_WORKFLOW.md)
- 🤝 [Collaborative Workspaces Example](docs/COLLABORATIVE_WORKSPACE.md)
- 📄 [Document RAG Examples](examples/pdf_rag_with_tables.rs)
- 🔌 [Multi-Provider Usage Example](examples/multi_provider_usage.rs)
- 🤖 [Robotics Scientist Agent](examples/robotics_research_engineer_example.rs)
- 🏢 [Multi-Agent Organization Example](examples/robotech_industries_organization_example.rs)
- 🤝 [Collaborative Workspaces Example](examples/collaborative_robotics_workspace.rs)
- 📚 [Knowledge Management](examples/rag_system_comprehensive.rs)
- ⚡ [Saga Workflows](examples/saga_workflow.rs)
- 🤖 [Saga LLM Workflows](examples/saga_llm_workflow.rs)
- 🔄 [Multi-Provider LLM Usage](examples/multi_provider_example.rs)
- 🌐 [Agent-to-Agent Communication](examples/a2a_communication.rs)
- 📄 [PDF RAG with Tables](examples/pdf_rag_with_tables.rs)
- 🗃️ [Unified Storage System](examples/unified_storage_system.rs)

### 🐛 Issues & Discussions

- 🐛 [Report Issues](https://github.com/ravituringworks/the-agency/issues)
- 💬 [Community Discussions](https://github.com/ravituringworks/the-agency/discussions)
- 🚀 [Feature Requests](https://github.com/ravituringworks/the-agency/issues/new?template=feature_request.md)

---

**Built with ❤️ in Rust by [Turing Works](https://turingworks.com)**
