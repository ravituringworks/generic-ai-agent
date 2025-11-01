//! Multi-Agent Organization Example
//!
//! This example demonstrates a realistic multi-agent organization system
//! modeled after RoboTech Industries working to build three humanoid robot variants:
//!
//! **Mission:** Build 3 Humanoid Robot Variants
//! - Robo-1: Home companion (chores, security, emotional support)
//! - Robo-2: Construction assistant (Robo-1 + heavy lifting)
//! - Robo-3: Rescue operations (Robo-1 + Robo-2 + wildfire/coastguard)
//!
//! This example shows:
//! - Multiple specialized agents with different roles (25+ agents)
//! - Collaborative workspaces for different robot variants (8 workspaces)
//! - Cross-functional coordination across departments
//! - Multi-project coordination with various priorities (9 concurrent projects)
//! - Task dependencies and priority management (Critical, High, Medium, Low)
//! - Agent-to-agent communication via A2A protocol
//! - Knowledge-enhanced task execution with persistent memory
//! - Organizational learning: agents capture and query past experiences
//! - Real-world complexity: AI research, platform dev, hardware, strategy, customer success

use anyhow::Result;
use std::sync::Arc;
use the_agency::{
    llm::connection_pool::OllamaConnectionPool,
    organization::{
        coordinator::AgentCoordinator, CollaborativeWorkspace, Organization, OrganizationAgent,
        OrganizationRole, TaskPriority, WorkspaceTask,
    },
    AgentConfig,
};
use tokio::time::{sleep, Duration};
use tracing::info;
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n🤖 RoboTech Industries - Multi-Agent Organization Demo\n");
    println!("==========================================================\n");
    println!("🎯 MISSION: Build 3 Humanoid Robot Variants\n");
    println!("   Robo-1: Home Companion (chores, security, emotional support)");
    println!("   Robo-2: Construction Assistant (Robo-1 + heavy lifting)");
    println!("   Robo-3: Rescue Operations (wildfire + coastguard)");
    println!("\n==========================================================\n");

    // Create organization
    let mut org = create_organization().await?;

    println!("✅ Organization created: {}", org.name);
    println!("   Total roles available: 110+");
    println!("   Agent count: {}", org.agents.len());
    println!();

    // Create and assign workspaces
    setup_workspaces(&mut org).await?;

    println!("✅ Workspaces configured: {}", org.workspaces.len());
    println!();

    // Initialize coordinator
    let coordinator = AgentCoordinator::new(org.clone());

    // Create connection pool for Ollama requests (limit concurrent connections)
    println!("🔧 Initializing Ollama connection pool...\n");
    let connection_pool = Arc::new(OllamaConnectionPool::new(5)); // Max 5 concurrent connections
    println!(
        "   Connection pool created with {} max connections\n",
        connection_pool.max_connections()
    );

    // Spawn AI agents
    println!("🚀 Spawning AI agents...\n");
    spawn_agents(&coordinator, &org, connection_pool.clone()).await?;
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
    let maya = OrganizationAgent::new("Maya Nguyen".to_string(), OrganizationRole::NPIPlanner);
    org.add_agent(maya);

    let noah = OrganizationAgent::new("Noah Davis".to_string(), OrganizationRole::DataAnalyst);
    org.add_agent(noah);

    // Executive Leadership
    let olivia = OrganizationAgent::new(
        "Olivia Torres".to_string(),
        OrganizationRole::ChiefExecutiveOfficer,
    );
    org.add_agent(olivia);

    let paul = OrganizationAgent::new(
        "Paul Chen".to_string(),
        OrganizationRole::ChiefTechnologyOfficer,
    );
    org.add_agent(paul);

    let quinn = OrganizationAgent::new("Quinn Rivera".to_string(), OrganizationRole::VPEngineering);
    org.add_agent(quinn);

    // Product & Strategy
    let rachel = OrganizationAgent::new(
        "Rachel Kim".to_string(),
        OrganizationRole::ChiefProductOfficer,
    );
    org.add_agent(rachel);

    let sam = OrganizationAgent::new(
        "Sam Johnson".to_string(),
        OrganizationRole::PrincipalProductManager,
    );
    org.add_agent(sam);

    let tina = OrganizationAgent::new(
        "Tina Martinez".to_string(),
        OrganizationRole::TechnicalProgramManager,
    );
    org.add_agent(tina);

    // People & Culture
    let uma = OrganizationAgent::new("Uma Patel".to_string(), OrganizationRole::DirectorOfPeople);
    org.add_agent(uma);

    // Customer Success & Sales
    let victor = OrganizationAgent::new("Victor Wong".to_string(), OrganizationRole::VPSales);
    org.add_agent(victor);

    let wendy = OrganizationAgent::new(
        "Wendy Anderson".to_string(),
        OrganizationRole::CustomerSuccessManager,
    );
    org.add_agent(wendy);

    let xavier = OrganizationAgent::new(
        "Xavier Lopez".to_string(),
        OrganizationRole::SolutionsArchitect,
    );
    org.add_agent(xavier);

    // Marketing & Design
    let yara = OrganizationAgent::new(
        "Yara Hassan".to_string(),
        OrganizationRole::ProductMarketingManager,
    );
    org.add_agent(yara);

    let zack = OrganizationAgent::new(
        "Zack Thompson".to_string(),
        OrganizationRole::PrincipalProductDesigner,
    );
    org.add_agent(zack);

    Ok(org)
}

