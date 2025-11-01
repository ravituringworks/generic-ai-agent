//! Multi-Agent Organization Example
//!
//! This example demonstrates a realistic multi-agent organization system
//! modeled after a robotics company structure. It shows:
//!
//! - Multiple specialized agents with different roles
//! - Collaborative workspaces for different projects
//! - Cross-workspace coordination
//! - Task dependencies and priority management
//! - Agent-to-agent communication

use anyhow::Result;
use the_agency::{
    organization::{
        coordinator::AgentCoordinator,
        CollaborativeWorkspace, Organization, OrganizationAgent, OrganizationRole, TaskPriority,
        WorkspaceTask,
    },
    AgentConfig,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n🤖 Multi-Agent Organization Demo\n");
    println!("==================================================\n");

    // Create organization
    let mut org = create_organization().await?;

    println!("✅ Organization created: {}", org.name);
    println!("   Total roles available: 60+");
    println!("   Agent count: {}", org.agents.len());
    println!();

    // Create and assign workspaces
    setup_workspaces(&mut org).await?;

    println!("✅ Workspaces configured: {}", org.workspaces.len());
    println!();

    // Initialize coordinator
    let coordinator = AgentCoordinator::new(org.clone());

    // Spawn AI agents
    println!("🚀 Spawning AI agents...\n");
    spawn_agents(&coordinator, &org).await?;
    println!("✅ All agents spawned and ready\n");

    // Execute multi-workspace projects
    println!("🎯 Executing Multi-Workspace Projects\n");
    println!("==================================================\n");

    execute_projects(&coordinator, &org).await?;

    // Display final state
    display_organization_state(&coordinator).await?;

    println!("\n✅ Demo complete!\n");

    Ok(())
}

/// Create the organization with agents
async fn create_organization() -> Result<Organization> {
    let mut org = Organization::new("RoboTech Industries".to_string());

    // Research & AI Team
    let alice = OrganizationAgent::new(
        "Alice Chen".to_string(),
        OrganizationRole::ResearchEngineerScaling,
    );
    org.add_agent(alice);

    let bob = OrganizationAgent::new(
        "Bob Martinez".to_string(),
        OrganizationRole::ResearchEngineerAutonomy,
    );
    org.add_agent(bob);

    let carol = OrganizationAgent::new(
        "Carol Kim".to_string(),
        OrganizationRole::ResearchEngineerWorldModels,
    );
    org.add_agent(carol);

    // Software Engineering Team
    let david = OrganizationAgent::new(
        "David Johnson".to_string(),
        OrganizationRole::SoftwareEngineerSimulation,
    );
    org.add_agent(david);

    let emily = OrganizationAgent::new(
        "Emily Zhang".to_string(),
        OrganizationRole::SoftwareEngineerPlatforms,
    );
    org.add_agent(emily);

    let frank = OrganizationAgent::new(
        "Frank Wilson".to_string(),
        OrganizationRole::SoftwareEngineerEmbeddedSystems,
    );
    org.add_agent(frank);

    // Hardware & Robotics Team
    let grace = OrganizationAgent::new(
        "Grace Lee".to_string(),
        OrganizationRole::HardcoreElectricalEngineer,
    );
    org.add_agent(grace);

    let henry = OrganizationAgent::new(
        "Henry Patel".to_string(),
        OrganizationRole::SeniorRoboticsEngineerControls,
    );
    org.add_agent(henry);

    let iris = OrganizationAgent::new(
        "Iris Anderson".to_string(),
        OrganizationRole::MechanicalEngineerAllLevels,
    );
    org.add_agent(iris);

    // Manufacturing & Production Team
    let jack = OrganizationAgent::new(
        "Jack Thompson".to_string(),
        OrganizationRole::ManufacturingEngineer,
    );
    org.add_agent(jack);

    let kate = OrganizationAgent::new(
        "Kate Brown".to_string(),
        OrganizationRole::AutomationEngineerManufacturing,
    );
    org.add_agent(kate);

    let leo = OrganizationAgent::new(
        "Leo Garcia".to_string(),
        OrganizationRole::QualityEngineerManufacturing,
    );
    org.add_agent(leo);

    // Supply Chain & Data Team
    let maya = OrganizationAgent::new(
        "Maya Nguyen".to_string(),
        OrganizationRole::NPIPlanner,
    );
    org.add_agent(maya);

    let noah = OrganizationAgent::new(
        "Noah Davis".to_string(),
        OrganizationRole::DataAnalyst,
    );
    org.add_agent(noah);

    Ok(org)
}

