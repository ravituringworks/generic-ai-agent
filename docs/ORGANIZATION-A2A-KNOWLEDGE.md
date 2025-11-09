# Organization A2A and Knowledge Management Integration

## Overview

This document describes the enhanced multi-agent organization system that integrates:

1. **A2A (Agent-to-Agent) Protocol** using flume channels for high-performance in-process messaging
2. **Knowledge Management** for organizational learning and memory

## Architecture

### Components

#### 1. LocalA2AClient (`src/organization/a2a_local.rs`)

A high-performance in-memory A2A client implementation using flume channels:

```rust
pub struct LocalA2AClient {
    config: A2AConfig,
    agent_registry: Arc<RwLock<HashMap<AgentId, LocalAgentEndpoint>>>,
    message_handlers: Arc<RwLock<Vec<Arc<dyn MessageHandler>>>>,
    stats: Arc<tokio::sync::Mutex<A2AStats>>,
    broadcast_sender: Arc<broadcast::Sender<A2AMessage>>,
}
```

**Features:**

- Bounded flume channels for each agent (default capacity: 100 messages)
- Support for all A2A message types (Request, Response, Event, Command, Query, Notification)
- Broadcast capability for multi-agent coordination
- Agent discovery and registration
- Message prioritization
- Statistics tracking

**Usage:**

```rust
let config = A2AConfig::default();
let client = LocalA2AClient::new(config)?;

// Register an agent with dedicated channel
let agent_id = AgentId::new("org", "alice");
let capabilities = AgentCapabilities {
    services: vec!["research".to_string()],
    protocols: vec!["local".to_string()],
    message_types: vec!["task".to_string()],
    metadata: HashMap::new(),
};

let receiver = client.register_agent_with_channel(
    agent_id.clone(),
    capabilities,
    100  // channel capacity
).await?;

// Send messages
let payload = MessagePayload::Task {
    task_id: "task-123".to_string(),
    operation: "process".to_string(),
    parameters: HashMap::from([
        ("priority".to_string(), "high".to_string()),
    ]),
};

client.notify(agent_id, payload).await?;

// Receive messages
while let Ok(message) = receiver.recv_async().await {
    // Process message
}
```

#### 2. Enhanced AgentCoordinator

The coordinator now uses A2A for all inter-agent communication:

```rust
pub struct AgentCoordinator {
    organization: Arc<RwLock<Organization>>,
    active_agents: Arc<RwLock<HashMap<String, Agent>>>,
    a2a_client: Arc<LocalA2AClient>,
    agent_id_map: Arc<RwLock<HashMap<String, AgentId>>>,
    knowledge_manager: Option<Arc<AdaptiveKnowledgeManager>>,
}
```

**Key Changes:**

- Replaced simple message queue with A2A protocol
- Added agent ID mapping (organization ID → A2A ID)
- Integrated knowledge manager for learning
- All messages now flow through A2A channels

#### 3. Knowledge Management Integration

Agents now:

1. **Query knowledge before tasks** - Retrieve relevant past experiences
2. **Store learnings after completion** - Document outcomes and insights
3. **Use consolidation features** - Merge similar knowledge
4. **Manage knowledge lifecycle** - Prune low-value memories

### Message Flow

```
┌─────────────┐
│ Coordinator │
└──────┬──────┘
       │ 1. spawn_agent()
       ▼
┌─────────────────────┐
│  LocalA2AClient     │
│  ┌───────────────┐  │
│  │ Agent Registry│  │
│  └───────────────┘  │
│  ┌───────────────┐  │
│  │ Flume Channels│  │
│  └───────────────┘  │
└──────┬──────────────┘
       │ 2. register_agent_with_channel()
       │
       ▼
┌─────────────────┐
│  Agent A        │
│  Receiver<Msg>  │
└─────────────────┘

Task Assignment Flow:
1. Coordinator.assign_task(agent_id, task)
2. Coordinator.send_message() → A2A notify()
3. Flume channel delivers message
4. Agent receives via receiver.recv_async()
5. Agent processes task with knowledge context
6. Agent stores learnings
7. Coordinator updates task status
```

## Knowledge Management Workflow

### Before Task Execution

