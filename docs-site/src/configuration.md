# Configuration

## Basic Configuration

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

## MCP Server Configuration

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

## Programmatic Configuration

```rust
use the_agency::{AgentBuilder, config::*};

let agent = AgentBuilder::new()
    .with_name("Custom Assistant".to_string())
    .with_system_prompt("You are an expert in Rust programming.".to_string())
    .with_ollama_url("http://custom-ollama:11434".to_string())
    .build()
    .await?;
```
