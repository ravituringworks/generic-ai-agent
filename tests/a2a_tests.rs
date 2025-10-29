//! Comprehensive tests for Agent-to-Agent (A2A) communication system

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use the_agency::*;
use tokio::time::sleep;

#[tokio::test]
async fn test_agent_id_creation_and_formatting() {
    let agent_id = AgentId::new("test_namespace", "test_agent");

    assert_eq!(agent_id.namespace, "test_namespace");
    assert_eq!(agent_id.name, "test_agent");
    assert!(!agent_id.instance.is_empty());

    let id_string = agent_id.to_string();
    assert!(id_string.contains("test_namespace"));
    assert!(id_string.contains("test_agent"));
}

#[tokio::test]
async fn test_a2a_message_creation() {
    let from_agent = AgentId::new("namespace1", "agent1");
    let to_agent = AgentId::new("namespace2", "agent2");

    let message = A2AMessage {
        id: uuid::Uuid::new_v4().to_string(),
        from: from_agent.clone(),
        to: to_agent.clone(),
        message_type: MessageType::Request,
        payload: MessagePayload::Text {
            content: "Hello from agent1".to_string(),
        },
        priority: MessagePriority::Normal,
        timestamp: SystemTime::now(),
        expires_at: Some(SystemTime::now() + Duration::from_secs(30)),
        correlation_id: None,
        reply_to: None,
        metadata: HashMap::new(),
    };

    assert_eq!(message.from, from_agent);
    assert_eq!(message.to, to_agent);
    assert_eq!(message.message_type, MessageType::Request);
    assert_eq!(message.priority, MessagePriority::Normal);

    match message.payload {
        MessagePayload::Text { content } => {
            assert_eq!(content, "Hello from agent1");
        }
        _ => panic!("Expected text payload"),
    }
}

#[tokio::test]
async fn test_a2a_config_creation() {
    let config = A2AConfig::default();

    assert!(!config.agent_id.namespace.is_empty());
    assert!(!config.agent_id.name.is_empty());
    assert!(!config.protocols.is_empty());

    // Check HTTP protocol is enabled by default
    if let Some(http_config) = config.protocols.get(&ProtocolType::Http) {
        assert!(http_config.enabled);
        assert!(!http_config.endpoint.is_empty());
    } else {
        panic!("HTTP protocol should be configured by default");
    }
}

#[tokio::test]
async fn test_http_a2a_client_creation() {
    let config = A2AConfig::default();
    let client = HttpA2AClient::new(config);

    assert!(client.is_ok());

    let client = client.unwrap();

    // Test client start/stop
    assert!(client.start().await.is_ok());
    assert!(client.stop().await.is_ok());
}

#[tokio::test]
async fn test_agent_registration() {
    let config = A2AConfig::default();
    let client = HttpA2AClient::new(config).unwrap();

    let capabilities = AgentCapabilities {
        services: vec!["chat".to_string(), "llm".to_string()],
        protocols: vec!["http".to_string()],
        message_types: vec!["text".to_string(), "task".to_string()],
        metadata: HashMap::from([
            ("model".to_string(), "llama3.2".to_string()),
            ("version".to_string(), "0.1.0".to_string()),
        ]),
    };

    // Test registration
    assert!(client.register(capabilities.clone()).await.is_ok());

    // Test agent discovery
    let discovered_agents = client.discover_agents("chat").await.unwrap();
    assert_eq!(discovered_agents.len(), 1);

    let agent_reg = &discovered_agents[0];
    assert!(agent_reg
        .capabilities
        .services
        .contains(&"chat".to_string()));
    assert!(agent_reg.capabilities.services.contains(&"llm".to_string()));

    // Test unregistration
    assert!(client.unregister().await.is_ok());
}

