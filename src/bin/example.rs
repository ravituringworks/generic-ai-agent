//! Example application demonstrating the Generic AI Agent

use the-agency::{
    Agent, AgentBuilder, AgentConfig,
    config::{McpServerConfig, MemoryConfig},
};
use std::io::{self, Write};
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🤖 Generic AI Agent - Example Application");
    println!("==========================================");

    // Create agent configuration
    let mut config = AgentConfig::default();
    
    // Configure memory to use an in-memory database for this example
    // For persistent storage, use: Some("sqlite:/path/to/database.db".to_string())
    config.memory = MemoryConfig {
        // Use an in-memory SQLite database for this demo
        database_url: Some("sqlite::memory:".to_string()),
        embedding_dimension: 768,
        max_search_results: 5,
        similarity_threshold: 0.6, // slightly lower to make recall easier
        persistent: true,
        store_type: "sqlite".to_string(),
    };

    // Add example MCP server (if available)
    let mcp_server = McpServerConfig {
        transport: "http".to_string(),
        url: Some("http://localhost:8080/mcp".to_string()),
        command: None,
        env: None,
        timeout: Some(30),
        auth_token: None,
        enabled: false, // Disabled by default since server may not exist
    };
    config.add_mcp_server("example-server".to_string(), mcp_server);

    // Customize agent behavior
    config.agent.name = "Example AI Assistant".to_string();
    config.agent.system_prompt = r#"
You are a helpful AI assistant with access to tools and memory. You can:

1. Remember previous conversations and retrieve relevant context
2. Call tools to get system information or perform actions
3. Use your reasoning capabilities to help users

