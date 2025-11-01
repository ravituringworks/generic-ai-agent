//! Local A2A Client for In-Process Agent Communication
//!
//! This module provides a high-performance in-memory implementation of the A2A
//! protocol using flume channels for cross-thread messaging between agents.

use crate::a2a::*;
use crate::error::{AgentError, Result};
use async_trait::async_trait;
use flume::{bounded, Receiver, Sender};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Local A2A client for in-process communication using flume channels
pub struct LocalA2AClient {
    config: A2AConfig,
    agent_registry: Arc<RwLock<HashMap<AgentId, LocalAgentEndpoint>>>,
    message_handlers: Arc<RwLock<Vec<Arc<dyn MessageHandler>>>>,
    stats: Arc<tokio::sync::Mutex<A2AStats>>,
    broadcast_sender: Arc<broadcast::Sender<A2AMessage>>,
}

/// Local agent endpoint with flume channel
struct LocalAgentEndpoint {
    registration: AgentRegistration,
    message_sender: Sender<A2AMessage>,
    _message_receiver: Option<Receiver<A2AMessage>>,
}

impl LocalA2AClient {
    /// Create a new local A2A client
    pub fn new(config: A2AConfig) -> Result<Self> {
        let (broadcast_sender, _) = broadcast::channel(1000);

        Ok(Self {
            config,
            agent_registry: Arc::new(RwLock::new(HashMap::new())),
            message_handlers: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(tokio::sync::Mutex::new(A2AStats::default())),
            broadcast_sender: Arc::new(broadcast_sender),
        })
    }

    /// Add a message handler
    pub async fn add_message_handler(&self, handler: Arc<dyn MessageHandler>) {
        let mut handlers = self.message_handlers.write().await;
        handlers.push(handler);
    }

    /// Register an agent with a dedicated channel
    pub async fn register_agent_with_channel(
        &self,
        agent_id: AgentId,
        capabilities: AgentCapabilities,
        capacity: usize,
    ) -> Result<Receiver<A2AMessage>> {
        let (sender, receiver) = bounded::<A2AMessage>(capacity);

        let registration = AgentRegistration {
            agent_id: agent_id.clone(),
            capabilities,
            endpoints: HashMap::from([("local".to_string(), "inmemory".to_string())]),
            heartbeat_interval: Duration::from_secs(30),
            registered_at: SystemTime::now(),
            last_seen: SystemTime::now(),
            status: AgentStatus::Online,
        };

        let endpoint = LocalAgentEndpoint {
            registration,
            message_sender: sender,
            _message_receiver: None,
        };

        let mut registry = self.agent_registry.write().await;
        registry.insert(agent_id, endpoint);

        Ok(receiver)
    }

    /// Process incoming message through handlers
    #[allow(dead_code)]
    async fn process_message(&self, message: A2AMessage) -> Result<Option<MessagePayload>> {
        let handlers = self.message_handlers.read().await;

        for handler in handlers.iter() {
            match message.message_type {
                MessageType::Request => {
                    if let Ok(response) = handler.handle_request(message.clone()).await {
                        return Ok(Some(response));
                    }
                }
                MessageType::Event => {
                    let _ = handler.handle_event(message.clone()).await;
                }
                MessageType::Command => {
                    if let Ok(response) = handler.handle_command(message.clone()).await {
                        return Ok(Some(response));
                    }
                }
                _ => {
                    if let Ok(response) = handler.handle_message(message.clone()).await {
                        return Ok(response);
                    }
                }
            }
        }

        Ok(None)
    }
}

#[async_trait]
impl A2AClient for LocalA2AClient {
    async fn send_message(&self, message: A2AMessage) -> Result<A2AResponse> {
        let start_time = SystemTime::now();

        // Look up target agent
        let registry = self.agent_registry.read().await;
        let target = registry
            .get(&message.to)
            .ok_or_else(|| AgentError::NotFound(format!("Agent {} not found", message.to)))?;

        // Send message via flume channel
        target
            .message_sender
            .send_async(message.clone())
            .await
            .map_err(|e| AgentError::Network(format!("Failed to send message: {}", e)))?;

        // Broadcast to subscribers
        let _ = self.broadcast_sender.send(message.clone());

        // Update stats
        let mut stats = self.stats.lock().await;
        stats.messages_sent += 1;

        let processing_time = start_time
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;

        Ok(A2AResponse {
            message_id: message.id,
            status: ResponseStatus::Success,
            payload: None,
            error: None,
            processing_time_ms: processing_time,
        })
    }

    async fn request(&self, to: AgentId, payload: MessagePayload) -> Result<A2AResponse> {
        let message = A2AMessage {
            id: Uuid::new_v4().to_string(),
            from: self.config.agent_id.clone(),
            to,
            message_type: MessageType::Request,
            payload,
            priority: MessagePriority::Normal,
            timestamp: SystemTime::now(),
            expires_at: Some(SystemTime::now() + Duration::from_secs(30)),
            correlation_id: None,
            reply_to: None,
            metadata: HashMap::new(),
        };

        self.send_message(message).await
    }