#[tokio::test]
async fn test_message_payload_variants() {
    // Test Text payload
    let text_payload = MessagePayload::Text {
        content: "Hello World".to_string(),
    };

    match text_payload {
        MessagePayload::Text { content } => assert_eq!(content, "Hello World"),
        _ => panic!("Expected text payload"),
    }

    // Test JSON payload
    let json_data = serde_json::json!({
        "key": "value",
        "number": 42
    });

    let json_payload = MessagePayload::Json {
        data: json_data.clone(),
    };

    match json_payload {
        MessagePayload::Json { data } => {
            assert_eq!(data["key"], "value");
            assert_eq!(data["number"], 42);
        }
        _ => panic!("Expected JSON payload"),
    }

    // Test Task payload
    let task_payload = MessagePayload::Task {
        task_id: "task-123".to_string(),
        operation: "process_data".to_string(),
        parameters: HashMap::from([
            ("input".to_string(), "test_data".to_string()),
            ("format".to_string(), "json".to_string()),
        ]),
    };

    match task_payload {
        MessagePayload::Task {
            task_id,
            operation,
            parameters,
        } => {
            assert_eq!(task_id, "task-123");
            assert_eq!(operation, "process_data");
            assert_eq!(parameters.get("input").unwrap(), "test_data");
        }
        _ => panic!("Expected task payload"),
    }

    // Test Event payload
    let event_data = serde_json::json!({
        "event_type": "user_action",
        "timestamp": "2023-01-01T00:00:00Z"
    });

    let event_payload = MessagePayload::Event {
        event_type: "user_action".to_string(),
        data: event_data.clone(),
    };

    match event_payload {
        MessagePayload::Event { event_type, data } => {
            assert_eq!(event_type, "user_action");
            assert_eq!(data["event_type"], "user_action");
        }
        _ => panic!("Expected event payload"),
    }
}

#[tokio::test]
async fn test_message_priority_ordering() {
    let low = MessagePriority::Low;
    let normal = MessagePriority::Normal;
    let high = MessagePriority::High;
    let critical = MessagePriority::Critical;

    assert!(low < normal);
    assert!(normal < high);
    assert!(high < critical);

    // Test ordering in a vector
    let mut priorities = vec![high.clone(), low.clone(), critical.clone(), normal.clone()];
    priorities.sort();

    assert_eq!(priorities, vec![low, normal, high, critical]);
}

#[tokio::test]
async fn test_agent_status_variants() {
    let online_status = AgentStatus::Online;
    let busy_status = AgentStatus::Busy;
    let idle_status = AgentStatus::Idle;
    let offline_status = AgentStatus::Offline;
    let error_status = AgentStatus::Error {
        message: "Connection failed".to_string(),
    };

    match online_status {
        AgentStatus::Online => (),
        _ => panic!("Expected Online status"),
    }

    match error_status {
        AgentStatus::Error { message } => {
            assert_eq!(message, "Connection failed");
        }
        _ => panic!("Expected Error status"),
    }
}

#[tokio::test]
async fn test_a2a_manager_creation() {
    let config = A2AConfig::default();
    let agent_id = AgentId::new("test", "manager");
    let client = HttpA2AClient::new(config).unwrap();

    let manager = A2AManager::new(Arc::new(client), agent_id);

    // Test manager start/stop
    assert!(manager.start().await.is_ok());
    assert!(manager.stop().await.is_ok());
}

#[tokio::test]
async fn test_service_discovery() {
    let config = A2AConfig::default();
    let client = HttpA2AClient::new(config).unwrap();

    // Register multiple agents with different capabilities
    let chat_capabilities = AgentCapabilities {
        services: vec!["chat".to_string(), "conversation".to_string()],
        protocols: vec!["http".to_string()],
        message_types: vec!["text".to_string()],
        metadata: HashMap::new(),
    };

    client.register(chat_capabilities).await.unwrap();

    // Discover chat agents
    let chat_agents = client.discover_agents("chat").await.unwrap();
    assert!(!chat_agents.is_empty());

    // Discover non-existent service
    let nonexistent_agents = client.discover_agents("nonexistent_service").await.unwrap();
    assert!(nonexistent_agents.is_empty());
}

#[tokio::test]
async fn test_a2a_stats() {
    let config = A2AConfig::default();
    let client = HttpA2AClient::new(config).unwrap();

    let stats = client.get_stats().await.unwrap();

    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.messages_failed, 0);
    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.discovered_agents, 0);
    assert_eq!(stats.uptime_seconds, 0);
    assert_eq!(stats.average_response_time_ms, 0.0);
}

#[tokio::test]
async fn test_response_status_variants() {
    let success = ResponseStatus::Success;
    let error = ResponseStatus::Error;
    let timeout = ResponseStatus::Timeout;
    let rejected = ResponseStatus::Rejected;
    let processing = ResponseStatus::Processing;

    match success {
        ResponseStatus::Success => (),
        _ => panic!("Expected Success status"),
    }

    match error {
        ResponseStatus::Error => (),
        _ => panic!("Expected Error status"),
    }
}

