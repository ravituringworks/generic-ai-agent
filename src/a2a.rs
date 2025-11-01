//! Agent-to-Agent (A2A) Communication System
//!
//! This module provides a comprehensive framework for agents to communicate with each other
//! through various protocols and patterns including:
//! - HTTP REST APIs
//! - WebSockets for real-time communication  
//! - Message queues (Redis, RabbitMQ)
//! - P2P networks
//! - Event-driven communication
//! - Service discovery and registration

use crate::error::{AgentError, Result};
use async_trait::async_trait;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{broadcast, Mutex, RwLock};
use uuid::Uuid;

/// Unique identifier for an agent
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentId {
    pub namespace: String,
    pub name: String,
    pub instance: String,
}

impl AgentId {
    pub fn new(namespace: &str, name: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            name: name.to_string(),
            instance: Uuid::new_v4().to_string(),
        }
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.namespace, self.name, self.instance)
    }
}

/// Agent capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub services: Vec<String>,
    pub protocols: Vec<String>,
    pub message_types: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Agent registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub agent_id: AgentId,
    pub capabilities: AgentCapabilities,
    pub endpoints: HashMap<String, String>,
    pub heartbeat_interval: Duration,
    pub registered_at: SystemTime,
    pub last_seen: SystemTime,
    pub status: AgentStatus,
}

/// Agent status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Online,
    Busy,
    Idle,
    Offline,
    Error { message: String },
}

/// Communication message between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub id: String,
    pub from: AgentId,
    pub to: AgentId,
    pub message_type: MessageType,
    pub payload: MessagePayload,
    pub priority: MessagePriority,
    pub timestamp: SystemTime,
    pub expires_at: Option<SystemTime>,
    pub correlation_id: Option<String>,
    pub reply_to: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Types of messages that can be exchanged
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Event,
    Command,
    Query,
    Notification,
    Heartbeat,
    Acknowledgment,
}

/// Message payload variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Text {
        content: String,
    },
    Json {
        data: serde_json::Value,
    },
    Binary {
        data: Vec<u8>,
    },
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

/// Message priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Response from message processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AResponse {
    pub message_id: String,
    pub status: ResponseStatus,
    pub payload: Option<MessagePayload>,
    pub error: Option<String>,
    pub processing_time_ms: u64,
}

/// Response status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error,
    Timeout,
    Rejected,
    Processing,
}

/// Communication protocol types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolType {
    Http,
    WebSocket,
    Redis,
    RabbitMQ,
    Tcp,
    Udp,
    InMemory,
}

/// A2A transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AConfig {
    pub agent_id: AgentId,
    pub protocols: HashMap<ProtocolType, ProtocolConfig>,
    pub discovery: ServiceDiscoveryConfig,
    pub security: SecurityConfig,
    pub routing: RoutingConfig,
}

/// Protocol-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub timeout: Duration,
    pub retry_attempts: u32,
    pub connection_pool_size: u32,
    pub settings: HashMap<String, String>,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    pub enabled: bool,
    pub registry_type: String,
    pub registry_url: String,
    pub heartbeat_interval: Duration,
    pub discovery_interval: Duration,
    pub ttl: Duration,
}

/// Security configuration for A2A communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_authentication: bool,
    pub enable_encryption: bool,
    pub api_key: Option<String>,
    pub certificate_path: Option<String>,
    pub allowed_agents: Option<Vec<AgentId>>,
    pub rate_limit: Option<RateLimitConfig>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub enable_per_agent_limits: bool,
}

/// Message routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub enable_routing: bool,
    pub routing_table: HashMap<String, Vec<AgentId>>,
    pub load_balancing: LoadBalancingStrategy,
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    Random,
    ConsistentHashing,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub timeout: Duration,
    pub half_open_max_calls: u32,
}

/// Main A2A communication client trait
#[async_trait]
pub trait A2AClient: Send + Sync {
    /// Send a message to another agent
    async fn send_message(&self, message: A2AMessage) -> Result<A2AResponse>;

    /// Send a request and wait for response
    async fn request(&self, to: AgentId, payload: MessagePayload) -> Result<A2AResponse>;

    /// Send a one-way message (no response expected)
    async fn notify(&self, to: AgentId, payload: MessagePayload) -> Result<()>;

    /// Broadcast message to multiple agents
    async fn broadcast(
        &self,
        to_agents: Vec<AgentId>,
        payload: MessagePayload,
    ) -> Result<Vec<A2AResponse>>;

    /// Subscribe to messages from other agents
    async fn subscribe(
        &self,
        message_types: Vec<MessageType>,
    ) -> Result<tokio::sync::broadcast::Receiver<A2AMessage>>;

    /// Register this agent with the discovery service
    async fn register(&self, capabilities: AgentCapabilities) -> Result<()>;

    /// Unregister this agent from the discovery service
    async fn unregister(&self) -> Result<()>;

    /// Discover other agents by capability
    async fn discover_agents(&self, capability: &str) -> Result<Vec<AgentRegistration>>;

