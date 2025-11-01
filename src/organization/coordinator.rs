//! Agent Coordination System
//!
//! Handles inter-agent communication, task delegation, and workflow orchestration

use super::{AgentStatus, Organization, OrganizationRole, WorkspaceTask};
use crate::error::Result;
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
    message_queue: Arc<RwLock<Vec<(String, AgentMessage)>>>,
}

impl AgentCoordinator {
    pub fn new(organization: Organization) -> Self {
        Self {
            organization: Arc::new(RwLock::new(organization)),
            active_agents: Arc::new(RwLock::new(HashMap::new())),
            message_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Initialize an agent in the organization
    pub async fn spawn_agent(
        &self,
        agent_id: String,
        config: AgentConfig,
    ) -> Result<()> {
        let agent = Agent::new(config).await?;
        let mut agents = self.active_agents.write().await;
        agents.insert(agent_id.clone(), agent);
        
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

    /// Execute a task with an agent
    pub async fn execute_task(
        &self,
        agent_id: &str,
        task: &WorkspaceTask,
    ) -> Result<TaskResult> {
        let agents = self.active_agents.read().await;
        
        if let Some(agent) = agents.get(agent_id) {
            info!("Agent {} executing task: {}", agent_id, task.title);
            
            // Clone agent to avoid holding lock during execution
            drop(agents);
            
            let prompt = format!(
                "Task: {}\n\nDescription: {}\n\nPlease provide a solution.",
                task.title, task.description
            );
            
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

    /// Send a message to an agent
    pub async fn send_message(&self, agent_id: &str, message: AgentMessage) {
        let mut queue = self.message_queue.write().await;
        queue.push((agent_id.to_string(), message));
        debug!("Message queued for agent: {}", agent_id);
    }

    /// Process pending messages
    pub async fn process_messages(&self) -> Result<()> {
        let mut queue = self.message_queue.write().await;
        let messages: Vec<_> = queue.drain(..).collect();
        drop(queue);

        for (agent_id, message) in messages {
            match message {
                AgentMessage::TaskAssignment { task, .. } => {
                    debug!("Processing task assignment for {}: {}", agent_id, task.title);
                    // Execute task
                    if let Ok(result) = self.execute_task(&agent_id, &task).await {
                        self.handle_task_completion(&agent_id, &task.id, result).await?;
                    }
                }
                AgentMessage::StatusUpdate { .. } => {
                    debug!("Status update from {}", agent_id);
                }
                _ => {
                    debug!("Message processed for {}", agent_id);
                }
            }
        }

        Ok(())
    }

    /// Handle task completion
    async fn handle_task_completion(
        &self,
        agent_id: &str,
        task_id: &str,
        result: TaskResult,
    ) -> Result<()> {
        let mut org = self.organization.write().await;
        
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
        
        info!("Task {} completed by {}: success={}", task_id, agent_id, result.success);
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
                self.assign_task(&agent_id, workspace_id, task.clone()).await?;
                self.process_messages().await?;
                
                // Get result (simplified - in real system would track results better)
                results.push(TaskResult {
                    success: true,
                    output: format!("Task {} completed", task.title),
                    metrics: HashMap::new(),
                    errors: Vec::new(),
                });
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

    #[tokio::test]
    async fn test_coordinator_creation() {
        let org = Organization::new("Test Org".to_string());
        let coordinator = AgentCoordinator::new(org);
        
        let org_state = coordinator.get_organization().await;
        assert_eq!(org_state.name, "Test Org");
    }

    #[tokio::test]
    async fn test_message_queue() {
        let org = Organization::new("Test Org".to_string());
        let coordinator = AgentCoordinator::new(org);
        
        let message = AgentMessage::StatusUpdate {
            agent_id: "test_agent".to_string(),
            status: AgentStatus::Available,
            current_tasks: vec![],
        };
        
        coordinator.send_message("test_agent", message).await;
        
        let queue = coordinator.message_queue.read().await;
        assert_eq!(queue.len(), 1);
    }
}