```rust
// 1. Query organizational memory for similar tasks
let similar_tasks = knowledge_manager
    .query_knowledge(role, &task.description)
    .await?;

// 2. Extract best practices
let best_practices = similar_tasks
    .iter()
    .filter(|k| k.quality_score > 0.8)
    .collect();

// 3. Build context prompt
let context_prompt = format!(
    "Past Experiences:\n{}\n\nCurrent Task:\n{}",
    format_experiences(&best_practices),
    task.description
);
```

### During Task Execution

```rust
// Agent processes with organizational context
let result = agent.process(&context_prompt).await?;
```

### After Task Completion

```rust
// 1. Extract learnings from result
let learning = KnowledgeEntry {
    content: format!("Task: {}\nApproach: {}\nOutcome: {}",
        task.title, result.approach, result.outcome),
    metadata: HashMap::from([
        ("role".to_string(), role.to_string()),
        ("task_type".to_string(), task.category.to_string()),
        ("quality_score".to_string(), calculate_quality(&result).to_string()),
        ("reuse_count".to_string(), "0".to_string()),
        ("timestamp".to_string(), Utc::now().to_rfc3339()),
    ]),
};

// 2. Store in knowledge base
knowledge_manager.store_learning(role, learning).await?;

// 3. Check if consolidation needed
if knowledge_manager.needs_consolidation(role).await? {
    knowledge_manager.consolidate_knowledge(role).await?;
}
```

## Message Types

### TaskAssignment

```rust
AgentMessage::TaskAssignment {
    task_id: String,
    task: WorkspaceTask,
    from_agent: String,
}

```

Converted to A2A:

```rust
MessagePayload::Task {
    task_id,
    operation: "assign",
    parameters: {
        "title": task.title,
        "description": task.description,
        "priority": task.priority.to_string(),
    }
}
```

### Collaboration

```rust
AgentMessage::Collaboration {
    workspace_id: String,
    message: String,
    from_agent: String,
}
```

Converted to A2A:

```rust
MessagePayload::Event {
    event_type: "collaboration",
    data: {
        "workspace_id": workspace_id,
        "message": message,
        "from": from_agent,
    }
}
```

### StatusUpdate

```rust
AgentMessage::StatusUpdate {
    agent_id: String,
    status: AgentStatus,
    current_tasks: Vec<String>,
}
```

### Question/Answer

For inter-agent queries and knowledge sharing.

## Performance Characteristics

### Flume Channels

- **Bounded channels** prevent memory exhaustion
- **MPMC** (Multi-Producer Multi-Consumer) support
- **Lock-free** for high throughput
- **Async-aware** with tokio integration

### Typical Latencies

- Message send: < 1μs (in-memory)
- Channel capacity: 100 messages (configurable)
- Broadcast: O(n) where n = number of subscribers

## Configuration

### Basic Setup

```rust
let coordinator = AgentCoordinator::new(organization);
```

### With Knowledge Management

```rust
let learning_config = LearningConfig {
    soft_limit_best_practices: 1000,
    hard_limit_best_practices: 5000,
    min_quality_score_to_keep: 0.7,
    min_reuse_count_to_keep: 2,
    max_age_days_if_unused: 90,
    enable_auto_consolidation: true,
};

let knowledge_manager = AdaptiveKnowledgeManager::new(learning_config);
let coordinator = AgentCoordinator::new(organization)
    .with_knowledge_manager(knowledge_manager);
```

### Agent Spawn

```rust
// Create agent config with memory enabled
let mut config = AgentConfig::default();
config.agent.use_memory = true;
config.agent.use_tools = true;
config.agent.system_prompt = role.system_prompt();  // includes learning instructions

coordinator.spawn_agent(agent_id, config).await?;
```

## Example: Complete Flow

