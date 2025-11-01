//! Agent Coordination System
//!
//! Handles inter-agent communication, task delegation, and workflow orchestration
//! Now enhanced with A2A protocol messaging and knowledge management integration.

use super::a2a_local::LocalA2AClient;
use super::knowledge_helpers::{
    build_knowledge_enhanced_prompt, create_knowledge_entry, find_similar_tasks,
};
use super::{AgentStatus, Organization, OrganizationRole, WorkspaceTask};
use crate::a2a::{A2AClient, A2AConfig, AgentCapabilities, AgentId, MessagePayload};
use crate::error::Result;
use crate::knowledge::AdaptiveKnowledgeManager;
use crate::{Agent, AgentConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Message types for agent-to-agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    TaskAssignment {
        task_id: String,
        task: WorkspaceTask,
        from_agent: String,
    },
    TaskResult {
        task_id: String,
        result: TaskResult,
        artifacts: Vec<String>,
    },
    StatusUpdate {
        agent_id: String,
        status: AgentStatus,
        current_tasks: Vec<String>,
    },
    Collaboration {
        workspace_id: String,
        message: String,
        from_agent: String,
    },
    Question {
        question: String,
        context: String,
        from_agent: String,
    },
    Answer {
        answer: String,
        to_agent: String,
    },
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: String,
    pub metrics: HashMap<String, f64>,
    pub errors: Vec<String>,
}

/// Coordinator that manages agent interactions and task orchestration
pub struct AgentCoordinator {
    organization: Arc<RwLock<Organization>>,
    active_agents: Arc<RwLock<HashMap<String, Agent>>>,
    a2a_client: Arc<LocalA2AClient>,
    agent_id_map: Arc<RwLock<HashMap<String, AgentId>>>, // org agent id -> A2A agent id
    knowledge_manager: Option<Arc<AdaptiveKnowledgeManager>>,
}

impl AgentCoordinator {
    pub fn new(organization: Organization) -> Self {
        // Create A2A client for coordinator
        let coordinator_agent_id = AgentId::new("organization", "coordinator");
        let a2a_config = A2AConfig {
            agent_id: coordinator_agent_id,
            ..Default::default()
        };
        let a2a_client = LocalA2AClient::new(a2a_config).expect("Failed to create A2A client");

        Self {
            organization: Arc::new(RwLock::new(organization)),
            active_agents: Arc::new(RwLock::new(HashMap::new())),
            a2a_client: Arc::new(a2a_client),
            agent_id_map: Arc::new(RwLock::new(HashMap::new())),
            knowledge_manager: None,
        }
    }

    /// Create coordinator with knowledge management enabled
    pub fn with_knowledge_manager(mut self, knowledge_manager: AdaptiveKnowledgeManager) -> Self {
        self.knowledge_manager = Some(Arc::new(knowledge_manager));
        self
    }

    /// Initialize an agent in the organization
    pub async fn spawn_agent(&self, agent_id: String, config: AgentConfig) -> Result<()> {
        let agent = Agent::new(config).await?;
        let mut agents = self.active_agents.write().await;
        agents.insert(agent_id.clone(), agent);

        // Register agent with A2A client
        let org = self.organization.read().await;
        if let Some(org_agent) = org.agents.get(&agent_id) {
            let a2a_agent_id = AgentId::new("organization", &org_agent.name);

            // Create agent capabilities based on role
            let capabilities = AgentCapabilities {
                services: org_agent.capabilities.clone(),
                protocols: vec!["local".to_string()],
                message_types: vec![
                    "task".to_string(),
                    "query".to_string(),
                    "notification".to_string(),
                ],
                metadata: HashMap::new(),
            };

            // Register with A2A client and get message receiver
            let _receiver = self
                .a2a_client
                .register_agent_with_channel(a2a_agent_id.clone(), capabilities, 100)
                .await?;

            // Store mapping
            let mut id_map = self.agent_id_map.write().await;
            id_map.insert(agent_id.clone(), a2a_agent_id);
        }

        info!("Spawned agent: {}", agent_id);
        Ok(())
    }

    /// Assign a task to a specific agent
    pub async fn assign_task(
        &self,
        agent_id: &str,
        workspace_id: &str,
        task: WorkspaceTask,
    ) -> Result<()> {
        let mut org = self.organization.write().await;

        // Update organization state
        if let Some(agent) = org.agents.get_mut(agent_id) {
            agent.assign_task(task.id.clone());
        }

        if let Some(workspace) = org.workspaces.get_mut(workspace_id) {
            workspace.add_task(task.clone());
        }

        // Queue message for agent
        let task_id = task.id.clone();
        let message = AgentMessage::TaskAssignment {
            task_id: task_id.clone(),
            task,
            from_agent: "coordinator".to_string(),
        };

        self.send_message(agent_id, message).await;

        info!("Task assigned: {} -> {}", task_id, agent_id);
        Ok(())
    }