/// Setup collaborative workspaces
async fn setup_workspaces(org: &mut Organization) -> Result<()> {
    // Workspace 1: AI & Autonomy Research
    let mut ai_ws = CollaborativeWorkspace::new(
        "AI & Autonomy Research".to_string(),
        "Develop next-generation AI models for humanoid robot autonomy".to_string(),
    );
    let ai_ws_id = ai_ws.id.clone();
    org.create_workspace(ai_ws);

    // Assign AI/Research agents
    let agent_names = ["Alice Chen", "Bob Martinez", "Carol Kim"];
    for name in agent_names {
        let agent_id = org.agents.iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone());
        if let Some(id) = agent_id {
            org.assign_agent_to_workspace(&id, &ai_ws_id)?;
        }
    }

    // Workspace 2: Software Platform Development
    let mut sw_ws = CollaborativeWorkspace::new(
        "Software Platform".to_string(),
        "Build core software infrastructure and simulation systems".to_string(),
    );
    let sw_ws_id = sw_ws.id.clone();
    org.create_workspace(sw_ws);

    // Assign software engineers
    let agent_names = ["David Johnson", "Emily Zhang", "Frank Wilson"];
    for name in agent_names {
        let agent_id = org.agents.iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone());
        if let Some(id) = agent_id {
            org.assign_agent_to_workspace(&id, &sw_ws_id)?;
        }
    }

    // Workspace 3: Hardware Integration
    let mut hw_ws = CollaborativeWorkspace::new(
        "Hardware Integration".to_string(),
        "Design and integrate electrical, mechanical, and control systems".to_string(),
    );
    let hw_ws_id = hw_ws.id.clone();
    org.create_workspace(hw_ws);

    // Assign hardware/robotics engineers
    let agent_names = ["Grace Lee", "Henry Patel", "Iris Anderson"];
    for name in agent_names {
        let agent_id = org.agents.iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone());
        if let Some(id) = agent_id {
            org.assign_agent_to_workspace(&id, &hw_ws_id)?;
        }
    }

    // Workspace 4: Manufacturing Excellence
    let mut mfg_ws = CollaborativeWorkspace::new(
        "Manufacturing Excellence".to_string(),
        "Optimize production processes and ensure quality standards".to_string(),
    );
    let mfg_ws_id = mfg_ws.id.clone();
    org.create_workspace(mfg_ws);

    // Assign manufacturing team
    let agent_names = ["Jack Thompson", "Kate Brown", "Leo Garcia"];
    for name in agent_names {
        let agent_id = org.agents.iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone());
        if let Some(id) = agent_id {
            org.assign_agent_to_workspace(&id, &mfg_ws_id)?;
        }
    }

    // Workspace 5: Supply Chain & Analytics
    let mut sc_ws = CollaborativeWorkspace::new(
        "Supply Chain & Analytics".to_string(),
        "Manage supply chain planning and data-driven insights".to_string(),
    );
    let sc_ws_id = sc_ws.id.clone();
    org.create_workspace(sc_ws);

    // Assign supply chain team
    let agent_names = ["Maya Nguyen", "Noah Davis"];
    for name in agent_names {
        let agent_id = org.agents.iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone());
        if let Some(id) = agent_id {
            org.assign_agent_to_workspace(&id, &sc_ws_id)?;
        }
    }

    Ok(())
}

/// Spawn AI agents with configurations
async fn spawn_agents(coordinator: &AgentCoordinator, org: &Organization) -> Result<()> {
    let config = AgentConfig::default();

    for (agent_id, agent) in &org.agents {
        coordinator
            .spawn_agent(agent_id.clone(), config.clone())
            .await?;
        info!("  ✓ Spawned: {} ({})", agent.name, format!("{:?}", agent.role));
    }

    Ok(())
}

