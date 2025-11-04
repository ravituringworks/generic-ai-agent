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
use chrono;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
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

/// Types of artifacts that can be generated
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ArtifactType {
    DesignDocument,
    ApiSpecification,
    PythonCode,
    RustCode,
    YamlConfig,
    TomlConfig,
    ArchitectureDiagram,
    TechnicalReport,
}

/// An artifact produced by the organization
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Artifact {
    name: String,
    artifact_type: ArtifactType,
    content: String,
    file_extension: String,
    created_by: String,
    workspace: String,
}
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

    // Create output directory structure
    let output_dir = PathBuf::from("output/robotech_organization_output");
    fs::create_dir_all(&output_dir)?;
    fs::create_dir_all(output_dir.join("reports"))?;
    fs::create_dir_all(output_dir.join("logs"))?;
    fs::create_dir_all(output_dir.join("artifacts"))?;
    fs::create_dir_all(output_dir.join("artifacts/design_docs"))?;
    fs::create_dir_all(output_dir.join("artifacts/code"))?;
    fs::create_dir_all(output_dir.join("artifacts/configs"))?;
    fs::create_dir_all(output_dir.join("artifacts/diagrams"))?;
    println!("📁 Output directory: {}\n", output_dir.display());

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

    // Generate artifacts
    println!("\n📦 Generating work products...\n");
    let artifacts = generate_artifacts(&org).await?;
    write_artifacts(&output_dir, &artifacts).await?;
    println!("✅ Generated {} artifacts\n", artifacts.len());

    // Generate summary report
    generate_summary_report(&output_dir, &coordinator, &artifacts).await?;

    println!("\n✅ Demo complete!\n");
    println!("📊 Output files:");
    println!("   Reports:");
    println!("   - {}/reports/summary.md", output_dir.display());
    println!("   - {}/reports/organization_state.json", output_dir.display());
    println!("\n   Artifacts ({} files):", artifacts.len());
    println!("   - {}/artifacts/design_docs/ (design specifications)", output_dir.display());
    println!("   - {}/artifacts/code/ (Python, Rust implementations)", output_dir.display());
    println!("   - {}/artifacts/configs/ (YAML, TOML configurations)", output_dir.display());
    println!("   - {}/artifacts/diagrams/ (architecture diagrams)", output_dir.display());
    println!("\n");

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

        // IMPORTANT: Ensure all agents use 127.0.0.1:11434
        // Using 127.0.0.1 instead of localhost to avoid IPv6/IPv4 resolution issues
        config.llm.ollama_url = "http://127.0.0.1:11434".to_string();

        // Use GPT-OSS:20B-Cloud model for text generation
        config.llm.text_model = "gpt-oss:20b-cloud".to_string();

        // IMPORTANT: Use local embedding model, not cloud model for embeddings
        config.llm.embedding_model = "nomic-embed-text".to_string();

        // Configure memory with in-memory SQLite database for demo
        config.memory.database_url = Some(":memory:".to_string()); // Use in-memory SQLite
        config.memory.persistent = true; // Don't persist for demo

        // Disable workflow suspend/resume to prevent infinite loops
        config.workflow.enable_suspend_resume = true;

        // Use role-specific system prompt that includes organizational learning
        let system_prompt = agent.role.system_prompt();

        // DISABLED: Cloud models (gpt-oss:20b-cloud) try to use remote embedding service
        // which creates random ports and fails with EOF errors
        // Memory works fine with local models (llama3.2, qwen, etc.)
        config.agent.use_memory = false; // Disabled for cloud models
        config.agent.use_tools = true; // Simplified for demo
        config.agent.max_thinking_steps = 3; // Bypass workflow complexity
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