```rust
use the_agency::{
    Organization, OrganizationAgent, OrganizationRole,
    AgentCoordinator, AgentConfig,
    AdaptiveKnowledgeManager, LearningConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Create organization
    let mut org = Organization::new("RoboTech".to_string());
    
    let alice = OrganizationAgent::new(
        "Alice Chen".to_string(),
        OrganizationRole::ResearchEngineerScaling,
    );
    org.add_agent(alice);
    
    // 2. Setup knowledge management
    let learning_config = LearningConfig::default();
    let knowledge_manager = AdaptiveKnowledgeManager::new(learning_config);
    
    // 3. Create coordinator with A2A and knowledge
    let coordinator = AgentCoordinator::new(org)
        .with_knowledge_manager(knowledge_manager);
    
    // 4. Spawn agents (registers with A2A automatically)
    for (agent_id, agent) in &coordinator.get_organization().await.agents {
        let mut config = AgentConfig::default();
        config.agent.use_memory = true;
        config.agent.system_prompt = agent.role.system_prompt();
        
        coordinator.spawn_agent(agent_id.clone(), config).await?;
    }
    
    // 5. Execute tasks (with knowledge integration)
    let task = WorkspaceTask::new(
        "Optimize ML Training".to_string(),
        "Improve distributed training efficiency".to_string(),
        vec![alice_id.clone()],
    );
    
    coordinator.assign_task(&alice_id, &workspace_id, task).await?;
    
    Ok(())
}
```

## Benefits

### A2A Integration

✅ **Type-safe messaging** - Compile-time verification
✅ **High performance** - Lock-free channels, < 1μs latency
✅ **Scalable** - Support for hundreds of agents
✅ **Flexible** - Multiple message types and patterns
✅ **Observable** - Built-in stats and tracing

### Knowledge Management

✅ **Organizational learning** - Agents improve over time
✅ **Avoid repeated mistakes** - Learn from past failures
✅ **Best practice propagation** - Share successful approaches
✅ **Context-aware execution** - Leverage relevant past work
✅ **Adaptive memory** - Automatic pruning and consolidation

## Future Enhancements

### Planned Features

- [ ] Distributed A2A using Redis/RabbitMQ for multi-process
- [ ] Advanced routing strategies (load balancing, circuit breakers)
- [ ] Knowledge recommendation engine
- [ ] Cross-workspace knowledge sharing
- [ ] Learning impact metrics dashboard
- [ ] Automatic pattern detection from agent work
- [ ] Expert agent identification and consultation

### Integration Opportunities

- [ ] Connect to external agent systems via A2A HTTP/WebSocket
- [ ] Persistent knowledge store (PostgreSQL, Qdrant)
- [ ] Real-time knowledge consolidation pipeline
- [ ] Multi-organization knowledge federation

## Migration Guide

### From Old Message Queue

**Before:**

```rust
let message = AgentMessage::TaskAssignment { ... };
coordinator.send_message(agent_id, message).await;
coordinator.process_messages().await?;
```

**After:**

```rust
let message = AgentMessage::TaskAssignment { ... };
coordinator.send_message(agent_id, message).await?;
// Messages are delivered immediately via A2A channels
// No need to call process_messages()
```

### Adding Knowledge Management

1. Enable memory in agent config:

```rust
config.agent.use_memory = true;
```

2. Use role-specific prompts (includes learning instructions):

```rust
config.agent.system_prompt = role.system_prompt();
```

3. Create coordinator with knowledge manager:

```rust
let coordinator = AgentCoordinator::new(org)
    .with_knowledge_manager(knowledge_manager);
```

## Testing

```rust
#[tokio::test]
async fn test_a2a_messaging() {
    let config = A2AConfig::default();
    let client = LocalA2AClient::new(config).unwrap();
    
    let agent_id = AgentId::new("test", "agent1");
    let capabilities = AgentCapabilities { ... };
    
    let rx = client
        .register_agent_with_channel(agent_id.clone(), capabilities, 10)
        .await
        .unwrap();
    
    let payload = MessagePayload::Text {
        content: "Test message".to_string(),
    };
    
    client.notify(agent_id, payload).await.unwrap();
    
    let received = rx.recv_async().await.unwrap();
    assert_eq!(received.message_type, MessageType::Notification);
}
```

## References

- [A2A Communication Documentation](A2A_COMMUNICATION.md)
- [Organizational Learning](ORGANIZATIONAL-LEARNING.md)
- [Knowledge Management API](../src/knowledge/manager.rs)
- [Flume Documentation](https://docs.rs/flume)
