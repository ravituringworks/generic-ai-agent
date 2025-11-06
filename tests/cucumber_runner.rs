//! Cucumber BDD test runner for The Agency
//!
//! This file configures and runs all BDD tests for the framework.
//! Run with: `cargo test --test cucumber_runner`

use cucumber::World;

mod bdd_steps;
mod llm_provider_bdd;

#[tokio::main]
async fn main() {
    println!("🧪 Running BDD Tests for The Agency");
    println!("=====================================\n");

    // Run agent capabilities tests
    println!("📋 Running Agent Capabilities Tests...");
    bdd_steps::AgentWorld::cucumber()
        .max_concurrent_scenarios(1) // Run scenarios sequentially to avoid conflicts
        .run("features/agent_capabilities.feature")
        .await;

    println!("\n✅ Agent Capabilities Tests Complete\n");

    // Run LLM provider tests (if providers are configured)
    if std::env::var("OPENAI_API_KEY").is_ok()
        || std::env::var("ANTHROPIC_API_KEY").is_ok()
        || std::env::var("OLLAMA_RUNNING").is_ok()
    {
        println!("📋 Running LLM Provider Tests...");
        llm_provider_bdd::LlmProviderWorld::cucumber()
            .max_concurrent_scenarios(1)
            .run("features/multi_provider_llm.feature")
            .await;
        println!("\n✅ LLM Provider Tests Complete\n");
    } else {
        println!("⚠️  Skipping LLM provider tests - no API keys or Ollama configured\n");
    }

    println!("\n🎉 All BDD Tests Complete!");
}