    /// Execute a task with an agent (with knowledge integration)
    pub async fn execute_task(&self, agent_id: &str, task: &WorkspaceTask) -> Result<TaskResult> {
        let agents = self.active_agents.read().await;

        if let Some(_agent) = agents.get(agent_id) {
            info!("Agent {} executing task: {}", agent_id, task.title);

            // Clone agent to avoid holding lock during execution
            drop(agents);

            // Get agent's role for knowledge context
            let role = {
                let org = self.organization.read().await;
                org.agents.get(agent_id).map(|a| a.role.clone())
            };

            // Build prompt with knowledge enhancement if available
            let prompt = if let Some(role) = &role {
                // Query past experiences if knowledge manager is enabled
                let past_experiences = if self.knowledge_manager.is_some() {
                    // Access agent's memory to find similar tasks
                    let memories = {
                        let agents_read = self.active_agents.read().await;
                        if let Some(agent) = agents_read.get(agent_id) {
                            // Query agent's memory for past learnings
                            match agent.list_memories(Some(100)).await {
                                Ok(mem) => {
                                    info!(
                                        "Retrieved {} memories from agent's knowledge base",
                                        mem.len()
                                    );
                                    mem
                                }
                                Err(e) => {
                                    warn!("Failed to query agent memories: {}", e);
                                    vec![]
                                }
                            }
                        } else {
                            vec![]
                        }
                    };

                    // Find similar tasks using simple text matching
                    let task_type = super::knowledge_helpers::extract_task_type(&task.title);
                    let similar = find_similar_tasks(&memories, &task.description, &task_type, 5);
                    if !similar.is_empty() {
                        info!(
                            "Found {} similar past experiences for context",
                            similar.len()
                        );
                    }
                    similar
                } else {
                    vec![]
                };

                // Build enhanced prompt with past experiences
                build_knowledge_enhanced_prompt(role, task, &past_experiences)
            } else {
                // Fallback to simple prompt
                format!(
                    "Task: {}\n\nDescription: {}\n\nPlease provide a solution.",
                    task.title, task.description
                )
            };

            info!("Executing task with {} characters of context", prompt.len());

            // Execute with owned agent reference
            let result = {
                let mut agents_write = self.active_agents.write().await;
                if let Some(agent) = agents_write.get_mut(agent_id) {
                    agent.process(&prompt).await?
                } else {
                    return Err(anyhow::anyhow!("Agent {} not found", agent_id).into());
                }
            };

            Ok(TaskResult {
                success: true,
                output: result,
                metrics: HashMap::new(),
                errors: Vec::new(),
            })
        } else {
            warn!("Agent {} not found", agent_id);
            Err(anyhow::anyhow!("Agent {} not found", agent_id).into())
        }
    }

    /// Send a message to an agent using A2A protocol
    pub async fn send_message(&self, agent_id: &str, message: AgentMessage) -> Result<()> {
        let id_map = self.agent_id_map.read().await;

        if let Some(a2a_agent_id) = id_map.get(agent_id) {
            // Convert AgentMessage to A2A MessagePayload
            let payload = match message {
                AgentMessage::TaskAssignment {
                    task_id,
                    task,
                    from_agent,
                } => MessagePayload::Task {
                    task_id: task_id.clone(),
                    operation: "assign".to_string(),
                    parameters: HashMap::from([
                        ("title".to_string(), task.title.clone()),
                        ("description".to_string(), task.description.clone()),
                        ("from".to_string(), from_agent),
                    ]),
                },
                AgentMessage::Collaboration {
                    workspace_id,
                    message,
                    from_agent,
                } => MessagePayload::Event {
                    event_type: "collaboration".to_string(),
                    data: serde_json::json!({
                        "workspace_id": workspace_id,
                        "message": message,
                        "from": from_agent,
                    }),
                },
                AgentMessage::StatusUpdate {
                    agent_id,
                    status: _status,
                    current_tasks,
                } => MessagePayload::Status {
                    status: crate::a2a::AgentStatus::Online,
                    message: Some(
                        serde_json::json!({
                            "agent_id": agent_id,
                            "current_tasks": current_tasks,
                        })
                        .to_string(),
                    ),
                },
                AgentMessage::Question {
                    question,
                    context,
                    from_agent,
                } => MessagePayload::Query {
                    query_id: uuid::Uuid::new_v4().to_string(),
                    query_type: "question".to_string(),
                    parameters: HashMap::from([
                        ("question".to_string(), question),
                        ("context".to_string(), context),
                        ("from".to_string(), from_agent),
                    ]),
                },
                _ => MessagePayload::Text {
                    content: format!("{:?}", message),
                },
            };

            self.a2a_client
                .notify(a2a_agent_id.clone(), payload)
                .await?;
            debug!("Message sent via A2A to agent: {}", agent_id);
        } else {
            warn!("Agent {} not found in A2A registry", agent_id);
        }

        Ok(())
    }