/// Execute projects across workspaces
async fn execute_projects(coordinator: &AgentCoordinator, org: &Organization) -> Result<()> {
    // Find workspace IDs
    let ai_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name == "AI & Autonomy Research")
        .map(|w| w.id.clone())
        .unwrap();

    let sw_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name == "Software Platform")
        .map(|w| w.id.clone())
        .unwrap();

    let hw_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name == "Hardware Integration")
        .map(|w| w.id.clone())
        .unwrap();

    let mfg_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name == "Manufacturing Excellence")
        .map(|w| w.id.clone())
        .unwrap();

    // Get agent IDs by name
    let get_agent_id = |name: &str| -> String {
        org.agents
            .iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone())
            .unwrap()
    };

    // Project 1: AI Research Tasks
    println!("📦 Project 1: AI & Autonomy Research\n");

    let task1 = WorkspaceTask::new(
        "Design World Model Architecture".to_string(),
        "Create architecture for next-gen world models in humanoid robots".to_string(),
        vec![get_agent_id("Carol Kim")],
    )
    .with_priority(TaskPriority::Critical);

    let task2 = WorkspaceTask::new(
        "Optimize RL Training Pipeline".to_string(),
        "Improve reinforcement learning training efficiency and scalability".to_string(),
        vec![get_agent_id("Alice Chen")],
    )
    .with_priority(TaskPriority::High);

    let ai_tasks = vec![task1, task2];
    let ai_results = coordinator
        .coordinate_workspace_project(&ai_ws_id, ai_tasks)
        .await?;

    println!("   ✅ Completed {} AI research tasks\n", ai_results.len());

    // Project 2: Software Development
    println!("📦 Project 2: Software Platform Development\n");

    let task3 = WorkspaceTask::new(
        "Build Robot Simulation Environment".to_string(),
        "Develop high-fidelity simulation for robot testing and training".to_string(),
        vec![get_agent_id("David Johnson")],
    )
    .with_priority(TaskPriority::Critical);

    let task4 = WorkspaceTask::new(
        "Implement Platform Infrastructure".to_string(),
        "Create scalable infrastructure for robot fleet management".to_string(),
        vec![get_agent_id("Emily Zhang")],
    )
    .with_priority(TaskPriority::High);

    let sw_tasks = vec![task3, task4];
    let sw_results = coordinator
        .coordinate_workspace_project(&sw_ws_id, sw_tasks)
        .await?;

    println!("   ✅ Completed {} software tasks\n", sw_results.len());

    // Project 3: Hardware Integration
    println!("📦 Project 3: Hardware Integration\n");

    let task5 = WorkspaceTask::new(
        "Design Motor Control System".to_string(),
        "Engineer advanced motor control for humanoid actuators".to_string(),
        vec![get_agent_id("Grace Lee")],
    )
    .with_priority(TaskPriority::Critical);

    let task6 = WorkspaceTask::new(
        "Integrate Robot Control Hardware".to_string(),
        "Integrate control systems with robot mechanical structure".to_string(),
        vec![get_agent_id("Henry Patel")],
    )
    .with_priority(TaskPriority::High);

    let hw_tasks = vec![task5, task6];
    let hw_results = coordinator
        .coordinate_workspace_project(&hw_ws_id, hw_tasks)
        .await?;

    println!("   ✅ Completed {} hardware tasks\n", hw_results.len());

    // Project 4: Manufacturing
    println!("📦 Project 4: Manufacturing Excellence\n");

    let task7 = WorkspaceTask::new(
        "Optimize Assembly Line".to_string(),
        "Improve robot assembly process efficiency and throughput".to_string(),
        vec![get_agent_id("Jack Thompson")],
    )
    .with_priority(TaskPriority::High);

    let task8 = WorkspaceTask::new(
        "Implement Quality Controls".to_string(),
        "Establish quality assurance protocols for production".to_string(),
        vec![get_agent_id("Leo Garcia")],
    )
    .with_priority(TaskPriority::Critical);

    let mfg_tasks = vec![task7, task8];
    let mfg_results = coordinator
        .coordinate_workspace_project(&mfg_ws_id, mfg_tasks)
        .await?;

    println!("   ✅ Completed {} manufacturing tasks\n", mfg_results.len());

    Ok(())
}

/// Display final organization state
async fn display_organization_state(coordinator: &AgentCoordinator) -> Result<()> {
    let org = coordinator.get_organization().await;

    println!("\n==================================================");
    println!("📊 Final Organization State");
    println!("==================================================\n");

    println!("🏢 Organization: {}", org.name);
    println!("👥 Total Agents: {}", org.agents.len());
    println!("🏗️  Total Workspaces: {}\n", org.workspaces.len());

    // Group agents by category
    use std::collections::HashMap;
    use the_agency::RoleCategory;

    let mut by_category: HashMap<String, Vec<&OrganizationAgent>> = HashMap::new();
    for agent in org.agents.values() {
        let category = format!("{:?}", agent.role.category());
        by_category.entry(category).or_default().push(agent);
    }

    println!("📋 Agents by Category:\n");
    for (category, agents) in by_category.iter() {
        println!("   {}: {} agents", category, agents.len());
    }

    println!("\n🏢 Workspace Summary:\n");
    for workspace in org.workspaces.values() {
        let completed = workspace
            .tasks
            .iter()
            .filter(|t| matches!(t.status, the_agency::TaskStatus::Completed))
            .count();

        println!("   📦 {}", workspace.name);
        println!("      Description: {}", workspace.description);
        println!("      Members: {} agents", workspace.member_agents.len());
        println!(
            "      Tasks: {}/{} completed",
            completed,
            workspace.tasks.len()
        );
        println!();
    }

    Ok(())
}
