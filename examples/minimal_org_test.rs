use anyhow::Result;
use the_agency::{Agent, AgentConfig};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Testing with single agent...");

    let mut config = AgentConfig::default();
    config.llm.ollama_url = "http://127.0.0.1:11434".to_string();
    config.memory.database_url = Some(":memory:".to_string());
    config.memory.persistent = false;
    config.agent.use_memory = false;
    config.agent.use_tools = false;
    config.agent.max_thinking_steps = 1;
    config.workflow.enable_suspend_resume = false;

    println!("Creating agent...");
    let mut agent = Agent::new(config).await?;

    println!("Processing request...");
    let response = agent.process("Say hello in 5 words").await?;

    println!("✅ SUCCESS! Response: {}", response);

    Ok(())
}
