//! Organization Environment Daemon
//!
//! A daemon that manages multi-agent organizations with collaborative workspaces.
//! Provides real-time coordination and task orchestration for complex engineering teams.

use anyhow::Result;
use std::sync::Arc;
use the_agency::{
    organization::{
        coordinator::{AgentCoordinator, TaskResult},
        CollaborativeWorkspace, Organization, OrganizationAgent, OrganizationRole, TaskPriority,
        WorkspaceTask,
    },
    AgentConfig,
};
use tokio::sync::RwLock;
use tracing::{error, info};

/// Organization daemon that runs the agent environment
pub struct OrganizationDaemon {
    coordinator: Arc<AgentCoordinator>,
    running: Arc<RwLock<bool>>,
}

impl OrganizationDaemon {
    pub fn new(organization: Organization) -> Self {
        let coordinator = Arc::new(AgentCoordinator::new(organization));
        let running = Arc::new(RwLock::new(false));

        Self {
            coordinator,
            running,
        }
    }

    /// Start the daemon
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting Organization Daemon");

        let mut running = self.running.write().await;
        *running = true;
        drop(running);

        Ok(())
    }

    /// Stop the daemon
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping Organization Daemon");

        let mut running = self.running.write().await;
        *running = false;

        Ok(())
    }

    /// Check if daemon is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get the coordinator
    pub fn coordinator(&self) -> &AgentCoordinator {
        &self.coordinator
    }

    /// Run the event loop
    pub async fn run(&self) -> Result<()> {
        info!("📡 Organization Daemon event loop started");

        while self.is_running().await {
            // Process pending messages
            if let Err(e) = self.coordinator.process_messages().await {
                error!("Error processing messages: {}", e);
            }

            // Small delay to prevent busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        info!("📡 Organization Daemon event loop stopped");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("========================================");
    info!("   Organization Daemon");
    info!("========================================");
    info!("");

    // Create the organization
    let mut org = Organization::new("RoboTech Industries".to_string());

    // Add agents with various roles
    info!("👥 Creating organization agents...");

    let alice = OrganizationAgent::new(
        "Alice".to_string(),
        OrganizationRole::ResearchEngineerScaling,
    );
    let alice_id = org.add_agent(alice);

    let bob = OrganizationAgent::new(
        "Bob".to_string(),
        OrganizationRole::SoftwareEngineerSimulation,
    );
    let bob_id = org.add_agent(bob);

    let charlie = OrganizationAgent::new(
        "Charlie".to_string(),
        OrganizationRole::ManufacturingEngineer,
    );
    let charlie_id = org.add_agent(charlie);

    let diana = OrganizationAgent::new(
        "Diana".to_string(),
        OrganizationRole::RoboticsEngineerControlsTesting,
    );
    let diana_id = org.add_agent(diana);

    info!("✅ Created {} agents", org.agents.len());

    // Create workspaces
    info!("🏢 Creating collaborative workspaces...");

    let mut sim_workspace = CollaborativeWorkspace::new(
        "Simulation Development".to_string(),
        "Develop robotics simulation environment".to_string(),
    );
    let sim_ws_id = sim_workspace.id.clone();

    let mut prod_workspace = CollaborativeWorkspace::new(
        "Production Engineering".to_string(),
        "Manufacturing process optimization".to_string(),
    );
    let prod_ws_id = prod_workspace.id.clone();

    org.create_workspace(sim_workspace);
    org.create_workspace(prod_workspace);

    info!("✅ Created {} workspaces", org.workspaces.len());

    // Assign agents to workspaces
    info!("🔗 Assigning agents to workspaces...");

    org.assign_agent_to_workspace(&alice_id, &sim_ws_id)?;
    org.assign_agent_to_workspace(&bob_id, &sim_ws_id)?;
    org.assign_agent_to_workspace(&charlie_id, &prod_ws_id)?;
    org.assign_agent_to_workspace(&diana_id, &sim_ws_id)?;

    info!("✅ Agent assignments complete");
    info!("");

    // Create daemon
    let daemon = OrganizationDaemon::new(org);
    daemon.start().await?;

    // Initialize agents with configurations
    info!("🤖 Spawning AI agents...");

    let base_config = AgentConfig::default();

    daemon
        .coordinator()
        .spawn_agent(alice_id.clone(), base_config.clone())
        .await?;
    daemon
        .coordinator()
        .spawn_agent(bob_id.clone(), base_config.clone())
        .await?;
    daemon
        .coordinator()
        .spawn_agent(charlie_id.clone(), base_config.clone())
        .await?;
    daemon
        .coordinator()
        .spawn_agent(diana_id.clone(), base_config)
        .await?;

    info!("✅ All agents spawned and ready");
    info!("");

    // Create and assign tasks
    info!("📋 Creating project tasks...");

    let task1 = WorkspaceTask::new(
        "Design Simulation Architecture".to_string(),
        "Create a scalable architecture for robot simulation system".to_string(),
        vec![alice_id.clone()],
    )
    .with_priority(TaskPriority::High);

    let task2 = WorkspaceTask::new(
        "Implement Physics Engine".to_string(),
        "Build physics engine integration for robot simulation".to_string(),
        vec![bob_id.clone()],
    )
    .with_priority(TaskPriority::Critical)
    .with_dependencies(vec![task1.id.clone()]);

    let task3 = WorkspaceTask::new(
        "Optimize Manufacturing Process".to_string(),
        "Analyze and improve robot assembly line efficiency".to_string(),
        vec![charlie_id.clone()],
    )
    .with_priority(TaskPriority::High);

    let task4 = WorkspaceTask::new(
        "Test Control Systems".to_string(),
        "Validate robot control algorithms in simulation".to_string(),
        vec![diana_id.clone()],
    )
    .with_priority(TaskPriority::Medium)
    .with_dependencies(vec![task2.id.clone()]);

    info!("✅ Created 4 project tasks");
    info!("");

    // Execute workflow
    info!("🎯 Executing multi-agent workflow...");
    info!("==========================================");
    info!("");

    // Simulation workspace project
    info!("📦 Workspace: Simulation Development");
    let sim_tasks = vec![task1, task2, task4];
    let sim_results = daemon
        .coordinator()
        .coordinate_workspace_project(&sim_ws_id, sim_tasks)
        .await?;

    info!("");
    info!(
        "✅ Simulation workspace: {} tasks completed",
        sim_results.len()
    );

    // Production workspace project
    info!("");
    info!("📦 Workspace: Production Engineering");
    let prod_tasks = vec![task3];
    let prod_results = daemon
        .coordinator()
        .coordinate_workspace_project(&prod_ws_id, prod_tasks)
        .await?;

    info!("");
    info!(
        "✅ Production workspace: {} tasks completed",
        prod_results.len()
    );

    // Display final organization state
    info!("");
    info!("==========================================");
    info!("📊 Final Organization State");
    info!("==========================================");

    let final_org = daemon.coordinator().get_organization().await;

    info!("Organization: {}", final_org.name);
    info!("Total Agents: {}", final_org.agents.len());
    info!("Total Workspaces: {}", final_org.workspaces.len());

    for (ws_id, workspace) in &final_org.workspaces {
        let completed = workspace
            .tasks
            .iter()
            .filter(|t| matches!(t.status, the_agency::TaskStatus::Completed))
            .count();
        info!(
            "  - {}: {} members, {}/{} tasks completed",
            workspace.name,
            workspace.member_agents.len(),
            completed,
            workspace.tasks.len()
        );
    }

    info!("");

    // Display results
    info!("📈 Task Results Summary:");
    info!("  Simulation workspace: {} successful", sim_results.len());
    info!("  Production workspace: {} successful", prod_results.len());

    info!("");
    info!("==========================================");
    info!("✅ Organization workflow complete!");
    info!("==========================================");

    // Stop daemon
    daemon.stop().await?;

    Ok(())
}
