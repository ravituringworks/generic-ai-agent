//! Agent-to-Agent (A2A) Communication Example
//!
//! This example demonstrates how to set up and use the A2A communication system
//! to enable agents to discover and communicate with each other.

use std::collections::HashMap;

use std::time::Duration;
use the_agency::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🤖 Agent-to-Agent Communication Example");
    println!("========================================");

    // Example 1: Basic A2A Setup
    println!("\n1. Basic A2A Client Setup");
    basic_a2a_setup().await?;

    // Example 2: Agent Registration and Discovery
    println!("\n2. Agent Registration and Service Discovery");
    agent_registration_example().await?;

    // Example 3: Message Exchange Patterns
    println!("\n3. Message Exchange Patterns");
    message_exchange_example().await?;

    // Example 4: Multi-Agent Collaboration
    println!("\n4. Multi-Agent Collaboration Scenario");
    multi_agent_collaboration().await?;

    // Example 5: Agent Integration with Full AI Agent
    println!("\n5. Integration with Full AI Agent");
    agent_integration_example().await?;

    Ok(())
}

/// Basic A2A client setup and configuration
async fn basic_a2a_setup() -> Result<()> {
    println!("Creating A2A configuration...");

    // Create custom A2A configuration
    let agent_id = AgentId::new("demo", "basic_agent");
    let mut a2a_config = A2AConfig {
        agent_id: agent_id.clone(),
        ..Default::default()
    };

    // Configure HTTP protocol
    if let Some(http_config) = a2a_config.protocols.get_mut(&ProtocolType::Http) {
        http_config.endpoint = "http://localhost:8080".to_string();
        http_config.timeout = Duration::from_secs(10);
        http_config.retry_attempts = 3;
    }

    // Enable service discovery
    a2a_config.discovery.enabled = true;
    a2a_config.discovery.heartbeat_interval = Duration::from_secs(30);

    println!("✓ Agent ID: {}", agent_id);
    println!("✓ HTTP endpoint: http://localhost:8080");
    println!("✓ Discovery enabled: {}", a2a_config.discovery.enabled);

    // Create A2A client
    let client = HttpA2AClient::new(a2a_config)?;
    println!("✓ A2A client created successfully");

    // Start the client
    client.start().await?;
    println!("✓ A2A client started");

    // Get initial stats
    let stats = client.get_stats().await?;
    println!(
        "✓ Initial stats: {} messages sent, {} failed",
        stats.messages_sent, stats.messages_failed
    );

    // Stop the client
    client.stop().await?;
    println!("✓ A2A client stopped");

    Ok(())
}

/// Agent registration and service discovery example
async fn agent_registration_example() -> Result<()> {
    println!("Setting up service discovery scenario...");

    // Create multiple agents with different capabilities
    let chat_agent = create_agent_with_capabilities(
        "demo",
        "chat_agent",
        vec![
            "chat".to_string(),
            "conversation".to_string(),
            "nlp".to_string(),
        ],
    )
    .await?;

    let task_agent = create_agent_with_capabilities(
        "demo",
        "task_agent",
        vec![
            "task_execution".to_string(),
            "workflow".to_string(),
            "automation".to_string(),
        ],
    )
    .await?;

    let data_agent = create_agent_with_capabilities(
        "demo",
        "data_agent",
        vec![
            "data_processing".to_string(),
            "analytics".to_string(),
            "reporting".to_string(),
        ],
    )
    .await?;

    println!("✓ Created 3 agents with different capabilities");

    // Discover agents by capability
    println!("\nDiscovering agents by capability:");

    let chat_agents = chat_agent.discover_agents("chat").await?;
    println!(
        "• Found {} agents with 'chat' capability",
        chat_agents.len()
    );

    let workflow_agents = task_agent.discover_agents("workflow").await?;
    println!(
        "• Found {} agents with 'workflow' capability",
        workflow_agents.len()
    );

    let analytics_agents = data_agent.discover_agents("analytics").await?;
    println!(
        "• Found {} agents with 'analytics' capability",
        analytics_agents.len()
    );

    // Show agent details
    if let Some(agent_reg) = chat_agents.first() {
        println!("\nChat Agent Details:");
        println!("• ID: {}", agent_reg.agent_id);
        println!("• Services: {:?}", agent_reg.capabilities.services);
        println!("• Status: {:?}", agent_reg.status);
        println!("• Endpoints: {:?}", agent_reg.endpoints);
    }

    Ok(())
}