    async fn notify(&self, to: AgentId, payload: MessagePayload) -> Result<()> {
        let message = A2AMessage {
            id: Uuid::new_v4().to_string(),
            from: self.config.agent_id.clone(),
            to,
            message_type: MessageType::Notification,
            payload,
            priority: MessagePriority::Normal,
            timestamp: SystemTime::now(),
            expires_at: None,
            correlation_id: None,
            reply_to: None,
            metadata: HashMap::new(),
        };

        self.send_message(message).await?;
        Ok(())
    }

    async fn broadcast(
        &self,
        to_agents: Vec<AgentId>,
        payload: MessagePayload,
    ) -> Result<Vec<A2AResponse>> {
        let mut responses = Vec::new();

        for agent_id in to_agents {
            let message = A2AMessage {
                id: Uuid::new_v4().to_string(),
                from: self.config.agent_id.clone(),
                to: agent_id,
                message_type: MessageType::Event,
                payload: payload.clone(),
                priority: MessagePriority::Normal,
                timestamp: SystemTime::now(),
                expires_at: None,
                correlation_id: None,
                reply_to: None,
                metadata: HashMap::new(),
            };

            match self.send_message(message).await {
                Ok(response) => responses.push(response),
                Err(e) => {
                    tracing::warn!("Failed to send broadcast message: {}", e);
                }
            }
        }

        Ok(responses)
    }

    async fn subscribe(
        &self,
        _message_types: Vec<MessageType>,
    ) -> Result<broadcast::Receiver<A2AMessage>> {
        Ok(self.broadcast_sender.subscribe())
    }

    async fn register(&self, capabilities: AgentCapabilities) -> Result<()> {
        let (sender, _receiver) = bounded::<A2AMessage>(100);

        let registration = AgentRegistration {
            agent_id: self.config.agent_id.clone(),
            capabilities,
            endpoints: HashMap::from([("local".to_string(), "inmemory".to_string())]),
            heartbeat_interval: Duration::from_secs(30),
            registered_at: SystemTime::now(),
            last_seen: SystemTime::now(),
            status: AgentStatus::Online,
        };

        let endpoint = LocalAgentEndpoint {
            registration,
            message_sender: sender,
            _message_receiver: None,
        };

        let mut registry = self.agent_registry.write().await;
        registry.insert(self.config.agent_id.clone(), endpoint);

        Ok(())
    }

    async fn unregister(&self) -> Result<()> {
        let mut registry = self.agent_registry.write().await;
        registry.remove(&self.config.agent_id);
        Ok(())
    }

    async fn discover_agents(&self, capability: &str) -> Result<Vec<AgentRegistration>> {
        let registry = self.agent_registry.read().await;
        let agents: Vec<AgentRegistration> = registry
            .values()
            .filter(|endpoint| {
                endpoint
                    .registration
                    .capabilities
                    .services
                    .contains(&capability.to_string())
            })
            .map(|endpoint| endpoint.registration.clone())
            .collect();

        Ok(agents)
    }

    async fn get_agent_info(&self, agent_id: &AgentId) -> Result<Option<AgentRegistration>> {
        let registry = self.agent_registry.read().await;
        Ok(registry
            .get(agent_id)
            .map(|endpoint| endpoint.registration.clone()))
    }

    async fn start(&self) -> Result<()> {
        tracing::info!(
            "Local A2A client started for agent: {}",
            self.config.agent_id
        );
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        tracing::info!(
            "Local A2A client stopped for agent: {}",
            self.config.agent_id
        );
        Ok(())
    }

    async fn get_stats(&self) -> Result<A2AStats> {
        let stats = self.stats.lock().await;
        Ok(stats.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_a2a_client_creation() {
        let config = A2AConfig::default();
        let client = LocalA2AClient::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let config = A2AConfig::default();
        let client = LocalA2AClient::new(config).unwrap();

        let agent_id = AgentId::new("test", "agent1");
        let capabilities = AgentCapabilities {
            services: vec!["test_service".to_string()],
            protocols: vec!["local".to_string()],
            message_types: vec!["request".to_string()],
            metadata: HashMap::new(),
        };

        let receiver = client
            .register_agent_with_channel(agent_id.clone(), capabilities, 10)
            .await;
        assert!(receiver.is_ok());

        let info = client.get_agent_info(&agent_id).await.unwrap();
        assert!(info.is_some());
    }

    #[tokio::test]
    async fn test_message_sending() {
        let config = A2AConfig::default();
        let client = LocalA2AClient::new(config).unwrap();

        // Register sender and receiver agents
        let sender_id = AgentId::new("test", "sender");
        let receiver_id = AgentId::new("test", "receiver");

        let capabilities = AgentCapabilities {
            services: vec!["echo".to_string()],
            protocols: vec!["local".to_string()],
            message_types: vec![],
            metadata: HashMap::new(),
        };

        let rx = client
            .register_agent_with_channel(receiver_id.clone(), capabilities, 10)
            .await
            .unwrap();

        // Send message
        let payload = MessagePayload::Text {
            content: "Hello, World!".to_string(),
        };

        let result = client.notify(receiver_id, payload).await;
        assert!(result.is_ok());

        // Verify message was received
        let received = rx.recv_async().await;
        assert!(received.is_ok());
        let msg = received.unwrap();
        assert_eq!(msg.message_type, MessageType::Notification);
    }
}