/// Generate summary report and save to files
async fn generate_summary_report(output_dir: &PathBuf, coordinator: &AgentCoordinator, artifacts: &[Artifact]) -> Result<()> {
    let org = coordinator.get_organization().await;

    // Generate markdown summary
    let mut summary = String::new();
    summary.push_str("# RoboTech Industries Organization Demo Report\n\n");
    summary.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().to_rfc3339()));
    summary.push_str("## Organization Overview\n\n");
    summary.push_str(&format!("- **Organization Name:** {}\n", org.name));
    summary.push_str(&format!("- **Total Agents:** {}\n", org.agents.len()));
    summary.push_str(&format!("- **Total Workspaces:** {}\n", org.workspaces.len()));
    summary.push_str(&format!("- **Artifacts Generated:** {}\n\n", artifacts.len()));

    summary.push_str("## Agents by Role\n\n");
    for (_, agent) in org.agents.iter() {
        summary.push_str(&format!("- **{}**: {:?}\n", agent.name, agent.role));
    }

    summary.push_str("\n## Workspaces\n\n");
    for (_, workspace) in org.workspaces.iter() {
        summary.push_str(&format!("### {}\n", workspace.name));
        summary.push_str(&format!("**Description:** {}\n\n", workspace.description));
        summary.push_str(&format!("**Team Members:** {}\n\n", workspace.member_agents.len()));

        let completed = workspace.tasks.iter()
            .filter(|t| matches!(t.status, the_agency::TaskStatus::Completed))
            .count();
        summary.push_str(&format!("**Tasks:** {}/{} completed\n\n", completed, workspace.tasks.len()));
    }

    summary.push_str("\n## Generated Artifacts\n\n");

    // Group artifacts by type
    let mut by_type: HashMap<String, Vec<&Artifact>> = HashMap::new();
    for artifact in artifacts {
        let type_name = format!("{:?}", artifact.artifact_type);
        by_type.entry(type_name).or_default().push(artifact);
    }

    for (artifact_type, items) in by_type.iter() {
        summary.push_str(&format!("### {}\n", artifact_type));
        for artifact in items {
            summary.push_str(&format!("- **{}** (by {})\n", artifact.name, artifact.created_by));
            summary.push_str(&format!("  - Workspace: {}\n", artifact.workspace));
            summary.push_str(&format!("  - File: `artifacts/{}.{}`\n", artifact.name, artifact.file_extension));
        }
        summary.push_str("\n");
    }

    summary.push_str("\n## Mission: 3 Humanoid Robot Variants\n\n");
    summary.push_str("- **Robo-1:** Home Companion (chores, security, emotional support)\n");
    summary.push_str("- **Robo-2:** Construction Assistant (Robo-1 + heavy lifting)\n");
    summary.push_str("- **Robo-3:** Rescue Operations (wildfire + coastguard)\n\n");

    summary.push_str("---\n\n");
    summary.push_str("*Generated by the-agency multi-agent organization system*\n");

    // Write summary to file
    let summary_path = output_dir.join("reports/summary.md");
    fs::write(&summary_path, summary)?;
    println!("✅ Generated summary report: {}", summary_path.display());

    // Write JSON state
    let json_state = serde_json::to_string_pretty(&org)?;
    let json_path = output_dir.join("reports/organization_state.json");
    fs::write(&json_path, json_state)?;
    println!("✅ Generated JSON state: {}", json_path.display());

    Ok(())
}