/// Different message exchange patterns
async fn message_exchange_example() -> Result<()> {
    println!("Demonstrating message exchange patterns...");

    // Create two agents
    let sender_config = A2AConfig::default();
    let receiver_config = A2AConfig::default();

    let sender = HttpA2AClient::new(sender_config)?;
    let receiver = HttpA2AClient::new(receiver_config)?;

    sender.start().await?;
    receiver.start().await?;

    // Register both agents
    let sender_capabilities = AgentCapabilities {
        services: vec!["sender".to_string()],
        protocols: vec!["http".to_string()],
        message_types: vec!["text".to_string(), "task".to_string()],
        metadata: HashMap::new(),
    };

    let receiver_capabilities = AgentCapabilities {
        services: vec!["receiver".to_string(), "processor".to_string()],
        protocols: vec!["http".to_string()],
        message_types: vec!["text".to_string(), "task".to_string()],
        metadata: HashMap::new(),
    };

    sender.register(sender_capabilities).await?;
    receiver.register(receiver_capabilities).await?;

    println!("✓ Both agents registered");

    // Example 1: Simple Text Message
    println!("\n📝 Text Message Exchange:");
    let _text_payload = MessagePayload::Text {
        content: "Hello from sender agent!".to_string(),
    };

    // Find receiver agents
    let receiver_agents = sender.discover_agents("receiver").await?;
    if let Some(receiver_agent) = receiver_agents.first() {
        println!("• Sending text message to receiver...");
        // Note: In a real implementation, this would actually send the message
        println!("• Message: Hello from sender agent!");
        println!("• Target: {}", receiver_agent.agent_id);
    }

    // Example 2: Task Message
    println!("\n📋 Task Message Exchange:");
    let _task_payload = MessagePayload::Task {
        task_id: "task-001".to_string(),
        operation: "process_data".to_string(),
        parameters: HashMap::from([
            ("dataset".to_string(), "customer_data.csv".to_string()),
            ("output_format".to_string(), "json".to_string()),
            ("include_summary".to_string(), "true".to_string()),
        ]),
    };

    if let Some(_processor_agent) = receiver_agents.first() {
        println!("• Sending task to processor...");
        println!("• Task ID: task-001");
        println!("• Operation: process_data");
        println!("• Parameters: dataset=customer_data.csv, output_format=json");
    }

    // Example 3: Event Broadcasting
    println!("\n📡 Event Broadcasting:");
    let _event_payload = MessagePayload::Event {
        event_type: "system_update".to_string(),
        data: serde_json::json!({
            "version": "2.0.1",
            "changes": ["bug fixes", "performance improvements"],
            "restart_required": false
        }),
    };

    let all_agents = sender.discover_agents("receiver").await?;
    println!(
        "• Broadcasting system_update event to {} agents",
        all_agents.len()
    );

    // Example 4: Query Message
    println!("\n🔍 Query Message:");
    let _query_payload = MessagePayload::Query {
        query_id: "query-001".to_string(),
        query_type: "status_check".to_string(),
        parameters: HashMap::from([
            ("include_metrics".to_string(), "true".to_string()),
            ("detail_level".to_string(), "full".to_string()),
        ]),
    };

    println!("• Sending status query to agents...");
    println!("• Query ID: query-001");
    println!("• Type: status_check");

    sender.stop().await?;
    receiver.stop().await?;
    println!("✓ Message exchange examples completed");

    Ok(())
}