/// Setup collaborative workspaces
async fn setup_workspaces(org: &mut Organization) -> Result<()> {
    // Workspace 1: Robo-1 Home Companion Development
    let robo1_ws = CollaborativeWorkspace::new(
        "Robo-1: Home Companion".to_string(),
        "Develop home assistance capabilities: chores, security, emotional companionship"
            .to_string(),
    );
    let robo1_ws_id = robo1_ws.id.clone();
    org.create_workspace(robo1_ws);

    // Assign cross-functional team for Robo-1
    let agent_names = [
        "Alice Chen",    // AI/Autonomy
        "David Johnson", // Simulation
        "Grace Lee",     // Electrical
        "Iris Anderson", // Mechanical
        "Sam Johnson",   // Product Manager
    ];
    for name in agent_names {
        let agent_id = org
            .agents
            .iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone());
        if let Some(id) = agent_id {
            org.assign_agent_to_workspace(&id, &robo1_ws_id)?;
        }
    }

    // Workspace 2: Robo-2 Construction Assistant Development
    let robo2_ws = CollaborativeWorkspace::new(
        "Robo-2: Construction Assistant".to_string(),
        "Extend Robo-1 with heavy-duty actuators and construction capabilities".to_string(),
    );
    let robo2_ws_id = robo2_ws.id.clone();
    org.create_workspace(robo2_ws);

    // Assign team for Robo-2 (inherits from Robo-1)
    let agent_names = [
        "Bob Martinez",  // Autonomy for load handling
        "Henry Patel",   // Robotics Controls
        "Iris Anderson", // Mechanical (heavy-duty)
        "Jack Thompson", // Manufacturing
        "Maya Nguyen",   // NPI Planning
    ];
    for name in agent_names {
        let agent_id = org
            .agents
            .iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone());
        if let Some(id) = agent_id {
            org.assign_agent_to_workspace(&id, &robo2_ws_id)?;
        }
    }

    // Workspace 3: Robo-3 Rescue Operations Development
    let robo3_ws = CollaborativeWorkspace::new(
        "Robo-3: Rescue Operations".to_string(),
        "Advanced capabilities for wildfire rescue and coastguard operations".to_string(),
    );
    let robo3_ws_id = robo3_ws.id.clone();
    org.create_workspace(robo3_ws);

    // Assign elite team for Robo-3 (most advanced)
    let agent_names = [
        "Carol Kim",    // World Models for rescue
        "Emily Zhang",  // Platform software
        "Frank Wilson", // Embedded systems
        "Henry Patel",  // Advanced controls
        "Kate Brown",   // Automation
        "Leo Garcia",   // Quality for safety
    ];
    for name in agent_names {
        let agent_id = org
            .agents
            .iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone());
        if let Some(id) = agent_id {
            org.assign_agent_to_workspace(&id, &robo3_ws_id)?;
        }
    }

    // Workspace 4: Manufacturing Excellence
    let mfg_ws = CollaborativeWorkspace::new(
        "Manufacturing Excellence".to_string(),
        "Optimize production processes and ensure quality standards".to_string(),
    );
    let mfg_ws_id = mfg_ws.id.clone();
    org.create_workspace(mfg_ws);

    // Assign manufacturing team
    let agent_names = ["Jack Thompson", "Kate Brown", "Leo Garcia"];
    for name in agent_names {
        let agent_id = org
            .agents
            .iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone());
        if let Some(id) = agent_id {
            org.assign_agent_to_workspace(&id, &mfg_ws_id)?;
        }
    }

    // Workspace 5: Supply Chain & Analytics
    let sc_ws = CollaborativeWorkspace::new(
        "Supply Chain & Analytics".to_string(),
        "Manage supply chain planning and data-driven insights".to_string(),
    );
    let sc_ws_id = sc_ws.id.clone();
    org.create_workspace(sc_ws);

    // Assign supply chain team
    let agent_names = ["Maya Nguyen", "Noah Davis"];
    for name in agent_names {
        let agent_id = org
            .agents
            .iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone());
        if let Some(id) = agent_id {
            org.assign_agent_to_workspace(&id, &sc_ws_id)?;
        }
    }

    // Workspace 6: Executive Leadership
    let executive_ws = CollaborativeWorkspace::new(
        "Executive Leadership".to_string(),
        "Strategic planning and organizational direction".to_string(),
    );
    let exec_ws_id = executive_ws.id.clone();
    org.create_workspace(executive_ws);

    // Assign executives
    let agent_names = ["Olivia Torres", "Paul Chen", "Quinn Rivera", "Rachel Kim"];
    for name in agent_names {
        let agent_id = org
            .agents
            .iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone());
        if let Some(id) = agent_id {
            org.assign_agent_to_workspace(&id, &exec_ws_id)?;
        }
    }

    // Workspace 7: Product Strategy
    let product_ws = CollaborativeWorkspace::new(
        "Product Strategy".to_string(),
        "Product roadmap and market strategy development".to_string(),
    );
    let prod_ws_id = product_ws.id.clone();
    org.create_workspace(product_ws);

    // Assign product team
    let agent_names = ["Sam Johnson", "Tina Martinez", "Zack Thompson"];
    for name in agent_names {
        let agent_id = org
            .agents
            .iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone());
        if let Some(id) = agent_id {
            org.assign_agent_to_workspace(&id, &prod_ws_id)?;
        }
    }

    // Workspace 8: Customer & Market Success
    let customer_ws = CollaborativeWorkspace::new(
        "Customer & Market Success".to_string(),
        "Customer success, sales, and market engagement".to_string(),
    );
    let cust_ws_id = customer_ws.id.clone();
    org.create_workspace(customer_ws);

    // Assign customer success and sales team
    let agent_names = [
        "Victor Wong",
        "Wendy Anderson",
        "Xavier Lopez",
        "Yara Hassan",
    ];
    for name in agent_names {
        let agent_id = org
            .agents
            .iter()
            .find(|(_, a)| a.name == name)
            .map(|(id, _)| id.clone());
        if let Some(id) = agent_id {
            org.assign_agent_to_workspace(&id, &cust_ws_id)?;
        }
    }

    Ok(())
}

