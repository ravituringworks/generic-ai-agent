# Agent-to-Agent (A2A) Communication System

The A2A (Agent-to-Agent) communication system enables your AI agents to discover, connect, and communicate with external agents, creating powerful multi-agent networks and collaborative AI systems.

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Architecture](#architecture)
- [Getting Started](#getting-started)
- [Configuration](#configuration)
- [Message Types and Patterns](#message-types-and-patterns)
- [Service Discovery](#service-discovery)
- [Security](#security)
- [Examples](#examples)
- [API Reference](#api-reference)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

The A2A communication system extends the capabilities of individual AI agents by allowing them to:

- **Discover** other agents with specific capabilities
- **Communicate** through various protocols (HTTP, WebSocket, message queues)
- **Collaborate** on complex tasks requiring specialized knowledge
- **Share** resources and knowledge across agent networks
- **Scale** horizontally by distributing workload across multiple agents

## Key Features

### 🌐 Multi-Protocol Support
- **HTTP REST APIs** - Standard web-based communication
- **WebSockets** - Real-time bidirectional communication
- **Message Queues** - Redis, RabbitMQ for reliable async messaging
- **TCP/UDP** - Low-level network protocols
- **In-Memory** - Testing and development

### 🔍 Service Discovery
- **Capability-Based Discovery** - Find agents by their services
- **Health Monitoring** - Automatic heartbeat and status tracking
- **Load Balancing** - Distribute requests across available agents
- **Circuit Breakers** - Fault tolerance and recovery

### 📨 Message Patterns
- **Request-Response** - Synchronous communication
- **Publish-Subscribe** - Event-driven messaging  
- **Broadcasting** - One-to-many communication
- **Queuing** - Reliable message delivery

### 🔒 Security & Reliability
- **Authentication** - API keys and certificates
- **Encryption** - TLS/SSL support
- **Rate Limiting** - Prevent abuse and overload
- **Message TTL** - Automatic expiration handling

## Architecture

```
┌─────────────────┐    A2A Messages     ┌─────────────────┐
│   Agent A       │◄──────────────────►│   Agent B       │
│                 │                     │                 │
│  ┌───────────┐  │                     │  ┌───────────┐  │
│  │    LLM    │  │                     │  │    LLM    │  │
│  ├───────────┤  │                     │  ├───────────┤  │
│  │  Memory   │  │                     │  │  Memory   │  │
│  ├───────────┤  │                     │  ├───────────┤  │
│  │    MCP    │  │                     │  │    MCP    │  │
│  ├───────────┤  │                     │  ├───────────┤  │
│  │    A2A    │  │                     │  │    A2A    │  │
│  │ Manager   │  │                     │  │ Manager   │  │
│  └───────────┘  │                     │  └───────────┘  │
└─────────────────┘                     └─────────────────┘
        │                                        │
        └────────────────┐              ┌───────┘
                         │              │
                    ┌────▼──────────────▼────┐
                    │  Service Discovery     │
                    │     Registry          │
                    └───────────────────────┘
```

## Getting Started

### 1. Basic Setup

```rust
use the_agency::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Create A2A configuration
    let agent_id = AgentId::new("my_namespace", "my_agent");
    let mut a2a_config = A2AConfig::default();
    a2a_config.agent_id = agent_id;
    
    // Create A2A client
    let client = HttpA2AClient::new(a2a_config)?;
    
    // Start the client
    client.start().await?;
    
    // Register capabilities
    let capabilities = AgentCapabilities {
        services: vec!["chat".to_string(), "llm".to_string()],
        protocols: vec!["http".to_string()],
        message_types: vec!["text".to_string()],
        metadata: HashMap::new(),
    };
    
    client.register(capabilities).await?;
    
    println!("A2A client started and registered!");
    
    Ok(())
}
```

### 2. Agent Integration

```rust
// Create AI Agent with A2A enabled
let mut config = AgentConfig::default();
config.a2a.discovery.enabled = true;
config.a2a.agent_id = AgentId::new("ai_network", "assistant");

let agent = Agent::new(config).await?;

// Start A2A communication
agent.start_a2a().await?;

// Send message to another agent
let target_agent = AgentId::new("ai_network", "specialist");
let response = agent.send_to_agent(target_agent, "Hello, can you help with this task?").await?;

println!("Response: {}", response);
```

## Configuration

### A2A Configuration Structure

```rust
pub struct A2AConfig {
    pub agent_id: AgentId,
    pub protocols: HashMap<ProtocolType, ProtocolConfig>,
    pub discovery: ServiceDiscoveryConfig,
    pub security: SecurityConfig,
    pub routing: RoutingConfig,
}
```

### Protocol Configuration

```rust
// HTTP Configuration
let http_config = ProtocolConfig {
    enabled: true,
    endpoint: "http://localhost:8080".to_string(),
    timeout: Duration::from_secs(30),
    retry_attempts: 3,
    connection_pool_size: 10,
    settings: HashMap::new(),
};

// WebSocket Configuration  
let ws_config = ProtocolConfig {
    enabled: true,
    endpoint: "ws://localhost:8081/ws".to_string(),
    timeout: Duration::from_secs(60),
    retry_attempts: 5,
    connection_pool_size: 5,
    settings: HashMap::from([
        ("max_frame_size".to_string(), "1048576".to_string()),
        ("ping_interval".to_string(), "30".to_string()),
    ]),
};
```

### Service Discovery Configuration

```rust
let discovery_config = ServiceDiscoveryConfig {
    enabled: true,
    registry_type: "consul".to_string(),
    registry_url: "http://localhost:8500".to_string(),
    heartbeat_interval: Duration::from_secs(30),
    discovery_interval: Duration::from_secs(60),
    ttl: Duration::from_secs(90),
};
```

### Security Configuration

```rust
let security_config = SecurityConfig {
    enable_authentication: true,
    enable_encryption: true,
    api_key: Some("your-api-key".to_string()),
    certificate_path: Some("/path/to/cert.pem".to_string()),
    allowed_agents: Some(vec![
        AgentId::new("trusted", "agent1"),
        AgentId::new("trusted", "agent2"),
    ]),
    rate_limit: Some(RateLimitConfig {
        requests_per_minute: 100,
        burst_size: 20,
        enable_per_agent_limits: true,
    }),
};
```

## Message Types and Patterns

### Message Types

The A2A system supports various message types for different communication patterns:

```rust
pub enum MessageType {
    Request,        // Expects a response
    Response,       // Reply to a request
    Event,          // One-way notification
    Command,        // Action to be executed
    Query,          // Information request
    Notification,   // Status update
    Heartbeat,      // Keep-alive signal
    Acknowledgment, // Confirm receipt
}
```

### Message Payloads

```rust
pub enum MessagePayload {
    Text { content: String },
    Json { data: serde_json::Value },
    Binary { data: Vec<u8> },
    Task { 
        task_id: String,
        operation: String,
        parameters: HashMap<String, String>,
    },
    Query {
        query_id: String,
        query_type: String,
        parameters: HashMap<String, String>,
    },
    Event {
        event_type: String,
        data: serde_json::Value,
    },
    Status {
        status: AgentStatus,
        message: Option<String>,
    },
}
```

### Communication Patterns

#### 1. Request-Response Pattern

```rust
// Send a request and wait for response
let payload = MessagePayload::Task {
    task_id: "process-001".to_string(),
    operation: "analyze_data".to_string(),
    parameters: HashMap::from([
        ("dataset".to_string(), "sales_data.csv".to_string()),
        ("format".to_string(), "summary".to_string()),
    ]),
};

let response = client.request(target_agent, payload).await?;
match response.payload {
    Some(MessagePayload::Json { data }) => {
        println!("Analysis result: {}", data);
    }
    _ => println!("Unexpected response format"),
}
```

#### 2. Event Broadcasting

```rust
// Broadcast event to multiple agents
let agents = client.discover_agents("data_processor").await?;
let agent_ids: Vec<AgentId> = agents.into_iter()
    .map(|reg| reg.agent_id)
    .collect();

let event = MessagePayload::Event {
    event_type: "data_updated".to_string(),
    data: serde_json::json!({
        "dataset_id": "ds-12345",
        "timestamp": "2023-12-01T10:00:00Z",
        "changes": 1547
    }),
};

let responses = client.broadcast(agent_ids, event).await?;
println!("Notified {} agents", responses.len());
```

#### 3. One-Way Notifications

```rust
// Send notification without expecting response
let notification = MessagePayload::Status {
    status: AgentStatus::Busy,
    message: Some("Processing large dataset".to_string()),
};

client.notify(target_agent, notification).await?;
```

## Service Discovery

### Registering Agent Capabilities

```rust
let capabilities = AgentCapabilities {
    services: vec![
        "natural_language_processing".to_string(),
        "text_analysis".to_string(),
        "sentiment_analysis".to_string(),
    ],
    protocols: vec!["http".to_string(), "websocket".to_string()],
    message_types: vec![
        "text".to_string(),
        "task".to_string(),
        "query".to_string(),
    ],
    metadata: HashMap::from([
        ("model".to_string(), "gpt-4".to_string()),
        ("languages".to_string(), "en,es,fr,de".to_string()),
        ("max_tokens".to_string(), "4096".to_string()),
    ]),
};

client.register(capabilities).await?;
```

### Discovering Agents

```rust
// Find all agents with NLP capabilities
let nlp_agents = client.discover_agents("natural_language_processing").await?;

for agent_reg in nlp_agents {
    println!("Found NLP agent: {}", agent_reg.agent_id.to_string());
    println!("  Status: {:?}", agent_reg.status);
    println!("  Services: {:?}", agent_reg.capabilities.services);
    println!("  Endpoints: {:?}", agent_reg.endpoints);
    
    // Check metadata for specific capabilities
    if let Some(languages) = agent_reg.capabilities.metadata.get("languages") {
        println!("  Supported languages: {}", languages);
    }
}
```

### Health Monitoring

```rust
// Check if agent is still available
if let Some(agent_info) = client.get_agent_info(&target_agent).await? {
    match agent_info.status {
        AgentStatus::Online => println!("Agent is online and ready"),
        AgentStatus::Busy => println!("Agent is busy, try again later"),
        AgentStatus::Error { message } => println!("Agent error: {}", message),
        _ => println!("Agent status: {:?}", agent_info.status),
    }
}
```

## Security

### Authentication

```rust
// API Key authentication
let mut security_config = SecurityConfig::default();
security_config.enable_authentication = true;
security_config.api_key = Some("your-secret-api-key".to_string());

// Certificate-based authentication
security_config.certificate_path = Some("/path/to/client.pem".to_string());
```

### Access Control

```rust
// Restrict communication to specific agents
security_config.allowed_agents = Some(vec![
    AgentId::new("trusted_namespace", "agent1"),
    AgentId::new("trusted_namespace", "agent2"),
    AgentId::new("partner_org", "specialist"),
]);
```

### Rate Limiting

```rust
let rate_limit = RateLimitConfig {
    requests_per_minute: 60,        // 60 requests per minute
    burst_size: 10,                 // Allow bursts up to 10 requests
    enable_per_agent_limits: true,  // Apply limits per source agent
};

security_config.rate_limit = Some(rate_limit);
```

## Examples

### Multi-Agent Document Processing Pipeline

```rust
async fn document_processing_pipeline() -> Result<()> {
    // 1. Create specialized agents
    let ingestion_agent = create_specialized_agent("doc_ingestion", vec!["file_processing"]).await?;
    let ocr_agent = create_specialized_agent("ocr", vec!["text_extraction", "image_processing"]).await?;
    let nlp_agent = create_specialized_agent("nlp", vec!["text_analysis", "entity_extraction"]).await?;
    let storage_agent = create_specialized_agent("storage", vec!["database", "indexing"]).await?;
    
    // 2. Document arrives at ingestion
    let document_task = MessagePayload::Task {
        task_id: "doc-001".to_string(),
        operation: "process_document".to_string(),
        parameters: HashMap::from([
            ("file_path".to_string(), "/uploads/document.pdf".to_string()),
            ("priority".to_string(), "high".to_string()),
        ]),
    };
    
    // 3. Chain processing through agents
    let ocr_agents = ingestion_agent.discover_agents("text_extraction").await?;
    if let Some(ocr_agent_reg) = ocr_agents.first() {
        let ocr_response = ingestion_agent.send_message(A2AMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from: ingestion_agent.config.agent_id.clone(),
            to: ocr_agent_reg.agent_id.clone(),
            message_type: MessageType::Request,
            payload: document_task,
            priority: MessagePriority::High,
            timestamp: SystemTime::now(),
            expires_at: Some(SystemTime::now() + Duration::from_secs(300)),
            correlation_id: Some("doc-001".to_string()),
            reply_to: None,
            metadata: HashMap::new(),
        }).await?;
        
        println!("OCR processing initiated: {:?}", ocr_response.status);
    }
    
    // Continue pipeline...
    Ok(())
}
```

### Collaborative Problem Solving

```rust
async fn collaborative_problem_solving(query: &str) -> Result<String> {
    let coordinator = HttpA2AClient::new(A2AConfig::default())?;
    coordinator.start().await?;
    
    // Discover specialist agents
    let math_agents = coordinator.discover_agents("mathematics").await?;
    let research_agents = coordinator.discover_agents("research").await?;
    let coding_agents = coordinator.discover_agents("programming").await?;
    
    let mut responses = Vec::new();
    
    // Delegate to math specialist if query contains mathematical terms
    if query.contains("calculate") || query.contains("equation") {
        if let Some(math_agent) = math_agents.first() {
            let math_task = MessagePayload::Query {
                query_id: uuid::Uuid::new_v4().to_string(),
                query_type: "mathematical_analysis".to_string(),
                parameters: HashMap::from([
                    ("query".to_string(), query.to_string()),
                    ("precision".to_string(), "high".to_string()),
                ]),
            };
            
            let response = coordinator.request(math_agent.agent_id.clone(), math_task).await?;
            responses.push(format!("Math analysis: {:?}", response.payload));
        }
    }
    
    // Delegate to research specialist for factual queries
    if query.contains("research") || query.contains("facts") {
        if let Some(research_agent) = research_agents.first() {
            let research_task = MessagePayload::Query {
                query_id: uuid::Uuid::new_v4().to_string(),
                query_type: "factual_research".to_string(),
                parameters: HashMap::from([
                    ("query".to_string(), query.to_string()),
                    ("sources".to_string(), "academic,news,web".to_string()),
                ]),
            };
            
            let response = coordinator.request(research_agent.agent_id.clone(), research_task).await?;
            responses.push(format!("Research findings: {:?}", response.payload));
        }
    }
    
    // Combine all responses
    Ok(responses.join("\n\n"))
}
```

## API Reference

### A2AClient Trait

```rust
#[async_trait]
pub trait A2AClient: Send + Sync {
    async fn send_message(&self, message: A2AMessage) -> Result<A2AResponse>;
    async fn request(&self, to: AgentId, payload: MessagePayload) -> Result<A2AResponse>;
    async fn notify(&self, to: AgentId, payload: MessagePayload) -> Result<()>;
    async fn broadcast(&self, to_agents: Vec<AgentId>, payload: MessagePayload) -> Result<Vec<A2AResponse>>;
    async fn subscribe(&self, message_types: Vec<MessageType>) -> Result<broadcast::Receiver<A2AMessage>>;
    async fn register(&self, capabilities: AgentCapabilities) -> Result<()>;
    async fn unregister(&self) -> Result<()>;
    async fn discover_agents(&self, capability: &str) -> Result<Vec<AgentRegistration>>;
    async fn get_agent_info(&self, agent_id: &AgentId) -> Result<Option<AgentRegistration>>;
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn get_stats(&self) -> Result<A2AStats>;
}
```

### Agent A2A Methods

```rust
impl Agent {
    pub async fn start_a2a(&self) -> Result<()>;
    pub async fn stop_a2a(&self) -> Result<()>;
    pub async fn send_to_agent(&self, target_agent: AgentId, message: &str) -> Result<String>;
    pub async fn discover_agents(&self, capability: &str) -> Result<Vec<AgentRegistration>>;
    pub async fn process_agent_task(&mut self, task_description: &str) -> Result<String>;
    pub fn has_a2a(&self) -> bool;
}
```

## Best Practices

### 1. Agent Design

- **Single Responsibility**: Each agent should specialize in specific capabilities
- **Stateless Operations**: Design agents to handle requests independently
- **Graceful Degradation**: Handle cases where other agents are unavailable
- **Resource Management**: Monitor memory and CPU usage in multi-agent environments

### 2. Message Design

- **Clear Semantics**: Use descriptive message types and operation names
- **Idempotency**: Design operations to be safely retryable
- **Timeouts**: Always set appropriate expiration times for messages
- **Correlation IDs**: Use correlation IDs to track related messages

### 3. Error Handling

```rust
// Robust error handling
match client.send_message(message).await {
    Ok(response) => match response.status {
        ResponseStatus::Success => {
            // Process successful response
            handle_success_response(response).await?;
        }
        ResponseStatus::Error => {
            // Handle agent-level error
            log::warn!("Agent returned error: {:?}", response.error);
            fallback_handling().await?;
        }
        ResponseStatus::Timeout => {
            // Handle timeout - maybe retry with different agent
            log::warn!("Request timed out, trying fallback agent");
            try_fallback_agent().await?;
        }
    },
    Err(e) => {
        // Handle network/transport errors
        log::error!("Communication error: {}", e);
        return Err(e);
    }
}
```

### 4. Performance Optimization

- **Connection Pooling**: Reuse connections for multiple requests
- **Async Processing**: Use non-blocking operations for better throughput
- **Load Balancing**: Distribute requests across available agents
- **Caching**: Cache agent discovery results when appropriate

### 5. Security

- **Principle of Least Privilege**: Only grant necessary permissions
- **Regular Key Rotation**: Rotate API keys and certificates regularly  
- **Input Validation**: Validate all incoming messages
- **Audit Logging**: Log all inter-agent communications

## Troubleshooting

### Common Issues

#### Agent Discovery Fails

```rust
// Check if discovery service is running
if !config.discovery.enabled {
    log::error!("Service discovery is disabled");
    return Err(AgentError::Config("Discovery disabled".to_string()));
}

// Verify network connectivity
match client.get_stats().await {
    Ok(stats) => log::info!("Client stats: {:?}", stats),
    Err(e) => log::error!("Client communication error: {}", e),
}
```

#### Message Timeouts

```rust
// Increase timeout for slow operations
let mut config = A2AConfig::default();
if let Some(http_config) = config.protocols.get_mut(&ProtocolType::Http) {
    http_config.timeout = Duration::from_secs(120); // 2 minutes
    http_config.retry_attempts = 5;
}
```

#### Authentication Failures

```rust
// Verify credentials
if let Some(api_key) = &config.security.api_key {
    log::info!("Using API key: {}****", &api_key[..4]);
} else {
    log::warn!("No API key configured");
}

// Check allowed agents list
if let Some(allowed) = &config.security.allowed_agents {
    log::info!("Allowed agents: {:?}", allowed);
}
```

### Debugging Tools

```rust
// Enable verbose logging
config.debug = true;

// Monitor message flow
let stats = client.get_stats().await?;
println!("Messages sent: {}", stats.messages_sent);
println!("Messages received: {}", stats.messages_received);
println!("Failed messages: {}", stats.messages_failed);
println!("Active connections: {}", stats.active_connections);

// Check agent registrations
let discovered = client.discover_agents("").await?; // Discover all agents
for agent in discovered {
    println!("Agent: {} - Status: {:?}", agent.agent_id.to_string(), agent.status);
}
```

## Conclusion

The A2A communication system enables powerful multi-agent architectures where specialized AI agents can collaborate to solve complex problems. By leveraging service discovery, reliable messaging, and security features, you can build scalable and resilient agent networks that extend far beyond the capabilities of individual agents.

For more examples and advanced usage patterns, see the [examples directory](../examples/) and the comprehensive test suite in [tests/a2a_tests.rs](../tests/a2a_tests.rs).