    /// Get agent information by ID
    async fn get_agent_info(&self, agent_id: &AgentId) -> Result<Option<AgentRegistration>>;

    /// Start the client (begin listening for messages)
    async fn start(&self) -> Result<()>;

    /// Stop the client
    async fn stop(&self) -> Result<()>;

    /// Get client statistics
    async fn get_stats(&self) -> Result<A2AStats>;
}

/// A2A client statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_failed: u64,
    pub active_connections: u32,
    pub discovered_agents: u32,
    pub uptime_seconds: u64,
    pub average_response_time_ms: f64,
}

/// Message handler trait for processing incoming messages
#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&self, message: A2AMessage) -> Result<Option<MessagePayload>>;
    async fn handle_request(&self, message: A2AMessage) -> Result<MessagePayload>;
    async fn handle_event(&self, message: A2AMessage) -> Result<()>;
    async fn handle_command(&self, message: A2AMessage) -> Result<MessagePayload>;
}

/// HTTP-based A2A client implementation
pub struct HttpA2AClient {
    config: A2AConfig,
    client: reqwest::Client,
    agent_registry: Arc<RwLock<HashMap<AgentId, AgentRegistration>>>,
    message_handlers: Arc<RwLock<Vec<Arc<dyn MessageHandler>>>>,
    stats: Arc<Mutex<A2AStats>>,
    message_sender: Arc<broadcast::Sender<A2AMessage>>,
    _message_receiver: broadcast::Receiver<A2AMessage>,
}

impl HttpA2AClient {
    pub fn new(config: A2AConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AgentError::Config(format!("Failed to create HTTP client: {}", e)))?;

        let (sender, receiver) = broadcast::channel(1000);

        Ok(Self {
            config,
            client,
            agent_registry: Arc::new(RwLock::new(HashMap::new())),
            message_handlers: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(Mutex::new(A2AStats::default())),
            message_sender: Arc::new(sender),
            _message_receiver: receiver,
        })
    }

    pub async fn add_message_handler(&self, handler: Arc<dyn MessageHandler>) {
        let mut handlers = self.message_handlers.write().await;
        handlers.push(handler);
    }

    #[allow(dead_code)]
    async fn process_incoming_message(
        &self,
        message: A2AMessage,
    ) -> Result<Option<MessagePayload>> {
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
impl A2AClient for HttpA2AClient {
    async fn send_message(&self, message: A2AMessage) -> Result<A2AResponse> {
        let start_time = SystemTime::now();

        // Look up target agent endpoint
        let registry = self.agent_registry.read().await;
        let target_agent = registry
            .get(&message.to)
            .ok_or_else(|| AgentError::NotFound(format!("Agent {} not found", message.to)))?;

        let endpoint = target_agent.endpoints.get("http").ok_or_else(|| {
            AgentError::Config("No HTTP endpoint found for target agent".to_string())
        })?;

        // Send HTTP request
        let response = self
            .client
            .post(format!("{}/a2a/message", endpoint))
            .json(&message)
            .send()
            .await
            .map_err(|e| AgentError::Network(format!("HTTP request failed: {}", e)))?;

        let _processing_time = start_time
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;

        if response.status().is_success() {
            let a2a_response: A2AResponse = response
                .json()
                .await
                .map_err(|e| AgentError::Network(format!("Failed to parse response: {}", e)))?;

            // Update stats
            let mut stats = self.stats.lock().await;
            stats.messages_sent += 1;

            Ok(a2a_response)
        } else {
            let mut stats = self.stats.lock().await;
            stats.messages_failed += 1;

            Err(AgentError::Network(format!(
                "HTTP request failed with status: {}",
                response.status()
            )))
        }
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
                    // Log error but continue with other agents
                    tracing::warn!("Failed to send broadcast message: {}", e);
                }
            }
        }

        Ok(responses)
    }

    async fn subscribe(
        &self,
        _message_types: Vec<MessageType>,
    ) -> Result<tokio::sync::broadcast::Receiver<A2AMessage>> {
        Ok(self.message_sender.subscribe())
    }

    async fn register(&self, capabilities: AgentCapabilities) -> Result<()> {
        let registration = AgentRegistration {
            agent_id: self.config.agent_id.clone(),
            capabilities,
            endpoints: HashMap::from([
                ("http".to_string(), "http://localhost:8080".to_string()), // Default endpoint
            ]),
            heartbeat_interval: Duration::from_secs(30),
            registered_at: SystemTime::now(),
            last_seen: SystemTime::now(),
            status: AgentStatus::Online,
        };

        // Store registration locally
        let mut registry = self.agent_registry.write().await;
        registry.insert(self.config.agent_id.clone(), registration);

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
            .filter(|agent| {
                agent
                    .capabilities
                    .services
                    .contains(&capability.to_string())
            })
            .cloned()
            .collect();

        Ok(agents)
    }

    async fn get_agent_info(&self, agent_id: &AgentId) -> Result<Option<AgentRegistration>> {
        let registry = self.agent_registry.read().await;
        Ok(registry.get(agent_id).cloned())
    }

    async fn start(&self) -> Result<()> {
        // In a full implementation, this would start HTTP server, WebSocket listeners, etc.
        tracing::info!(
            "A2A HTTP client started for agent: {}",
            self.config.agent_id.to_string()
        );
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        tracing::info!(
            "A2A HTTP client stopped for agent: {}",
            self.config.agent_id.to_string()
        );
        Ok(())
    }

    async fn get_stats(&self) -> Result<A2AStats> {
        let stats = self.stats.lock().await;
        Ok(stats.clone())
    }
}