/// Multi-agent collaboration scenario
async fn multi_agent_collaboration() -> Result<()> {
    println!("Setting up multi-agent collaboration scenario...");

    // Scenario: Document Processing Pipeline
    // 1. Document Ingestion Agent - receives documents
    // 2. NLP Processing Agent - extracts text and metadata
    // 3. Analysis Agent - performs sentiment/topic analysis
    // 4. Storage Agent - saves results to database

    let agents = vec![
        ("ingestion", vec!["document_ingestion", "file_processing"]),
        ("nlp", vec!["text_extraction", "nlp", "metadata"]),
        (
            "analysis",
            vec!["sentiment_analysis", "topic_modeling", "classification"],
        ),
        ("storage", vec!["database", "persistence", "indexing"]),
    ];

    let mut agent_clients = Vec::new();

    // Create all agents
    for (name, services) in agents {
        let client = create_agent_with_capabilities(
            "pipeline",
            name,
            services.into_iter().map(String::from).collect(),
        )
        .await?;
        agent_clients.push((name, client));
        println!("✓ Created {} agent", name);
    }

    println!("\n🔄 Simulating Document Processing Pipeline:");

    // Step 1: Document arrives at ingestion agent
    println!("1. Document received by ingestion agent");
    let _document_data = serde_json::json!({
        "document_id": "doc-12345",
        "filename": "quarterly_report.pdf",
        "size_bytes": 2485760,
        "mime_type": "application/pdf",
        "metadata": {
            "author": "Finance Team",
            "created_date": "2023-12-01T10:00:00Z"
        }
    });

    // Step 2: Forward to NLP agent
    println!("2. Forwarding to NLP processing agent...");
    if let Some((_, ingestion_agent)) = agent_clients.iter().find(|(name, _)| *name == "ingestion")
    {
        let nlp_agents = ingestion_agent.discover_agents("nlp").await?;
        if !nlp_agents.is_empty() {
            println!("   • Found {} NLP agents available", nlp_agents.len());

            let _nlp_task = MessagePayload::Task {
                task_id: "nlp-12345".to_string(),
                operation: "extract_text_and_metadata".to_string(),
                parameters: HashMap::from([
                    ("document_id".to_string(), "doc-12345".to_string()),
                    ("extract_images".to_string(), "true".to_string()),
                    ("language_detection".to_string(), "auto".to_string()),
                ]),
            };

            println!("   • Task created: extract_text_and_metadata");
        }
    }

    // Step 3: Analysis phase
    println!("3. Text analysis processing...");
    let _analysis_task = MessagePayload::Task {
        task_id: "analysis-12345".to_string(),
        operation: "comprehensive_analysis".to_string(),
        parameters: HashMap::from([
            ("document_id".to_string(), "doc-12345".to_string()),
            ("sentiment_analysis".to_string(), "true".to_string()),
            ("topic_extraction".to_string(), "true".to_string()),
            ("entity_recognition".to_string(), "true".to_string()),
        ]),
    };

    println!("   • Analyzing sentiment, topics, and entities");

    // Step 4: Storage and indexing
    println!("4. Storing results and creating indexes...");
    let _storage_task = MessagePayload::Task {
        task_id: "storage-12345".to_string(),
        operation: "store_document_results".to_string(),
        parameters: HashMap::from([
            ("document_id".to_string(), "doc-12345".to_string()),
            ("create_search_index".to_string(), "true".to_string()),
            ("enable_full_text_search".to_string(), "true".to_string()),
        ]),
    };

    println!("   • Document indexed and searchable");

    // Step 5: Completion notification
    println!("5. Pipeline completed successfully! ✨");

    let _completion_event = MessagePayload::Event {
        event_type: "document_processing_complete".to_string(),
        data: serde_json::json!({
            "document_id": "doc-12345",
            "processing_time_ms": 5420,
            "pages_processed": 24,
            "text_length": 15678,
            "entities_found": 42,
            "sentiment_score": 0.75,
            "topics": ["financial performance", "quarterly results", "market analysis"]
        }),
    };

    println!("   • Broadcasting completion event to all agents");

    // Show pipeline statistics
    println!("\n📊 Pipeline Statistics:");
    for (name, client) in &agent_clients {
        let stats = client.get_stats().await?;
        println!(
            "• {} agent: {} messages sent, {} received",
            name, stats.messages_sent, stats.messages_received
        );
    }

    Ok(())
}

