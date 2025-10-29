//! Test loading configuration from config.toml

use the_agency::AgentConfig;

fn main() -> anyhow::Result<()> {
    println!("🔧 Testing configuration loading from config.toml...\n");

    // Load configuration
    let config = AgentConfig::from_file("config.toml")?;
    println!("✅ Configuration loaded successfully!\n");

    // Display key settings
    println!("📋 Configuration Summary:");
    println!("   ├─ LLM Model: {}", config.llm.text_model);
    println!("   ├─ Embedding Model: {}", config.llm.embedding_model);
    println!("   ├─ Cache Enabled: {}", config.llm.cache.enabled);
    println!("   ├─ Cache Max Entries: {}", config.llm.cache.max_entries);
    println!("   ├─ Cache TTL: {}s", config.llm.cache.ttl_seconds);
    println!("   ├─ Agent Name: {}", config.agent.name);
    println!(
        "   └─ Task Models: {} configured\n",
        config.llm.task_models.len()
    );

    // Validate configuration
    config.validate()?;
    println!("✅ Configuration is valid!\n");

    // Show task models
    if !config.llm.task_models.is_empty() {
        println!("🎯 Task-Specific Models:");
        for (task_name, task_config) in &config.llm.task_models {
            println!(
                "   ├─ {}: {} (temp: {:.2})",
                task_name,
                task_config.model,
                task_config.temperature.unwrap_or(config.llm.temperature)
            );
        }
        println!();
    }

    println!("✨ All checks passed! The configuration is ready to use.");

    Ok(())
}