You are friendly, helpful, and always try to provide accurate information.
When you don't know something, you admit it rather than making things up.
"#.to_string();

    println!("Initializing agent...");

    // Create the agent
    let mut agent = match Agent::new(config).await {
        Ok(agent) => {
            println!("✅ Agent initialized successfully!");
            agent
        }
        Err(e) => {
            error!("❌ Failed to initialize agent: {}", e);
            eprintln!("Make sure Ollama is running with the required models:");
            eprintln!("  ollama pull llama3.2");
            eprintln!("  ollama pull nomic-embed-text");
            return Err(e.into());
        }
    };

    // Show agent statistics
    let stats = agent.stats().await;
    println!("\n📊 Agent Statistics:");
    println!("  • Conversation length: {}", stats.conversation_length);
    println!("  • Memory entries: {}", stats.memory_stats.total_memories);
    println!("  • Embedding dimension: {}", stats.memory_stats.embedding_dimension);
    println!("  • Connected MCP servers: {}", stats.mcp_stats.connected_servers);
    println!("  • Total tools available: {}", stats.mcp_stats.total_tools + stats.builtin_tools_count);
    println!("  • Built-in tools: {}", stats.builtin_tools_count);

    println!("\n🚀 Agent is ready! Type 'help' for commands or 'quit' to exit.\n");

    // Interactive loop
    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "quit" || input == "exit" {
            println!("👋 Goodbye!");
            break;
        }

        if input == "help" {
            print_help();
            continue;
        }

        if input == "stats" {
            let stats = agent.stats().await;
            print_stats(&stats);
            continue;
        }

        if input == "clear" {
            agent.clear_conversation();
            println!("🧹 Conversation history cleared.");
            continue;
        }

        if input.starts_with("demo ") {
            let demo_type = &input[5..];
            run_demo(&mut agent, demo_type).await?;
            continue;
        }

        // Process user input
        print!("🤔 Agent is thinking...\n");
        match agent.process(input).await {
            Ok(response) => {
                println!("🤖 Agent: {}\n", response);
            }
            Err(e) => {
                error!("❌ Error processing input: {}", e);
                println!("❌ Sorry, I encountered an error: {}\n", e);
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("\n📚 Available Commands:");
    println!("  • help           - Show this help message");
    println!("  • stats          - Show agent statistics");
    println!("  • clear          - Clear conversation history");
    println!("  • demo <type>    - Run a demo (memory, tools, workflow)");
    println!("  • quit/exit      - Exit the application");
    println!("\n💡 Example queries:");
    println!("  • \"What is my system information?\"");
    println!("  • \"Tell me about Rust programming\"");
    println!("  • \"Remember that I like coffee\"");
    println!("  • \"What did we talk about earlier?\"");
    println!();
}

fn print_stats(stats: &the-agency::agent::AgentStats) {
    println!("\n📊 Current Agent Statistics:");
    println!("┌─────────────────────────────────────────┐");
    println!("│ Conversation & Memory                   │");
    println!("├─────────────────────────────────────────┤");
    println!("│ Messages in conversation: {:>13} │", stats.conversation_length);
    println!("│ Total memories stored: {:>16} │", stats.memory_stats.total_memories);
    println!("│ Embedding dimension: {:>18} │", stats.memory_stats.embedding_dimension);
    println!("├─────────────────────────────────────────┤");
    println!("│ Tools & Capabilities                    │");
    println!("├─────────────────────────────────────────┤");
    println!("│ MCP servers connected: {:>16} │", stats.mcp_stats.connected_servers);
    println!("│ Total MCP tools: {:>22} │", stats.mcp_stats.total_tools);
    println!("│ Built-in tools: {:>23} │", stats.builtin_tools_count);
    println!("└─────────────────────────────────────────┘");
    
    if !stats.mcp_stats.servers.is_empty() {
        println!("\n🔧 MCP Servers:");
        for (name, tool_count) in &stats.mcp_stats.servers {
            println!("  • {}: {} tools", name, tool_count);
        }
    }
    println!();
}

async fn run_demo(agent: &mut Agent, demo_type: &str) -> anyhow::Result<()> {
    match demo_type {
        "memory" => {
            println!("\n🧠 Memory Demo");
            println!("==============");
            
            info!("Running memory demo");
            
            // Store some information
            let responses = vec![
                agent.process("Remember that my favorite programming language is Rust").await?,
                agent.process("I also enjoy working with Python for data science").await?,
                agent.process("My name is Alice and I work as a software engineer").await?,
            ];
            
            for response in responses {
                println!("🤖 {}", response);
            }
            
            println!("\nNow let's see if the agent can recall this information:");
            
            let recall_response = agent.process("What do you know about my preferences and background?").await?;
            println!("🤖 {}", recall_response);
        },
        
        "tools" => {
            println!("\n🔧 Tools Demo");
            println!("=============");
            
            info!("Running tools demo");
            
            let response = agent.process("Can you show me my system information?").await?;
            println!("🤖 {}", response);
        },
        
        "workflow" => {
            println!("\n⚡ Workflow Demo");
            println!("===============");
            
            info!("Running workflow demo");
            
            println!("This demo shows the agent's reasoning process:");
            
            let response = agent.process("I need to know about my system and also want you to remember that I'm interested in AI and machine learning").await?;
            println!("🤖 {}", response);
            
            println!("\nNow let's see if it can combine memory and tools:");
            let response2 = agent.process("Based on what you know about me and my system, what programming setup would you recommend?").await?;
            println!("🤖 {}", response2);
        },
        
        _ => {
            println!("❌ Unknown demo type. Available: memory, tools, workflow");
        }
    }
    
    println!();
    Ok(())
}

/// Example showing how to use the agent programmatically
#[allow(dead_code)]
async fn programmatic_example() -> anyhow::Result<()> {
    info!("Running programmatic example");

    // Create agent with custom configuration
    let agent = AgentBuilder::new()
        .with_name("Programmatic Assistant".to_string())
        .with_system_prompt("You are a helpful programming assistant specialized in Rust.".to_string())
        .build()
        .await?;

    // Example interaction
    let mut agent = agent;
    let response = agent.process("Explain the ownership system in Rust").await?;
    println!("Response: {}", response);

    // Get conversation history
    let conversation = agent.get_conversation();
    println!("Conversation has {} messages", conversation.len());

    Ok(())
}