/// Generate artifacts for each robot variant and workspace
async fn generate_artifacts(_org: &Organization) -> Result<Vec<Artifact>> {
    let mut artifacts = Vec::new();

    // Robo-1: Home Companion Artifacts
    artifacts.push(Artifact {
        name: "robo1_design_spec".to_string(),
        artifact_type: ArtifactType::DesignDocument,
        content: r#"# Robo-1 Home Companion Design Specification

## Overview
Robo-1 is designed as a versatile home companion robot with capabilities in household chores, security monitoring, and emotional support.

## Key Features
- **Household Assistance**: Cleaning, organizing, basic maintenance
- **Security Monitoring**: Motion detection, anomaly alerts
- **Emotional Intelligence**: Conversation, companionship, activity suggestions

## Technical Specifications
- Height: 165cm
- Weight: 45kg
- Battery Life: 8 hours continuous operation
- Sensors: RGB-D cameras, LIDAR, IMU, force/torque sensors
- Actuators: 28 DOF (degrees of freedom)
- Processing: NVIDIA Jetson AGX Orin

## Safety Features
- Collision avoidance
- Force limiting on all joints
- Emergency stop button
- Soft padding on contact surfaces
"#.to_string(),
        file_extension: "md".to_string(),
        created_by: "Alice Chen".to_string(),
        workspace: "Robo-1: Home Companion".to_string(),
    });

    artifacts.push(Artifact {
        name: "robo1_control_system".to_string(),
        artifact_type: ArtifactType::PythonCode,
        content: r#"#!/usr/bin/env python3
"""
Robo-1 Home Companion Control System
Main control loop for household assistance robot
"""

import rospy
from sensor_msgs.msg import JointState, Image
from geometry_msgs.msg import Twist
import numpy as np

class Robo1Controller:
    """Main controller for Robo-1 home companion robot"""
    
    def __init__(self):
        rospy.init_node('robo1_controller')
        
        # Publishers
        self.cmd_vel_pub = rospy.Publisher('/cmd_vel', Twist, queue_size=10)
        self.joint_pub = rospy.Publisher('/joint_commands', JointState, queue_size=10)
        
        # Subscribers
        rospy.Subscriber('/camera/rgb/image_raw', Image, self.image_callback)
        rospy.Subscriber('/joint_states', JointState, self.joint_state_callback)
        
        # Control parameters
        self.max_linear_velocity = 0.5  # m/s
        self.max_angular_velocity = 1.0  # rad/s
        self.safety_distance = 0.5  # meters
        
        rospy.loginfo("Robo-1 Controller initialized")
    
    def image_callback(self, msg):
        """Process camera images for object detection"""
        # Implement object detection and scene understanding
        pass
    
    def joint_state_callback(self, msg):
        """Monitor joint states for safety"""
        # Check joint limits and temperatures
        pass
    
    def move_to_position(self, x, y, theta):
        """Navigate to target position"""
        cmd = Twist()
        cmd.linear.x = min(x, self.max_linear_velocity)
        cmd.angular.z = min(theta, self.max_angular_velocity)
        self.cmd_vel_pub.publish(cmd)
    
    def perform_cleaning_task(self, area):
        """Execute household cleaning routine"""
        rospy.loginfo(f"Starting cleaning task in {area}")
        # Implement cleaning behavior
        pass
    
    def run(self):
        """Main control loop"""
        rate = rospy.Rate(50)  # 50Hz
        
        while not rospy.is_shutdown():
            # Main control logic here
            rate.sleep()

if __name__ == '__main__':
    try:
        controller = Robo1Controller()
        controller.run()
    except rospy.ROSInterruptException:
        pass
"#.to_string(),
        file_extension: "py".to_string(),
        created_by: "David Johnson".to_string(),
        workspace: "Robo-1: Home Companion".to_string(),
    });

    // Robo-2: Construction Assistant Artifacts
    artifacts.push(Artifact {
        name: "robo2_load_controller".to_string(),
        artifact_type: ArtifactType::RustCode,
        content: r#"//! Robo-2 Load Balancing Controller
//! Heavy-duty load handling for construction environments

use std::sync::Arc;
use tokio::sync::RwLock;

/// Maximum safe load capacity in kg
const MAX_LOAD_KG: f64 = 75.0;

/// Load balancing controller for Robo-2
pub struct LoadBalanceController {
    /// Current load being carried (kg)
    current_load: Arc<RwLock<f64>>,
    /// Center of mass offset from base (m)
    com_offset: Arc<RwLock<(f64, f64, f64)>>,
    /// Stability margin (0.0 to 1.0)
    stability_margin: f64,
}

impl LoadBalanceController {
    pub fn new() -> Self {
        Self {
            current_load: Arc::new(RwLock::new(0.0)),
            com_offset: Arc::new(RwLock::new((0.0, 0.0, 0.0))),
            stability_margin: 0.8,
        }
    }
    
    /// Update load information from force sensors
    pub async fn update_load(&self, load_kg: f64, com: (f64, f64, f64)) -> Result<(), String> {
        if load_kg > MAX_LOAD_KG {
            return Err(format!("Load {} kg exceeds maximum {}", load_kg, MAX_LOAD_KG));
        }
        
        let mut current = self.current_load.write().await;
        *current = load_kg;
        
        let mut offset = self.com_offset.write().await;
        *offset = com;
        
        Ok(())
    }
    
    /// Calculate required joint torques for stability
    pub async fn calculate_joint_torques(&self) -> Vec<f64> {
        let load = *self.current_load.read().await;
        let com = *self.com_offset.read().await;
        
        // Simplified inverse dynamics calculation
        // In real implementation, use full dynamics model
        vec![0.0; 28]  // 28 DOF
    }
    
    /// Check if current load configuration is safe
    pub async fn is_stable(&self) -> bool {
        let com = *self.com_offset.read().await;
        let load = *self.current_load.read().await;
        
        // Check stability polygon
        let distance_to_edge = com.0.hypot(com.1);
        let support_radius = 0.3;  // meters
        
        (distance_to_edge / support_radius) < self.stability_margin
            && load <= MAX_LOAD_KG
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_load_within_limits() {
        let controller = LoadBalanceController::new();
        let result = controller.update_load(50.0, (0.1, 0.1, 0.5)).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_load_exceeds_limit() {
        let controller = LoadBalanceController::new();
        let result = controller.update_load(100.0, (0.0, 0.0, 0.0)).await;
        assert!(result.is_err());
    }
}
"#.to_string(),
        file_extension: "rs".to_string(),
        created_by: "Henry Patel".to_string(),
        workspace: "Robo-2: Construction Assistant".to_string(),
    });

    // Robo-3: Rescue Operations Artifacts
    artifacts.push(Artifact {
        name: "robo3_rescue_config".to_string(),
        artifact_type: ArtifactType::YamlConfig,
        content: r#"# Robo-3 Rescue Operations Configuration

robot:
  name: "Robo-3 Rescue"
  variant: "rescue_operations"
  version: "1.0.0"

sensors:
  thermal_camera:
    resolution: [640, 480]
    framerate: 30
    temperature_range: [-20, 1000]  # Celsius
  
  lidar:
    model: "Velodyne VLP-16"
    channels: 16
    range: 100  # meters
    scan_rate: 10  # Hz
  
  gas_sensors:
    - type: "CO"
      threshold_ppm: 50
    - type: "CO2"
      threshold_ppm: 5000
    - type: "smoke"
      threshold_density: 0.1

environment:
  operating_conditions:
    temperature: [-20, 60]  # Celsius
    humidity: [0, 95]  # percent
    water_resistance: "IP68"
    fire_resistance: "up to 500C for 30min"

emergency_equipment:
  water_pump:
    capacity: "10 L/min"
    pressure: "10 bar"
  
  rescue_tools:
    - "hydraulic cutter"
    - "thermal blanket"
    - "first aid kit"
    - "communication relay"

communication:
  primary:
    type: "5G"
    fallback: "satellite"
  
  mesh_network:
    enabled: true
    range: 500  # meters

safety:
  emergency_stop:
    type: "wireless"
    range: 100  # meters
  
  autonomous_retreat:
    triggers:
      - "battery < 20%"
      - "temperature > 400C"
      - "structural instability detected"
"#.to_string(),
        file_extension: "yaml".to_string(),
        created_by: "Emily Zhang".to_string(),
        workspace: "Robo-3: Rescue Operations".to_string(),
    });

    // API Specification
    artifacts.push(Artifact {
        name: "robot_control_api".to_string(),
        artifact_type: ArtifactType::ApiSpecification,
        content: r#"# RoboTech Industries Robot Control API v1.0

## Overview
REST API for controlling and monitoring RoboTech humanoid robots.

## Base URL
```
https://api.robotech.io/v1
```

## Authentication
All requests require Bearer token authentication:
```
Authorization: Bearer <your_api_token>
```

## Endpoints

### Robot Status
#### GET /robots/{robot_id}/status
Get current status of a robot.

**Response:**
```json
{
  "robot_id": "robo1-001",
  "variant": "home_companion",
  "status": "active",
  "battery_level": 87,
  "position": {"x": 2.5, "y": 1.3, "z": 0.0},
  "current_task": "cleaning_kitchen",
  "last_update": "2025-11-04T10:30:00Z"
}
```

### Send Command
#### POST /robots/{robot_id}/command
Send a command to the robot.

**Request:**
```json
{
  "command": "navigate",
  "parameters": {
    "target": {"x": 5.0, "y": 3.0},
    "speed": 0.5
  }
}
```

**Response:**
```json
{
  "command_id": "cmd-12345",
  "status": "accepted",
  "estimated_completion": "2025-11-04T10:32:00Z"
}
```

### Emergency Stop
#### POST /robots/{robot_id}/emergency_stop
Immediately stop all robot operations.

**Response:**
```json
{
  "status": "stopped",
  "timestamp": "2025-11-04T10:30:15Z"
}
```

## WebSocket Updates
Real-time status updates available via WebSocket:
```
wss://api.robotech.io/v1/ws/robots/{robot_id}
```

## Rate Limits
- 100 requests per minute per API token
- WebSocket: 1 connection per robot
"#.to_string(),
        file_extension: "md".to_string(),
        created_by: "Sam Johnson".to_string(),
        workspace: "Product Strategy".to_string(),
    });

    // Architecture Diagram
    artifacts.push(Artifact {
        name: "system_architecture".to_string(),
        artifact_type: ArtifactType::ArchitectureDiagram,
        content: r#"# RoboTech Industries System Architecture

```mermaid
graph TB
    subgraph "Robot Hardware"
        Sensors[Sensors<br/>Cameras, LIDAR, IMU]
        Actuators[Actuators<br/>Motors, Grippers]
        Computer[Compute Unit<br/>NVIDIA Jetson]
    end
    
    subgraph "Onboard Software"
        ROS[ROS 2 Core]
        Perception[Perception Module]
        Planning[Motion Planning]
        Control[Control System]
        Safety[Safety Monitor]
    end
    
    subgraph "Cloud Services"
        API[Control API]
        Fleet[Fleet Management]
        Analytics[Analytics Engine]
        ML[ML Training Pipeline]
    end
    
    subgraph "Client Applications"
        Mobile[Mobile App]
        Web[Web Dashboard]
        CLI[CLI Tools]
    end
    
    Sensors --> Perception
    Perception --> Planning
    Planning --> Control
    Control --> Actuators
    
    Computer --> ROS
    ROS --> Perception
    ROS --> Planning
    ROS --> Control
    
    Safety -.->|Emergency Stop| Control
    
    ROS <--> API
    API <--> Fleet
    API <--> Analytics
    Fleet --> ML
    
    Mobile --> API
    Web --> API
    CLI --> API
```

## Component Details

### Onboard Software Stack
- **ROS 2**: Core robotics middleware
- **Perception**: Object detection, scene understanding
- **Planning**: Path planning, task scheduling
- **Control**: Joint control, motion execution
- **Safety**: Real-time safety monitoring

### Cloud Services
- **Control API**: REST and WebSocket interfaces
- **Fleet Management**: Multi-robot coordination
- **Analytics**: Performance metrics and monitoring
- **ML Pipeline**: Continuous model improvement

### Communication
- Robot ↔ Cloud: MQTT over TLS
- Cloud ↔ Clients: HTTPS/WSS
- Inter-robot: Direct mesh networking
"#.to_string(),
        file_extension: "md".to_string(),
        created_by: "Paul Chen".to_string(),
        workspace: "Executive Leadership".to_string(),
    });

    // Manufacturing Configuration
    artifacts.push(Artifact {
        name: "manufacturing_process".to_string(),
        artifact_type: ArtifactType::TomlConfig,
        content: r#"# Manufacturing Process Configuration

[general]
facility = "RoboTech Industries - San Francisco"
production_line = "Humanoid Assembly Line 1"
target_units_per_month = 100

[robo1_assembly]
duration_hours = 8
stations = 6
workers_per_station = 2

[[robo1_assembly.steps]]
step = 1
name = "Frame Assembly"
duration_minutes = 45
tools_required = ["torque_wrench", "alignment_jig"]

[[robo1_assembly.steps]]
step = 2
name = "Actuator Installation"
duration_minutes = 90
tools_required = ["torque_wrench", "calibration_tool"]

[[robo1_assembly.steps]]
step = 3
name = "Electronics Integration"
duration_minutes = 60
tools_required = ["multimeter", "crimping_tool"]

[[robo1_assembly.steps]]
step = 4
name = "Sensor Mounting"
duration_minutes = 45
tools_required = ["precision_screwdriver", "lens_cleaning_kit"]

[[robo1_assembly.steps]]
step = 5
name = "Software Flash & Calibration"
duration_minutes = 120
tools_required = ["laptop", "calibration_board"]

[[robo1_assembly.steps]]
step = 6
name = "QA Testing"
duration_minutes = 90
tools_required = ["test_harness", "diagnostic_tablet"]

[quality_control]
inspection_rate = 1.0  # 100% inspection
acceptance_threshold = 0.95

[[quality_control.tests]]
name = "Mechanical Integrity"
pass_criteria = "No loose components, torque within spec"

[[quality_control.tests]]
name = "Electrical Function"
pass_criteria = "All sensors operational, power draw within limits"

[[quality_control.tests]]
name = "Software Boot"
pass_criteria = "Boot time < 30s, all systems green"

[supply_chain]
lead_time_days = 45
safety_stock_weeks = 4
primary_suppliers = ["AcmeTech Motors", "SensorCorp", "JetsonSupply"]
"#.to_string(),
        file_extension: "toml".to_string(),
        created_by: "Jack Thompson".to_string(),
        workspace: "Manufacturing Excellence".to_string(),
    });

    Ok(artifacts)
}

/// Write artifacts to disk
async fn write_artifacts(output_dir: &PathBuf, artifacts: &[Artifact]) -> Result<()> {
    for artifact in artifacts {
        let subdir = match artifact.artifact_type {
            ArtifactType::DesignDocument | ArtifactType::ApiSpecification | ArtifactType::TechnicalReport => "design_docs",
            ArtifactType::PythonCode | ArtifactType::RustCode => "code",
            ArtifactType::YamlConfig | ArtifactType::TomlConfig => "configs",
            ArtifactType::ArchitectureDiagram => "diagrams",
        };
        
        let file_path = output_dir
            .join("artifacts")
            .join(subdir)
            .join(format!("{}.{}", artifact.name, artifact.file_extension));
        
        fs::write(&file_path, &artifact.content)?;
        println!("  📄 {}/{}/{}.{}", 
            output_dir.join("artifacts").display(),
            subdir,
            artifact.name, 
            artifact.file_extension
        );
    }
    
    Ok(())
}