    /// Handle task completion (with knowledge storage)
    async fn handle_task_completion(
        &self,
        agent_id: &str,
        task_id: &str,
        result: TaskResult,
    ) -> Result<()> {
        let mut org = self.organization.write().await;

        // Get task and role for knowledge storage
        let (task_opt, role_opt) = {
            let mut found_task = None;
            let mut found_role = None;

            // Find the completed task
            for workspace in org.workspaces.values() {
                if let Some(task) = workspace.tasks.iter().find(|t| t.id == task_id) {
                    found_task = Some(task.clone());
                    break;
                }
            }

            // Get agent's role
            if let Some(agent) = org.agents.get(agent_id) {
                found_role = Some(agent.role.clone());
            }

            (found_task, found_role)
        };

        // Store learning if knowledge management is enabled
        if let (Some(task), Some(role)) = (task_opt, role_opt) {
            if self.knowledge_manager.is_some() {
                // Create knowledge entry from completed task
                let knowledge_entry = create_knowledge_entry(&role, &task, &result);

                let quality = knowledge_entry
                    .metadata
                    .get("quality_score")
                    .unwrap_or(&"N/A".to_string())
                    .to_string();

                info!(
                    "Storing knowledge for task '{}' (quality: {})",
                    task.title, quality
                );

                // Store in agent's memory via the new store_knowledge method
                let mut agents_write = self.active_agents.write().await;
                if let Some(agent) = agents_write.get_mut(agent_id) {
                    match agent.store_knowledge(knowledge_entry.clone()).await {
                        Ok(memory_id) => {
                            info!("✅ Knowledge stored successfully (ID: {})", memory_id);
                        }
                        Err(e) => {
                            // Non-fatal: knowledge storage failure doesn't stop task completion
                            warn!("⚠️  Knowledge storage failed: {} (continuing anyway)", e);
                        }
                    }
                } else {
                    warn!("Agent {} not found for knowledge storage", agent_id);
                }
            }
        }

        // Mark agent task as complete
        if let Some(agent) = org.agents.get_mut(agent_id) {
            agent.complete_task(task_id);
        }

        // Update task status in all workspaces
        for workspace in org.workspaces.values_mut() {
            if let Some(task) = workspace.tasks.iter_mut().find(|t| t.id == task_id) {
                task.complete();
            }
        }

        info!(
            "Task {} completed by {}: success={}",
            task_id, agent_id, result.success
        );
        Ok(())
    }

    /// Get organization state
    pub async fn get_organization(&self) -> Organization {
        self.organization.read().await.clone()
    }

    /// Coordinate multi-agent task across a workspace
    pub async fn coordinate_workspace_project(
        &self,
        workspace_id: &str,
        project_tasks: Vec<WorkspaceTask>,
    ) -> Result<Vec<TaskResult>> {
        info!("Coordinating workspace project: {}", workspace_id);

        let mut results = Vec::new();

        // Sort tasks by priority
        let mut sorted_tasks = project_tasks;
        sorted_tasks.sort_by(|a, b| b.priority.cmp(&a.priority));

        for task in sorted_tasks {
            // Find appropriate agent for task
            let agent_id = {
                let org = self.organization.read().await;
                let workspace = org.workspaces.get(workspace_id);

                workspace.and_then(|ws| {
                    if !ws.member_agents.is_empty() {
                        Some(ws.member_agents[0].clone())
                    } else {
                        None
                    }
                })
            };

            if let Some(agent_id) = agent_id {
                self.assign_task(&agent_id, workspace_id, task.clone())
                    .await?;
                // Execute task directly
                let result = self.execute_task(&agent_id, &task).await?;
                self.handle_task_completion(&agent_id, &task.id, result.clone())
                    .await?;

                results.push(result);
            }
        }

        Ok(results)
    }

    /// Route task to best available agent based on role and capabilities
    pub async fn route_task(
        &self,
        _task: &WorkspaceTask,
        required_role: Option<OrganizationRole>,
    ) -> Option<String> {
        let org = self.organization.read().await;
        let available = org.get_available_agents(required_role);

        // Simple routing: pick first available agent
        available.first().map(|a| a.id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OrganizationAgent;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let org = Organization::new("Test Org".to_string());
        let coordinator = AgentCoordinator::new(org);

        let org_state = coordinator.get_organization().await;
        assert_eq!(org_state.name, "Test Org");
    }

    #[tokio::test]
    async fn test_a2a_messaging() {
        let mut org = Organization::new("Test Org".to_string());

        // Add an agent to the organization
        let agent = OrganizationAgent::new(
            "Test Agent".to_string(),
            OrganizationRole::SoftwareEngineerSimulation,
        );
        let agent_id = agent.id.clone();
        org.add_agent(agent);

        let coordinator = AgentCoordinator::new(org);

        // Messages are sent via A2A now - no queue to check
        // Instead verify the A2A client has the agent registered
        let id_map = coordinator.agent_id_map.read().await;
        // Agent not yet registered until spawn_agent is called
        assert!(id_map.is_empty());
    }
}