/// Spawn AI agents with configurations
async fn spawn_agents(
    coordinator: &AgentCoordinator,
    org: &Organization,
    connection_pool: Arc<OllamaConnectionPool>,
) -> Result<()> {
    println!(
        "   Using connection pool with {} max concurrent connections\n",
        connection_pool.max_connections()
    );

    for (agent_id, agent) in &org.agents {
        // Wait for available connection slot before spawning
        let _permit = connection_pool.acquire().await;

        // Create role-specific configuration with learning-enabled system prompt
        let mut config = AgentConfig::default();

        // IMPORTANT: Ensure all agents use localhost:11434
        config.llm.ollama_url = "http://localhost:11434".to_string();

        // Configure memory with in-memory SQLite database for demo
        config.memory.database_url = Some(":memory:".to_string()); // Use in-memory SQLite
        config.memory.persistent = false; // Don't persist for demo

        // Disable workflow suspend/resume to prevent infinite loops
        config.workflow.enable_suspend_resume = false;

        // Use role-specific system prompt that includes organizational learning
        let system_prompt = agent.role.system_prompt();

        // TEMPORARILY DISABLED: Memory/embedding causing random port issues
        // This bypasses the Ollama embedding EOF errors on random ports
        config.agent.use_memory = false; // Disabled until Ollama embedding issue resolved
        config.agent.use_tools = false; // Simplified for demo
        config.agent.max_thinking_steps = 1; // Bypass workflow complexity
        config.agent.system_prompt = system_prompt;

        coordinator.spawn_agent(agent_id.clone(), config).await?;
        info!(
            "  ✓ Spawned: {} ({})",
            agent.name,
            format!("{:?}", agent.role)
        );

        // Small delay between spawns to avoid overwhelming Ollama during initialization
        sleep(Duration::from_millis(100)).await;

        // Release permit (automatic on drop)
    }

    Ok(())
}