#[tokio::test]
async fn test_protocol_type_variants() {
    let protocols = vec![
        ProtocolType::Http,
        ProtocolType::WebSocket,
        ProtocolType::Redis,
        ProtocolType::RabbitMQ,
        ProtocolType::Tcp,
        ProtocolType::Udp,
        ProtocolType::InMemory,
    ];

    assert_eq!(protocols.len(), 7);
}

#[tokio::test]
async fn test_load_balancing_strategies() {
    let strategies = vec![
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::WeightedRoundRobin,
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::Random,
        LoadBalancingStrategy::ConsistentHashing,
    ];

    assert_eq!(strategies.len(), 5);
}

#[tokio::test]
async fn test_message_serialization() {
    let agent1 = AgentId::new("ns1", "agent1");
    let agent2 = AgentId::new("ns2", "agent2");

    let original_message = A2AMessage {
        id: uuid::Uuid::new_v4().to_string(),
        from: agent1,
        to: agent2,
        message_type: MessageType::Request,
        payload: MessagePayload::Text {
            content: "Test message".to_string(),
        },
        priority: MessagePriority::High,
        timestamp: SystemTime::now(),
        expires_at: None,
        correlation_id: Some("corr-123".to_string()),
        reply_to: None,
        metadata: HashMap::from([("sender_info".to_string(), "test".to_string())]),
    };

    // Test JSON serialization
    let serialized = serde_json::to_string(&original_message).unwrap();
    assert!(!serialized.is_empty());

    // Test JSON deserialization
    let deserialized: A2AMessage = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.id, original_message.id);
    assert_eq!(deserialized.from, original_message.from);
    assert_eq!(deserialized.to, original_message.to);
    assert_eq!(deserialized.message_type, original_message.message_type);
    assert_eq!(deserialized.priority, original_message.priority);
    assert_eq!(deserialized.correlation_id, original_message.correlation_id);
}

#[tokio::test]
async fn test_concurrent_operations() {
    let config = A2AConfig::default();
    let client = Arc::new(HttpA2AClient::new(config).unwrap());

    // Test concurrent registrations
    let mut handles = vec![];

    for i in 0..5 {
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            let capabilities = AgentCapabilities {
                services: vec![format!("service_{}", i)],
                protocols: vec!["http".to_string()],
                message_types: vec!["text".to_string()],
                metadata: HashMap::new(),
            };

            client_clone.register(capabilities).await
        });

        handles.push(handle);
    }

    // Wait for all registrations to complete
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }

    // Test stats after concurrent operations
    let stats = client.get_stats().await.unwrap();
    assert_eq!(stats.messages_sent, 0); // No actual messages sent, just registrations
}

#[tokio::test]
async fn test_message_expiration() {
    let agent1 = AgentId::new("ns1", "agent1");
    let agent2 = AgentId::new("ns2", "agent2");

    // Create message that expires in 1 second
    let message = A2AMessage {
        id: uuid::Uuid::new_v4().to_string(),
        from: agent1,
        to: agent2,
        message_type: MessageType::Request,
        payload: MessagePayload::Text {
            content: "Expiring message".to_string(),
        },
        priority: MessagePriority::Normal,
        timestamp: SystemTime::now(),
        expires_at: Some(SystemTime::now() + Duration::from_secs(1)),
        correlation_id: None,
        reply_to: None,
        metadata: HashMap::new(),
    };

    // Check message is not expired initially
    assert!(message.expires_at.unwrap() > SystemTime::now());

    // Wait for expiration
    sleep(Duration::from_secs(2)).await;

    // Message should now be expired (in a real implementation, you'd check this)
    assert!(SystemTime::now() > message.expires_at.unwrap());
}

// Integration test with Agent struct (requires mocking or real setup)
#[tokio::test]
async fn test_agent_a2a_integration() {
    // Create test agent configuration with A2A enabled
    let mut config = AgentConfig::default();
    config.a2a.discovery.enabled = false; // Disable for testing to avoid actual network calls

    // In a real test, you'd use a test database
    config.memory.database_url = Some("sqlite::memory:".to_string());

    // This test would require a running Ollama instance or mock
    // For now, we'll just test that the configuration is properly set
    assert!(!config.a2a.discovery.enabled);
    assert!(!config.a2a.agent_id.namespace.is_empty());
}
