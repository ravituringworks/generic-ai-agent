# Getting Started

This guide will walk you through the process of setting up your environment and running The Agency for the first time.

## Prerequisites

1. **Install Rust** (1.75 or later):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install and run Ollama**:

   ```bash
   # Install Ollama
   curl -fsSL https://ollama.ai/install.sh | sh
   
   # Pull required models
   ollama pull qwen3-coder:480b-cloud
   ollama pull nomic-embed-text
   ```

3. **Clone and build**:

   ```bash
   git clone https://github.com/ravituringworks/the-agency.git
   cd the-agency
   cargo build --release
   ```

## Basic Usage

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