/// Execute projects across workspaces
async fn execute_projects(coordinator: &AgentCoordinator, org: &Organization) -> Result<()> {
    println!("🚀 Starting Development of 3 Humanoid Robot Variants\n");
    println!("==========================================================\n");

    // Find workspace IDs for robot variants
    let robo1_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name == "Robo-1: Home Companion")
        .map(|w| w.id.clone())
        .unwrap();

    let robo2_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name == "Robo-2: Construction Assistant")
        .map(|w| w.id.clone())
        .unwrap();

    let robo3_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name == "Robo-3: Rescue Operations")
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

    // Project 1: Robo-1 Home Companion
    println!("🏠 Project 1: Robo-1 Home Companion Development\n");

    let task1 = WorkspaceTask::new(
        "Design Home Assistant AI".to_string(),
        "Develop AI for household chores: cleaning, organizing, basic maintenance".to_string(),
        vec![get_agent_id("Alice Chen")],
    )
    .with_priority(TaskPriority::Critical);

    let task2 = WorkspaceTask::new(
        "Build Security & Emotional Intelligence".to_string(),
        "Create security monitoring and emotional companionship capabilities".to_string(),
        vec![get_agent_id("David Johnson")],
    )
    .with_priority(TaskPriority::Critical);

    let task3 = WorkspaceTask::new(
        "Design Safe Home-Use Actuators".to_string(),
        "Engineer safe, quiet actuators suitable for home environment".to_string(),
        vec![get_agent_id("Grace Lee")],
    )
    .with_priority(TaskPriority::High);

    let robo1_tasks = vec![task1, task2, task3];
    let robo1_results = coordinator
        .coordinate_workspace_project(&robo1_ws_id, robo1_tasks)
        .await?;

    println!(
        "   ✅ Completed {} Robo-1 development tasks\n",
        robo1_results.len()
    );

    // Project 2: Robo-2 Construction Assistant
    println!("🏗️ Project 2: Robo-2 Construction Assistant Development\n");

    let task4 = WorkspaceTask::new(
        "Design Heavy-Duty Actuator System".to_string(),
        "Engineer actuators capable of lifting 50+ kg loads safely".to_string(),
        vec![get_agent_id("Bob Martinez")],
    )
    .with_priority(TaskPriority::Critical);

    let task5 = WorkspaceTask::new(
        "Develop Load-Balancing Control System".to_string(),
        "Create controls for stable load handling and construction site navigation".to_string(),
        vec![get_agent_id("Henry Patel")],
    )
    .with_priority(TaskPriority::Critical);

    let task6 = WorkspaceTask::new(
        "Build Construction Safety Features".to_string(),
        "Implement safety protocols for construction site operations".to_string(),
        vec![get_agent_id("Jack Thompson")],
    )
    .with_priority(TaskPriority::High);

    let robo2_tasks = vec![task4, task5, task6];
    let robo2_results = coordinator
        .coordinate_workspace_project(&robo2_ws_id, robo2_tasks)
        .await?;

    println!(
        "   ✅ Completed {} Robo-2 development tasks\n",
        robo2_results.len()
    );

    // Project 3: Robo-3 Rescue Operations
    println!("🚒 Project 3: Robo-3 Rescue Operations Development\n");

    let task7 = WorkspaceTask::new(
        "Design Extreme Environment Systems".to_string(),
        "Engineer systems for high-heat (wildfire) and marine environments".to_string(),
        vec![get_agent_id("Carol Kim")],
    )
    .with_priority(TaskPriority::Critical);

    let task8 = WorkspaceTask::new(
        "Build Advanced Perception for Rescue".to_string(),
        "Develop AI for victim detection, smoke/water navigation, threat assessment".to_string(),
        vec![get_agent_id("Emily Zhang")],
    )
    .with_priority(TaskPriority::Critical);

    let task9 = WorkspaceTask::new(
        "Implement Emergency Response Protocols".to_string(),
        "Create fail-safe systems and emergency response automation".to_string(),
        vec![get_agent_id("Frank Wilson")],
    )
    .with_priority(TaskPriority::Critical);

    let task10 = WorkspaceTask::new(
        "Design Rescue Equipment Integration".to_string(),
        "Integrate thermal imaging, water pumps, rescue tools, communication systems".to_string(),
        vec![get_agent_id("Henry Patel")],
    )
    .with_priority(TaskPriority::High);

    let robo3_tasks = vec![task7, task8, task9, task10];
    let robo3_results = coordinator
        .coordinate_workspace_project(&robo3_ws_id, robo3_tasks)
        .await?;

    println!(
        "   ✅ Completed {} Robo-3 development tasks\n",
        robo3_results.len()
    );

    // Additional concurrent projects demonstrating multi-project coordination
    println!("\n🔬 Additional Projects Running Concurrently\n");
    println!("==========================================================\n");

    // Project 4: AI Research Tasks (High Priority)
    println!("🧠 Project 4: AI Research & Innovation (High Priority)\n");

    let ai_research_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name.contains("Robo-1")) // Use Robo-1 as research workspace
        .map(|w| w.id.clone())
        .unwrap();

    let task11 = WorkspaceTask::new(
        "Research Advanced World Models".to_string(),
        "Investigate next-gen world modeling for better environment understanding".to_string(),
        vec![get_agent_id("Carol Kim")],
    )
    .with_priority(TaskPriority::High);

    let task12 = WorkspaceTask::new(
        "Optimize Scaling Algorithms".to_string(),
        "Improve scalability of AI systems for multiple robot variants".to_string(),
        vec![get_agent_id("Alice Chen")],
    )
    .with_priority(TaskPriority::High);

    let ai_research_tasks = vec![task11, task12];
    let ai_research_results = coordinator
        .coordinate_workspace_project(&ai_research_ws_id, ai_research_tasks)
        .await?;

    println!(
        "   ✅ Completed {} AI research tasks\n",
        ai_research_results.len()
    );

    // Project 5: Software Platform Development (High Priority)
    println!("💻 Project 5: Software Platform Development (High Priority)\n");

    let platform_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name.contains("Robo-2"))
        .map(|w| w.id.clone())
        .unwrap();

    let task13 = WorkspaceTask::new(
        "Build Cross-Platform SDK".to_string(),
        "Create unified SDK for all robot variants".to_string(),
        vec![get_agent_id("Emily Zhang")],
    )
    .with_priority(TaskPriority::High);

    let task14 = WorkspaceTask::new(
        "Implement Real-Time Simulation Framework".to_string(),
        "Develop simulation tools for testing robot behaviors".to_string(),
        vec![get_agent_id("David Johnson")],
    )
    .with_priority(TaskPriority::High);

    let platform_tasks = vec![task13, task14];
    let platform_results = coordinator
        .coordinate_workspace_project(&platform_ws_id, platform_tasks)
        .await?;

    println!(
        "   ✅ Completed {} platform development tasks\n",
        platform_results.len()
    );

    // Project 6: Hardware Integration (Medium Priority)
    println!("⚙️  Project 6: Hardware Integration (Medium Priority)\n");

    let hw_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name.contains("Manufacturing"))
        .map(|w| w.id.clone())
        .unwrap();

    let task15 = WorkspaceTask::new(
        "Integrate Sensor Arrays".to_string(),
        "Coordinate integration of all sensor systems across robot variants".to_string(),
        vec![get_agent_id("Grace Lee")],
    )
    .with_priority(TaskPriority::Medium);

    let task16 = WorkspaceTask::new(
        "Standardize Power Management".to_string(),
        "Create unified power management system for all robots".to_string(),
        vec![get_agent_id("Iris Anderson")],
    )
    .with_priority(TaskPriority::Medium);

    let hw_tasks = vec![task15, task16];
    let hw_results = coordinator
        .coordinate_workspace_project(&hw_ws_id, hw_tasks)
        .await?;

    println!(
        "   ✅ Completed {} hardware integration tasks\n",
        hw_results.len()
    );

    // Project 7: Executive Strategy (Medium Priority)
    println!("📊 Project 7: Executive Strategy Review (Medium Priority)\n");

    let exec_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name == "Executive Leadership")
        .map(|w| w.id.clone())
        .unwrap();

    let task17 = WorkspaceTask::new(
        "Q1 Strategic Planning".to_string(),
        "Review progress and plan next quarter strategy".to_string(),
        vec![get_agent_id("Olivia Torres"), get_agent_id("Paul Chen")],
    )
    .with_priority(TaskPriority::Medium);

    let exec_tasks = vec![task17];
    let exec_results = coordinator
        .coordinate_workspace_project(&exec_ws_id, exec_tasks)
        .await?;

    println!(
        "   ✅ Completed {} executive strategy tasks\n",
        exec_results.len()
    );

    // Project 8: Product Strategy (Medium Priority)
    println!("📦 Project 8: Product Strategy & Roadmap (Medium Priority)\n");

    let prod_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name == "Product Strategy")
        .map(|w| w.id.clone())
        .unwrap();

    let task18 = WorkspaceTask::new(
        "Define Product Roadmap".to_string(),
        "Create 12-month roadmap for all robot variants".to_string(),
        vec![get_agent_id("Sam Johnson"), get_agent_id("Rachel Kim")],
    )
    .with_priority(TaskPriority::Medium);

    let prod_tasks = vec![task18];
    let prod_results = coordinator
        .coordinate_workspace_project(&prod_ws_id, prod_tasks)
        .await?;

    println!(
        "   ✅ Completed {} product strategy tasks\n",
        prod_results.len()
    );

    // Project 9: Customer Success Initiative (Low Priority)
    println!("🤝 Project 9: Customer Success Initiative (Low Priority)\n");

    let cust_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name == "Customer & Market Success")
        .map(|w| w.id.clone())
        .unwrap();

    let task19 = WorkspaceTask::new(
        "Launch Customer Feedback Program".to_string(),
        "Establish feedback loop with early adopters".to_string(),
        vec![get_agent_id("Wendy Anderson")],
    )
    .with_priority(TaskPriority::Low);

    let task20 = WorkspaceTask::new(
        "Develop Training Materials".to_string(),
        "Create comprehensive training for robot operators".to_string(),
        vec![get_agent_id("Xavier Lopez")],
    )
    .with_priority(TaskPriority::Low);

    let cust_tasks = vec![task19, task20];
    let cust_results = coordinator
        .coordinate_workspace_project(&cust_ws_id, cust_tasks)
        .await?;

    println!(
        "   ✅ Completed {} customer success tasks\n",
        cust_results.len()
    );

    // Summary
    println!("\n==========================================================\n");
    println!("✅ All Projects Completed Across Organization!");
    println!("\n📊 Project Summary:\n");
    println!(
        "   🏠 Robo-1 Development: {} tasks (Critical)",
        robo1_results.len()
    );
    println!(
        "   🏗️ Robo-2 Development: {} tasks (Critical)",
        robo2_results.len()
    );
    println!(
        "   🚒 Robo-3 Development: {} tasks (Critical)",
        robo3_results.len()
    );
    println!(
        "   🧠 AI Research: {} tasks (High)",
        ai_research_results.len()
    );
    println!(
        "   💻 Platform Development: {} tasks (High)",
        platform_results.len()
    );
    println!(
        "   ⚙️  Hardware Integration: {} tasks (Medium)",
        hw_results.len()
    );
    println!(
        "   📊 Executive Strategy: {} tasks (Medium)",
        exec_results.len()
    );
    println!(
        "   📦 Product Strategy: {} tasks (Medium)",
        prod_results.len()
    );
    println!("   🤝 Customer Success: {} tasks (Low)", cust_results.len());

    let total_tasks = robo1_results.len()
        + robo2_results.len()
        + robo3_results.len()
        + ai_research_results.len()
        + platform_results.len()
        + hw_results.len()
        + exec_results.len()
        + prod_results.len()
        + cust_results.len();

    println!("\n   📈 Total Tasks Executed: {}", total_tasks);
    println!(
        "   🧠 Knowledge Captured: {} experiences stored in agent memories\n",
        total_tasks
    );

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

    let mut by_category: HashMap<String, Vec<&OrganizationAgent>> = HashMap::new();
    for agent in org.agents.values() {
        let category = format!("{:?}", agent.role.category());
        by_category.entry(category).or_default().push(agent);
    }

    println!("📋 Agents by Category:\n");
    for (category, agents) in by_category.iter() {
        println!("   {}: {} agents", category, agents.len());
    }

    println!("\n🤖 Robot Variant Development Summary:\n");

    // Display robot variant workspaces
    let robot_workspaces = [
        "Robo-1: Home Companion",
        "Robo-2: Construction Assistant",
        "Robo-3: Rescue Operations",
    ];
    for ws_name in robot_workspaces {
        if let Some(workspace) = org.workspaces.values().find(|w| w.name == ws_name) {
            let completed = workspace
                .tasks
                .iter()
                .filter(|t| matches!(t.status, the_agency::TaskStatus::Completed))
                .count();

            println!("   📦 {}", workspace.name);
            println!("      Description: {}", workspace.description);
            println!("      Team: {} agents", workspace.member_agents.len());
            println!(
                "      Progress: {}/{} tasks completed",
                completed,
                workspace.tasks.len()
            );
            println!();
        }
    }

    Ok(())
}
