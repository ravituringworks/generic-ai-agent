//! Example demonstrating the datetime and location identification tools
//! 
//! This example shows how to use the built-in datetime and location tools
//! to get comprehensive time and location information from the system.

use the-agency::{
    Agent, AgentConfig,
    tools::{BuiltinTools, execute_datetime_info, execute_location_info, execute_system_info}
};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🕐 DateTime and Location Tools Demo");
    println!("=====================================");
    println!();

    // Demo 1: Direct tool execution
    println!("📍 Demo 1: Direct Tool Execution");
    println!("---------------------------------");
    
    println!("🔧 Executing system_info tool...");
    let system_result = execute_system_info().await;
    if let Some(content) = system_result.content.first() {
        match content {
            the-agency::mcp::ToolContent::Text { text } => {
                println!("✅ {}", text);
            }
            _ => println!("❌ Unexpected content format"),
        }
    }
    println!();

    println!("🕐 Executing datetime_info tool...");
    let datetime_result = execute_datetime_info().await;
    if let Some(content) = datetime_result.content.first() {
        match content {
            the-agency::mcp::ToolContent::Text { text } => {
                println!("✅ {}", text);
            }
            _ => println!("❌ Unexpected content format"),
        }
    }
    println!();

    println!("📍 Executing location_info tool...");
    let location_result = execute_location_info().await;
    if let Some(content) = location_result.content.first() {
        match content {
            the-agency::mcp::ToolContent::Text { text } => {
                println!("✅ {}", text);
            }
            _ => println!("❌ Unexpected content format"),
        }
    }
    println!();

    // Demo 2: Using BuiltinTools registry
    println!("📚 Demo 2: Using BuiltinTools Registry");
    println!("--------------------------------------");
    
    let tools = BuiltinTools::new();
    let available_tools = tools.list_tools();
    
    println!("🛠️  Available tools: {:?}", available_tools);
    println!();
    
    for tool_name in &available_tools {
        println!("🔧 Executing tool: {}", tool_name);
        if let Some(result) = tools.execute(tool_name).await {
            if let Some(content) = result.content.first() {
                match content {
                    the-agency::mcp::ToolContent::Text { text } => {
                        // For display, truncate very long output
                        if text.len() > 500 {
                            println!("✅ {}... (truncated)", &text[..500]);
                        } else {
                            println!("✅ {}", text);
                        }
                    }
                    _ => println!("❌ Unexpected content format"),
                }
            }
        } else {
            println!("❌ Tool execution failed");
        }
        println!();
    }

    // Demo 3: Agent integration
    println!("🤖 Demo 3: Agent Integration");
    println!("----------------------------");
    
    let config = AgentConfig::default();
    let agent = Agent::new(config).await?;
    
    let stats = agent.stats().await;
    println!("📊 Agent Statistics:");
    println!("   • Built-in tools available: {}", stats.builtin_tools_count);
    println!("   • MCP servers connected: {}", stats.mcp_stats.connected_servers);
    println!("   • Total MCP tools: {}", stats.mcp_stats.total_tools);
    println!("   • Memory entries: {}", stats.memory_stats.total_memories);
    println!();

    // Demo 4: Interactive mode
    println!("💬 Demo 4: Interactive Agent Conversation");
    println!("------------------------------------------");
    println!("Ask the agent about time and location information!");
    println!("Example queries:");
    println!("  • 'What time is it?'");
    println!("  • 'What's my current location?'");
    println!("  • 'Tell me about my system'");
    println!("  • 'What timezone am I in?'");
    println!("  • Type 'quit' to exit");
    println!();

    loop {
        print!("You: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!("👋 Goodbye!");
            break;
        }
        
        println!("🤖 Agent: Processing your request...");
        
        // For this demo, we'll simulate how the agent would use these tools
        // based on the query content
        if input.to_lowercase().contains("time") || input.to_lowercase().contains("date") {
            println!("🕐 Using datetime_info tool...");
            let result = execute_datetime_info().await;
            if let Some(content) = result.content.first() {
                match content {
                    the-agency::mcp::ToolContent::Text { text } => {
                        println!("🤖 Agent: Based on the datetime information:");
                        println!("{}", text);
                    }
                    _ => println!("🤖 Agent: Got datetime info but couldn't format it properly."),
                }
            }
        } else if input.to_lowercase().contains("location") || 
                  input.to_lowercase().contains("where") ||
                  input.to_lowercase().contains("timezone") {
            println!("📍 Using location_info tool...");
            let result = execute_location_info().await;
            if let Some(content) = result.content.first() {
                match content {
                    the-agency::mcp::ToolContent::Text { text } => {
                        println!("🤖 Agent: Based on the location information:");
                        println!("{}", text);
                    }
                    _ => println!("🤖 Agent: Got location info but couldn't format it properly."),
                }
            }
        } else if input.to_lowercase().contains("system") ||
                  input.to_lowercase().contains("computer") ||
                  input.to_lowercase().contains("platform") {
            println!("🔧 Using system_info tool...");
            let result = execute_system_info().await;
            if let Some(content) = result.content.first() {
                match content {
                    the-agency::mcp::ToolContent::Text { text } => {
                        println!("🤖 Agent: Here's your system information:");
                        println!("{}", text);
                    }
                    _ => println!("🤖 Agent: Got system info but couldn't format it properly."),
                }
            }
        } else {
            println!("🤖 Agent: I can help you with time, date, location, and system information.");
            println!("       Try asking about the current time, your location, or system details!");
        }
        println!();
    }

    // Demo 5: Concurrent execution
    println!("⚡ Demo 5: Concurrent Tool Execution");
    println!("-----------------------------------");
    println!("Executing all tools concurrently...");
    
    let start_time = std::time::Instant::now();
    
    let (system_result, datetime_result, location_result) = tokio::join!(
        execute_system_info(),
        execute_datetime_info(),
        execute_location_info()
    );
    
    let elapsed = start_time.elapsed();
    
    println!("⏱️  All tools completed in: {:?}", elapsed);
    println!("✅ System tool success: {}", !system_result.is_error);
    println!("✅ DateTime tool success: {}", !datetime_result.is_error);
    println!("✅ Location tool success: {}", !location_result.is_error);
    println!();
    
    println!("🎉 Demo completed successfully!");
    println!("The datetime and location tools are now available for your AI agent to use.");
    println!("These tools can be automatically called by the agent when users ask about:");
    println!("  • Current date and time");
    println!("  • Timezone information");
    println!("  • Location details");
    println!("  • System locale settings");

    Ok(())
}

/// Helper function to demonstrate tool availability check
pub fn check_tool_availability() {
    let tools = BuiltinTools::new();
    let available = tools.list_tools();
    
    println!("🔍 Tool Availability Check:");
    println!("   • system_info: {}", available.contains(&"system_info".to_string()));
    println!("   • datetime_info: {}", available.contains(&"datetime_info".to_string()));
    println!("   • location_info: {}", available.contains(&"location_info".to_string()));
}