/// Integration with full AI Agent
async fn agent_integration_example() -> Result<()> {
    println!("Creating AI Agent with A2A capabilities...");

    // Create agent configuration with A2A enabled
    let mut config = AgentConfig::default();
    config.agent.name = "AI Assistant with A2A".to_string();
    config.agent.system_prompt =
        "You are an AI assistant that can communicate with other agents.".to_string();

    // Enable A2A communication
    config.a2a.discovery.enabled = true;
    config.a2a.agent_id = AgentId::new("ai_network", "assistant");

    // Use in-memory database for this example
    config.memory.database_url = Some("sqlite::memory:".to_string());

    println!("✓ Configuration created with A2A enabled");

    // Note: In a real scenario, you would create the agent like this:
    // let agent = Agent::new(config).await?;
    // But this requires Ollama to be running

    println!("✓ Agent would be initialized with:");
    println!("  • Agent ID: {}", config.a2a.agent_id);
    println!("  • Discovery enabled: {}", config.a2a.discovery.enabled);
    println!("  • Memory: In-memory SQLite");

    // Show A2A methods that would be available
    println!("\n🔗 Available A2A Methods:");
    println!("  • agent.start_a2a() - Start A2A communication");
    println!("  • agent.send_to_agent(target, message) - Send message to another agent");
    println!("  • agent.discover_agents(capability) - Find agents with specific capabilities");
    println!("  • agent.process_agent_task(task) - Process task from another agent");
    println!("  • agent.has_a2a() - Check if A2A is enabled");

    // Example usage scenarios
    println!("\n🎯 Example Usage Scenarios:");
    println!("1. Collaborative Problem Solving:");
    println!("   • AI Agent A receives complex query");
    println!("   • Discovers specialized agents (math, research, coding)");
    println!("   • Delegates subtasks to specialist agents");
    println!("   • Combines results into comprehensive answer");

    println!("\n2. Distributed Knowledge Network:");
    println!("   • Each agent specializes in specific domain");
    println!("   • Agents share knowledge through A2A messages");
    println!("   • Dynamic load balancing based on agent availability");
    println!("   • Fault tolerance through agent discovery");

    println!("\n3. Multi-Modal Processing:");
    println!("   • Text processing agent handles natural language");
    println!("   • Image processing agent handles visual content");
    println!("   • Audio processing agent handles speech/sound");
    println!("   • Coordination agent orchestrates the pipeline");

    Ok(())
}

/// Helper function to create an agent with specific capabilities
async fn create_agent_with_capabilities(
    namespace: &str,
    name: &str,
    services: Vec<String>,
) -> Result<HttpA2AClient> {
    let agent_id = AgentId::new(namespace, name);
    let config = A2AConfig {
        agent_id,
        ..Default::default()
    };

    let client = HttpA2AClient::new(config)?;

    let capabilities = AgentCapabilities {
        services,
        protocols: vec!["http".to_string()],
        message_types: vec!["text".to_string(), "task".to_string(), "event".to_string()],
        metadata: HashMap::from([
            ("created_by".to_string(), "example".to_string()),
            ("version".to_string(), "1.0.0".to_string()),
        ]),
    };

    client.start().await?;
    client.register(capabilities).await?;

    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example_functions() {
        // Test that our example functions don't panic
        assert!(basic_a2a_setup().await.is_ok());
        assert!(agent_registration_example().await.is_ok());
        assert!(message_exchange_example().await.is_ok());
    }
}