impl Default for A2AStats {
    fn default() -> Self {
        Self {
            messages_sent: 0,
            messages_received: 0,
            messages_failed: 0,
            active_connections: 0,
            discovered_agents: 0,
            uptime_seconds: 0,
            average_response_time_ms: 0.0,
        }
    }
}

impl Default for A2AConfig {
    fn default() -> Self {
        Self {
            agent_id: AgentId::new("default", "agent"),
            protocols: HashMap::from([(
                ProtocolType::Http,
                ProtocolConfig {
                    enabled: true,
                    endpoint: "http://localhost:8080".to_string(),
                    timeout: Duration::from_secs(30),
                    retry_attempts: 3,
                    connection_pool_size: 10,
                    settings: HashMap::new(),
                },
            )]),
            discovery: ServiceDiscoveryConfig {
                enabled: true,
                registry_type: "http".to_string(),
                registry_url: "http://localhost:8500".to_string(),
                heartbeat_interval: Duration::from_secs(30),
                discovery_interval: Duration::from_secs(60),
                ttl: Duration::from_secs(90),
            },
            security: SecurityConfig {
                enable_authentication: false,
                enable_encryption: false,
                api_key: None,
                certificate_path: None,
                allowed_agents: None,
                rate_limit: None,
            },
            routing: RoutingConfig {
                enable_routing: false,
                routing_table: HashMap::new(),
                load_balancing: LoadBalancingStrategy::RoundRobin,
                circuit_breaker: CircuitBreakerConfig {
                    enabled: false,
                    failure_threshold: 5,
                    timeout: Duration::from_secs(60),
                    half_open_max_calls: 3,
                },
            },
        }
    }
}

/// A2A Manager that coordinates all agent-to-agent communication
pub struct A2AManager {
    client: Arc<dyn A2AClient>,
    handlers: Arc<RwLock<HashMap<String, Arc<dyn MessageHandler>>>>,
    agent_id: AgentId,
}

impl A2AManager {
    pub fn new(client: Arc<dyn A2AClient>, agent_id: AgentId) -> Self {
        Self {
            client,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            agent_id,
        }
    }

    pub async fn add_handler(&self, service_name: String, handler: Arc<dyn MessageHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(service_name, handler);
    }

    pub async fn start(&self) -> Result<()> {
        self.client.start().await
    }

    pub async fn stop(&self) -> Result<()> {
        self.client.stop().await
    }

    pub async fn send_request(
        &self,
        to: AgentId,
        service: &str,
        payload: MessagePayload,
    ) -> Result<A2AResponse> {
        let mut metadata = HashMap::new();
        metadata.insert("service".to_string(), service.to_string());

        let message = A2AMessage {
            id: Uuid::new_v4().to_string(),
            from: self.agent_id.clone(),
            to,
            message_type: MessageType::Request,
            payload,
            priority: MessagePriority::Normal,
            timestamp: SystemTime::now(),
            expires_at: Some(SystemTime::now() + Duration::from_secs(30)),
            correlation_id: None,
            reply_to: None,
            metadata,
        };

        self.client.send_message(message).await
    }

    pub async fn discover_service(&self, service_name: &str) -> Result<Vec<AgentRegistration>> {
        self.client.discover_agents(service_name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id_creation() {
        let agent_id = AgentId::new("test_namespace", "test_agent");
        assert_eq!(agent_id.namespace, "test_namespace");
        assert_eq!(agent_id.name, "test_agent");
        assert!(!agent_id.instance.is_empty());
    }

    #[test]
    fn test_message_creation() {
        let from = AgentId::new("ns1", "agent1");
        let to = AgentId::new("ns2", "agent2");

        let message = A2AMessage {
            id: Uuid::new_v4().to_string(),
            from,
            to,
            message_type: MessageType::Request,
            payload: MessagePayload::Text {
                content: "Hello".to_string(),
            },
            priority: MessagePriority::Normal,
            timestamp: SystemTime::now(),
            expires_at: None,
            correlation_id: None,
            reply_to: None,
            metadata: HashMap::new(),
        };

        assert_eq!(message.message_type, MessageType::Request);
        assert_eq!(message.priority, MessagePriority::Normal);
    }

    #[tokio::test]
    async fn test_http_a2a_client_creation() {
        let config = A2AConfig::default();
        let client = HttpA2AClient::new(config);
        assert!(client.is_ok());
    }
}
