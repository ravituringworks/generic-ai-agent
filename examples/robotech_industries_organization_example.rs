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
    println!(
        "   - {}/reports/organization_state.json",
        output_dir.display()
    );
    println!("\n   Artifacts ({} files):", artifacts.len());
    println!(
        "   - {}/artifacts/design_docs/ (design specifications)",
        output_dir.display()
    );
    println!(
        "   - {}/artifacts/code/ (Python, Rust implementations)",
        output_dir.display()
    );
    println!(
        "   - {}/artifacts/configs/ (YAML, TOML configurations)",
        output_dir.display()
    );
    println!(
        "   - {}/artifacts/diagrams/ (architecture diagrams)",
        output_dir.display()
    );
    println!("\n");

    Ok(())
}

/// Create the organization with agents
async fn create_organization() -> Result<Organization> {
    let mut org = Organization::new("RoboTech Industries".to_string());

    // Research & AI Team
    let alice = OrganizationAgent::new(
        "EMP001".to_string(),
        OrganizationRole::ResearchEngineerScaling,
    );
    org.add_agent(alice);

    let bob = OrganizationAgent::new(
        "EMP002".to_string(),
        OrganizationRole::ResearchEngineerAutonomy,
    );
    org.add_agent(bob);

    let carol = OrganizationAgent::new(
        "EMP003".to_string(),
        OrganizationRole::ResearchEngineerWorldModels,
    );
    org.add_agent(carol);

    // Software Engineering Team
    let david = OrganizationAgent::new(
        "EMP004".to_string(),
        OrganizationRole::SoftwareEngineerSimulation,
    );
    org.add_agent(david);

    let emily = OrganizationAgent::new(
        "EMP005".to_string(),
        OrganizationRole::SoftwareEngineerPlatforms,
    );
    org.add_agent(emily);

    let frank = OrganizationAgent::new(
        "EMP006".to_string(),
        OrganizationRole::SoftwareEngineerEmbeddedSystems,
    );
    org.add_agent(frank);

    // Hardware & Robotics Team
    let grace = OrganizationAgent::new(
        "EMP007".to_string(),
        OrganizationRole::HardcoreElectricalEngineer,
    );
    org.add_agent(grace);

    let henry = OrganizationAgent::new(
        "EMP008".to_string(),
        OrganizationRole::SeniorRoboticsEngineerControls,
    );
    org.add_agent(henry);

    let iris = OrganizationAgent::new(
        "EMP009".to_string(),
        OrganizationRole::MechanicalEngineerAllLevels,
    );
    org.add_agent(iris);

    // Manufacturing & Production Team
    let jack = OrganizationAgent::new(
        "EMP010".to_string(),
        OrganizationRole::ManufacturingEngineer,
    );
    org.add_agent(jack);

    let kate = OrganizationAgent::new(
        "EMP011".to_string(),
        OrganizationRole::AutomationEngineerManufacturing,
    );
    org.add_agent(kate);

    let leo = OrganizationAgent::new(
        "EMP012".to_string(),
        OrganizationRole::QualityEngineerManufacturing,
    );
    org.add_agent(leo);

    // Supply Chain & Data Team
    let maya = OrganizationAgent::new("EMP013".to_string(), OrganizationRole::NPIPlanner);
    org.add_agent(maya);

    let noah = OrganizationAgent::new("EMP014".to_string(), OrganizationRole::DataAnalyst);
    org.add_agent(noah);

    // Executive Leadership
    let olivia = OrganizationAgent::new(
        "EMP015".to_string(),
        OrganizationRole::ChiefExecutiveOfficer,
    );
    org.add_agent(olivia);

    let paul = OrganizationAgent::new(
        "EMP016".to_string(),
        OrganizationRole::ChiefTechnologyOfficer,
    );
    org.add_agent(paul);

    let quinn = OrganizationAgent::new("EMP017".to_string(), OrganizationRole::VPEngineering);
    org.add_agent(quinn);

    // Product & Strategy
    let rachel = OrganizationAgent::new(
        "EMP018".to_string(),
        OrganizationRole::ChiefProductOfficer,
    );
    org.add_agent(rachel);

    let sam = OrganizationAgent::new(
        "EMP019".to_string(),
        OrganizationRole::PrincipalProductManager,
    );
    org.add_agent(sam);

    let tina = OrganizationAgent::new(
        "EMP020".to_string(),
        OrganizationRole::TechnicalProgramManager,
    );
    org.add_agent(tina);

    // People & Culture
    let uma = OrganizationAgent::new("EMP021".to_string(), OrganizationRole::DirectorOfPeople);
    org.add_agent(uma);

    // Customer Success & Sales
    let victor = OrganizationAgent::new("EMP022".to_string(), OrganizationRole::VPSales);
    org.add_agent(victor);

    let wendy = OrganizationAgent::new(
        "EMP023".to_string(),
        OrganizationRole::CustomerSuccessManager,
    );
    org.add_agent(wendy);

    let xavier = OrganizationAgent::new(
        "EMP024".to_string(),
        OrganizationRole::SolutionsArchitect,
    );
    org.add_agent(xavier);

    // Marketing & Design
    let yara = OrganizationAgent::new(
        "EMP025".to_string(),
        OrganizationRole::ProductMarketingManager,
    );
    org.add_agent(yara);

    let zack = OrganizationAgent::new(
        "EMP026".to_string(),
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
        "EMP001",    // AI/Autonomy
        "EMP004", // Simulation
        "EMP007",     // Electrical
        "EMP009", // Mechanical
        "EMP019",   // Product Manager
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
        "EMP002",  // Autonomy for load handling
        "EMP008",   // Robotics Controls
        "EMP009", // Mechanical (heavy-duty)
        "EMP010", // Manufacturing
        "EMP013",   // NPI Planning
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
        "EMP003",    // World Models for rescue
        "EMP005",  // Platform software
        "EMP006", // Embedded systems
        "EMP008",  // Advanced controls
        "EMP011",   // Automation
        "EMP012",   // Quality for safety
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
    let agent_names = ["EMP010", "EMP011", "EMP012"];
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
    let agent_names = ["EMP013", "EMP014"];
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
    let agent_names = ["EMP015", "EMP016", "EMP017", "EMP018"];
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
    let agent_names = ["EMP019", "EMP020", "EMP026"];
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
        "EMP022",
        "EMP023",
        "EMP024",
        "EMP025",
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
        vec![get_agent_id("EMP001")],
    )
    .with_priority(TaskPriority::Critical);

    let task2 = WorkspaceTask::new(
        "Build Security & Emotional Intelligence".to_string(),
        "Create security monitoring and emotional companionship capabilities".to_string(),
        vec![get_agent_id("EMP004")],
    )
    .with_priority(TaskPriority::Critical);

    let task3 = WorkspaceTask::new(
        "Design Safe Home-Use Actuators".to_string(),
        "Engineer safe, quiet actuators suitable for home environment".to_string(),
        vec![get_agent_id("EMP007")],
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
        vec![get_agent_id("EMP002")],
    )
    .with_priority(TaskPriority::Critical);

    let task5 = WorkspaceTask::new(
        "Develop Load-Balancing Control System".to_string(),
        "Create controls for stable load handling and construction site navigation".to_string(),
        vec![get_agent_id("EMP008")],
    )
    .with_priority(TaskPriority::Critical);

    let task6 = WorkspaceTask::new(
        "Build Construction Safety Features".to_string(),
        "Implement safety protocols for construction site operations".to_string(),
        vec![get_agent_id("EMP010")],
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
        vec![get_agent_id("EMP003")],
    )
    .with_priority(TaskPriority::Critical);

    let task8 = WorkspaceTask::new(
        "Build Advanced Perception for Rescue".to_string(),
        "Develop AI for victim detection, smoke/water navigation, threat assessment".to_string(),
        vec![get_agent_id("EMP005")],
    )
    .with_priority(TaskPriority::Critical);

    let task9 = WorkspaceTask::new(
        "Implement Emergency Response Protocols".to_string(),
        "Create fail-safe systems and emergency response automation".to_string(),
        vec![get_agent_id("EMP006")],
    )
    .with_priority(TaskPriority::Critical);

    let task10 = WorkspaceTask::new(
        "Design Rescue Equipment Integration".to_string(),
        "Integrate thermal imaging, water pumps, rescue tools, communication systems".to_string(),
        vec![get_agent_id("EMP008")],
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
        vec![get_agent_id("EMP003")],
    )
    .with_priority(TaskPriority::High);

    let task12 = WorkspaceTask::new(
        "Optimize Scaling Algorithms".to_string(),
        "Improve scalability of AI systems for multiple robot variants".to_string(),
        vec![get_agent_id("EMP001")],
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
        vec![get_agent_id("EMP005")],
    )
    .with_priority(TaskPriority::High);

    let task14 = WorkspaceTask::new(
        "Implement Real-Time Simulation Framework".to_string(),
        "Develop simulation tools for testing robot behaviors".to_string(),
        vec![get_agent_id("EMP004")],
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
        vec![get_agent_id("EMP007")],
    )
    .with_priority(TaskPriority::Medium);

    let task16 = WorkspaceTask::new(
        "Standardize Power Management".to_string(),
        "Create unified power management system for all robots".to_string(),
        vec![get_agent_id("EMP009")],
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
        vec![get_agent_id("EMP015"), get_agent_id("EMP016")],
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
        vec![get_agent_id("EMP019"), get_agent_id("EMP018")],
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
        vec![get_agent_id("EMP023")],
    )
    .with_priority(TaskPriority::Low);

    let task20 = WorkspaceTask::new(
        "Develop Training Materials".to_string(),
        "Create comprehensive training for robot operators".to_string(),
        vec![get_agent_id("EMP024")],
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

    // ===== ADVANCED DEVELOPMENT PHASES =====
    println!("\n🏭 PHASE 10: Manufacturing Work Orders (Supplier-Ready)\n");
    println!("==========================================================\n");

    let mfg_task1 = WorkspaceTask::new(
        "Generate CNC Manufacturing Work Orders".to_string(),
        "Create supplier-ready CNC machining work orders with GD&T drawings, material specs (Al 6061-T6), \
        tolerances (±0.001\" for bearing fits), STEP/DXF files, and complete specifications ready for Xometry, \
        Protolabs, or local CNC shops.".to_string(),
        vec![get_agent_id("EMP009")],
    )
    .with_priority(TaskPriority::Critical);

    let mfg_task2 = WorkspaceTask::new(
        "Generate PCB Fabrication Orders".to_string(),
        "Create complete PCB fabrication packages with Gerber files, drill files, BOM in CSV format, \
        pick-and-place files, and assembly notes ready for PCBWay, JLCPCB, or OSH Park.".to_string(),
        vec![get_agent_id("EMP007")],
    )
    .with_priority(TaskPriority::Critical);

    let mfg_ws_id = org
        .workspaces
        .values()
        .find(|w| w.name == "Manufacturing Excellence")
        .map(|w| w.id.clone())
        .unwrap();

    let mfg_tasks = vec![mfg_task1, mfg_task2];
    let mfg_wo_results = coordinator
        .coordinate_workspace_project(&mfg_ws_id, mfg_tasks)
        .await?;

    println!(
        "   ✅ Completed {} manufacturing work order packages\n",
        mfg_wo_results.len()
    );

    // Phase 11: Assembly Procedures
    println!("🔩 PHASE 11: Assembly Procedures & Unit Testing\n");
    println!("==========================================================\n");

    let asm_task1 = WorkspaceTask::new(
        "Generate Mechanical Assembly Procedures".to_string(),
        "Create comprehensive assembly procedures with step-by-step instructions, torque specifications \
        (M4: 2.8 N·m, M6: 8 N·m), bearing installation procedures, alignment checks, and unit testing \
        protocols with acceptance criteria.".to_string(),
        vec![get_agent_id("EMP010")],
    )
    .with_priority(TaskPriority::High);

    let asm_task2 = WorkspaceTask::new(
        "Generate Actuation System Assembly Procedures".to_string(),
        "Create detailed actuation assembly documentation with motor-to-gearbox mounting, encoder calibration, \
        wiring color codes, and unit testing (no-load current <500mA, position control ±1°, thermal test 30min <80°C).".to_string(),
        vec![get_agent_id("EMP008")],
    )
    .with_priority(TaskPriority::High);

    let asm_tasks = vec![asm_task1, asm_task2];
    let asm_results = coordinator
        .coordinate_workspace_project(&mfg_ws_id, asm_tasks)
        .await?;

    println!(
        "   ✅ Completed {} assembly procedure documents\n",
        asm_results.len()
    );

    // Phase 12: Validation Testing
    println!("🧪 PHASE 12: Validation & Performance Testing\n");
    println!("==========================================================\n");

    let val_task1 = WorkspaceTask::new(
        "Generate System Validation Test Plan".to_string(),
        "Create comprehensive system validation with test environment setup, pre-test checklists, \
        system bring-up procedures, subsystem integration tests (power, communications, sensors), \
        and safety system validation (emergency stop <100ms)."
            .to_string(),
        vec![get_agent_id("EMP012")],
    )
    .with_priority(TaskPriority::Critical);

    let val_task2 = WorkspaceTask::new(
        "Generate Locomotion Testing Protocol".to_string(),
        "Create detailed locomotion testing with progressive sequence (static balance 30s, weight shifting, \
        single-leg stance 10s, forward walking 1 m/s, obstacle negotiation), performance metrics (step length, \
        stride frequency, energy consumption), and failure mode testing.".to_string(),
        vec![get_agent_id("EMP008")],
    )
    .with_priority(TaskPriority::Critical);

    let val_tasks = vec![val_task1, val_task2];
    let val_results = coordinator
        .coordinate_workspace_project(&platform_ws_id, val_tasks)
        .await?;

    println!(
        "   ✅ Completed {} validation test protocols\n",
        val_results.len()
    );

    // Phase 13: Software Development
    println!("💻 PHASE 13: Software Development & Control Systems\n");
    println!("==========================================================\n");

    let sw_task1 = WorkspaceTask::new(
        "Develop ROS2 Software Architecture".to_string(),
        "Create complete ROS2 workspace with robot_description, robot_bringup, robot_control packages. \
        Include URDF/XACRO files with complete kinematic chain, launch files for simulation/hardware, \
        parameter files (PID gains, sensor configs), and Docker container setup.".to_string(),
        vec![get_agent_id("EMP004"), get_agent_id("EMP005")],
    )
    .with_priority(TaskPriority::Critical);

    let sw_task2 = WorkspaceTask::new(
        "Implement Low-Level Control Algorithms".to_string(),
        "Develop real-time control algorithms: PID position/velocity/torque control (500Hz loop), \
        inverse kinematics solvers, balance controller (ZMP, LIPM, CoM planning), gait generation \
        (footstep planning, phase state machine), and compliance control for manipulation."
            .to_string(),
        vec![get_agent_id("EMP008"), get_agent_id("EMP006")],
    )
    .with_priority(TaskPriority::Critical);

    let sw_task3 = WorkspaceTask::new(
        "Develop Perception Pipeline".to_string(),
        "Create perception software stack with sensor drivers (RealSense, IMU, LIDAR), sensor fusion (EKF), \
        computer vision pipeline (YOLO, semantic segmentation, depth processing), SLAM implementation, \
        and GPU acceleration (CUDA/TensorRT).".to_string(),
        vec![get_agent_id("EMP001"), get_agent_id("EMP002")],
    )
    .with_priority(TaskPriority::Critical);

    let sw_task4 = WorkspaceTask::new(
        "Develop AI/ML Models".to_string(),
        "Create ML models for behaviors: MoveIt2 integration, RL framework (PPO/SAC algorithms), \
        imitation learning, vision-based learning (grasp pose estimation), model deployment (ONNX/TensorRT), \
        and training infrastructure.".to_string(),
        vec![get_agent_id("EMP003"), get_agent_id("EMP001")],
    )
    .with_priority(TaskPriority::High);

    let sw_tasks = vec![sw_task1, sw_task2, sw_task3, sw_task4];
    let sw_results = coordinator
        .coordinate_workspace_project(&platform_ws_id, sw_tasks)
        .await?;

    println!(
        "   ✅ Completed {} software development modules\n",
        sw_results.len()
    );

    // Phase 14: Documentation & Handoff
    println!("📚 PHASE 14: Documentation & Project Handoff\n");
    println!("==========================================================\n");

    let doc_task1 = WorkspaceTask::new(
        "Generate Technical Documentation Package".to_string(),
        "Create comprehensive technical documentation: system architecture diagrams, design rationale, \
        consolidated BOM with suppliers, assembly documentation, test results and validation data, \
        change log, and safety documentation (FMEA, risk assessment). Export as PDF with searchable text.".to_string(),
        vec![get_agent_id("EMP020"), get_agent_id("EMP017")],
    )
    .with_priority(TaskPriority::Critical);

    let doc_task2 = WorkspaceTask::new(
        "Generate User Operation Manual".to_string(),
        "Create user-friendly operation manual with quick start guide, operation modes (manual/semi-autonomous/autonomous), \
        safety procedures, basic troubleshooting, operational limits, and software interface guide. \
        Export as PDF and interactive HTML.".to_string(),
        vec![get_agent_id("EMP023"), get_agent_id("EMP024")],
    )
    .with_priority(TaskPriority::High);

    let doc_task3 = WorkspaceTask::new(
        "Generate Maintenance and Service Manual".to_string(),
        "Create detailed maintenance manual with preventive maintenance schedule (daily/weekly/monthly/annual), \
        component replacement procedures, lubrication guide, diagnostic procedures, calibration procedures, \
        spare parts list, and safety for technicians. Export as PDF with laminated quick-reference sheets.".to_string(),
        vec![get_agent_id("EMP012"), get_agent_id("EMP010")],
    )
    .with_priority(TaskPriority::High);

    let doc_tasks = vec![doc_task1, doc_task2, doc_task3];
    let doc_results = coordinator
        .coordinate_workspace_project(&prod_ws_id, doc_tasks)
        .await?;

    println!(
        "   ✅ Completed {} documentation packages\n",
        doc_results.len()
    );

    // Phase 15: Regulatory & Compliance
    println!("\n⚖️ PHASE 15: Regulatory & Compliance\n");
    println!("==========================================================\n");

    let reg_task1 = WorkspaceTask::new(
        "Generate Safety Certification Package".to_string(),
        "Create comprehensive safety certification documentation for UL, CE, FCC compliance. Include: \
        safety analysis (FMEA, FTA), hazard identification, risk assessment matrix, safety requirements traceability, \
        test plans for electrical safety (UL 60950, IEC 62368), EMC testing (FCC Part 15, EN 55032), \
        mechanical safety (ISO 13849, ISO 12100), functional safety analysis, safety test reports, \
        declaration of conformity templates, and technical construction files for CE marking.".to_string(),
        vec![get_agent_id("EMP012"), get_agent_id("EMP017")],
    )
    .with_priority(TaskPriority::Critical);

    let reg_task2 = WorkspaceTask::new(
        "Generate IP Protection Strategy".to_string(),
        "Create intellectual property protection package including: patent landscape analysis, \
        patentability search results, provisional/utility patent applications for key innovations \
        (novel actuation mechanisms, control algorithms, sensor fusion methods), trade secret identification \
        and protection procedures, trademark registrations (brand, logos), copyright protection for software, \
        IP licensing strategy, freedom-to-operate analysis, and defensive publication strategy.".to_string(),
        vec![get_agent_id("EMP016"), get_agent_id("EMP018")],
    )
    .with_priority(TaskPriority::High);

    let reg_task3 = WorkspaceTask::new(
        "Generate Environmental Compliance Package".to_string(),
        "Create environmental compliance documentation including: RoHS compliance declaration, \
        material disclosure statements, restricted substance testing reports (lead, mercury, cadmium, hexavalent chromium), \
        REACH SVHC declaration, conflict minerals reporting, WEEE compliance and recycling procedures, \
        packaging material declarations, battery disposal procedures (EPA, state regulations), \
        California Prop 65 warnings if applicable, and supplier environmental compliance verification.".to_string(),
        vec![get_agent_id("EMP013"), get_agent_id("EMP012")],
    )
    .with_priority(TaskPriority::High);

    let reg_tasks = vec![reg_task1, reg_task2, reg_task3];
    let reg_results = coordinator
        .coordinate_workspace_project(&prod_ws_id, reg_tasks)
        .await?;

    println!(
        "   ✅ Completed {} regulatory compliance packages\n",
        reg_results.len()
    );

    // Phase 16: Environmental & Durability Testing
    println!("🌡️ PHASE 16: Environmental & Durability Testing\n");
    println!("==========================================================\n");

    let env_task1 = WorkspaceTask::new(
        "Generate Environmental Test Plan".to_string(),
        "Create comprehensive environmental testing protocol including: temperature cycling (-40°C to +85°C, \
        MIL-STD-810 or IEC 60068-2-14), thermal shock testing, humidity testing (85% RH at 85°C, IEC 60068-2-78), \
        salt spray corrosion testing (ASTM B117, 48-96 hours), altitude testing (low pressure), sand and dust \
        ingress testing (IP6X), water ingress testing (IPX7/IPX8), UV exposure and weathering (ASTM G154), \
        test setup procedures, acceptance criteria, data collection templates, and failure analysis procedures.".to_string(),
        vec![get_agent_id("EMP012"), get_agent_id("EMP009")],
    )
    .with_priority(TaskPriority::Critical);

    let env_task2 = WorkspaceTask::new(
        "Generate Mechanical Durability Test Plan".to_string(),
        "Create mechanical durability testing protocol including: vibration testing (random, sinusoidal, \
        MIL-STD-810 or IEC 60068-2-64), mechanical shock testing (half-sine, saw-tooth, IEC 60068-2-27), \
        drop testing (various heights, orientations, surfaces), repetitive stress testing (joint cycling 100k+ cycles), \
        wear testing for contact surfaces, fatigue analysis and accelerated life testing, packaging drop testing \
        (ISTA procedures), transportation simulation, acceptance criteria based on functional requirements, \
        and MTBF/MTTF calculation methodology.".to_string(),
        vec![get_agent_id("EMP009"), get_agent_id("EMP008")],
    )
    .with_priority(TaskPriority::Critical);

    let env_task3 = WorkspaceTask::new(
        "Generate EMI/EMC Test Plan".to_string(),
        "Create electromagnetic compatibility testing protocol including: conducted emissions (FCC Part 15B, \
        CISPR 32), radiated emissions (30 MHz - 6 GHz), conducted immunity (IEC 61000-4-6), radiated immunity \
        (IEC 61000-4-3, 80 MHz - 6 GHz, 3-10 V/m), ESD immunity (IEC 61000-4-2, contact/air discharge), \
        electrical fast transient/burst (IEC 61000-4-4), surge immunity (IEC 61000-4-5), power frequency magnetic field \
        (IEC 61000-4-8), test lab selection and scheduling, pre-compliance testing procedures, and remediation strategies.".to_string(),
        vec![get_agent_id("EMP007"), get_agent_id("EMP006")],
    )
    .with_priority(TaskPriority::High);

    let env_tasks = vec![env_task1, env_task2, env_task3];
    let env_results = coordinator
        .coordinate_workspace_project(&hw_ws_id, env_tasks)
        .await?;

    println!(
        "   ✅ Completed {} environmental test plans\n",
        env_results.len()
    );

    // Phase 17: Production Scaling
    println!("🏭 PHASE 17: Production Scaling & Manufacturing Ramp\n");
    println!("==========================================================\n");

    let scale_task1 = WorkspaceTask::new(
        "Generate Pilot Production Plan".to_string(),
        "Create pilot production (10-100 units) plan including: pilot run objectives and success criteria, \
        production line layout and workstation design, takt time analysis and capacity planning, \
        assembly sequence optimization, tooling and fixture requirements, worker training curriculum and certification, \
        quality control checkpoints (in-process inspection, final QA), yield analysis and defect tracking (Pareto analysis), \
        design for manufacturing (DFM) recommendations, cost analysis (labor, material, overhead), \
        lessons learned documentation, and readiness criteria for volume production.".to_string(),
        vec![get_agent_id("EMP010"), get_agent_id("EMP011")],
    )
    .with_priority(TaskPriority::Critical);

    let scale_task2 = WorkspaceTask::new(
        "Generate Volume Manufacturing Plan".to_string(),
        "Create volume manufacturing (1000+ units) plan including: production capacity analysis (units/month), \
        manufacturing automation strategy (pick-and-place machines, AOI, automated test equipment), \
        line balancing and bottleneck analysis, inventory management strategy (JIT, safety stock levels), \
        supplier qualification and dual-sourcing strategy, statistical process control (SPC) implementation \
        (Cp, Cpk targets), lean manufacturing initiatives (5S, kaizen, waste reduction), production scheduling \
        and MRP system, quality management system (ISO 9001), continuous improvement program, and cost reduction roadmap.".to_string(),
        vec![get_agent_id("EMP011"), get_agent_id("EMP013")],
    )
    .with_priority(TaskPriority::Critical);

    let scale_task3 = WorkspaceTask::new(
        "Generate Supplier Management Strategy".to_string(),
        "Create supplier management framework including: supplier selection criteria and scorecard, \
        approved vendor list (AVL) with qualification status, supplier audit procedures and checklists, \
        quality agreements and SLAs, supplier performance monitoring (on-time delivery, quality PPM, responsiveness), \
        supplier development programs, cost reduction partnerships (should-cost modeling), \
        supply chain risk assessment (single-source risks, geopolitical risks, natural disaster contingencies), \
        second-source development timeline, contract manufacturing (CM) evaluation if applicable, \
        and supplier relationship management (SRM) procedures.".to_string(),
        vec![get_agent_id("EMP013"), get_agent_id("EMP010")],
    )
    .with_priority(TaskPriority::High);

    let scale_tasks = vec![scale_task1, scale_task2, scale_task3];
    let scale_results = coordinator
        .coordinate_workspace_project(&hw_ws_id, scale_tasks)
        .await?;

    println!(
        "   ✅ Completed {} production scaling plans\n",
        scale_results.len()
    );

    // Phase 18: Field Deployment & Operations
    println!("🚀 PHASE 18: Field Deployment & Operations\n");
    println!("==========================================================\n");

    let deploy_task1 = WorkspaceTask::new(
        "Generate Deployment & Installation Procedures".to_string(),
        "Create field deployment package including: pre-shipment inspection checklist, \
        packaging specifications (foam inserts, anti-static protection, shock indicators), \
        shipping procedures and carrier selection (temperature-controlled if needed), \
        customs documentation and HS codes, installation procedures (site assessment, unpacking, assembly), \
        commissioning checklist (power-on sequence, network configuration, sensor calibration verification), \
        customer acceptance testing (CAT) procedures, installation troubleshooting guide, \
        field service technician training materials, and installation time estimates.".to_string(),
        vec![get_agent_id("EMP024"), get_agent_id("EMP010")],
    )
    .with_priority(TaskPriority::High);

    let deploy_task2 = WorkspaceTask::new(
        "Generate Fleet Management System Design".to_string(),
        "Create fleet management platform architecture including: cloud infrastructure design (AWS/Azure/GCP), \
        robot telemetry data collection (health metrics, performance KPIs, usage patterns), \
        real-time monitoring dashboard (status, location, alerts), remote diagnostics and log collection, \
        OTA software update system (staged rollout, rollback capability), fleet-wide analytics and reporting, \
        predictive maintenance algorithms (anomaly detection, failure prediction), multi-tenancy architecture \
        for multiple customers, data security and privacy controls, API design for third-party integrations, \
        and scalability planning (support 10k+ robots).".to_string(),
        vec![get_agent_id("EMP005"), get_agent_id("EMP004")],
    )
    .with_priority(TaskPriority::Critical);

    let deploy_task3 = WorkspaceTask::new(
        "Generate Customer Support Infrastructure".to_string(),
        "Create customer support framework including: helpdesk system setup (Zendesk, Salesforce Service Cloud), \
        tiered support structure (L1: basic troubleshooting, L2: advanced technical, L3: engineering escalation), \
        support SLAs (response time: 4 hours critical, 24 hours high, resolution time targets), \
        knowledge base and FAQ documentation, ticketing workflow and escalation procedures, \
        remote support tools (VPN access, screen sharing), warranty claim procedures and RMA process, \
        spare parts inventory planning (critical components, lead times), field service dispatch system, \
        customer satisfaction tracking (CSAT, NPS), and support cost analysis.".to_string(),
        vec![get_agent_id("EMP023"), get_agent_id("EMP024")],
    )
    .with_priority(TaskPriority::High);

    let deploy_tasks = vec![deploy_task1, deploy_task2, deploy_task3];
    let deploy_results = coordinator
        .coordinate_workspace_project(&cust_ws_id, deploy_tasks)
        .await?;

    println!(
        "   ✅ Completed {} deployment and operations packages\n",
        deploy_results.len()
    );

    // Phase 19: Cybersecurity
    println!("🔒 PHASE 19: Cybersecurity & Data Protection\n");
    println!("==========================================================\n");

    let cyber_task1 = WorkspaceTask::new(
        "Generate Cybersecurity Threat Model".to_string(),
        "Create comprehensive threat modeling and security analysis including: asset identification \
        (robot firmware, sensor data, customer data, cloud services), threat actor profiling (nation states, \
        hacktivists, criminals, insiders), attack surface analysis (network interfaces, USB ports, wireless protocols), \
        attack tree analysis (denial of service, unauthorized control, data exfiltration, tampering), \
        STRIDE threat modeling (Spoofing, Tampering, Repudiation, Information Disclosure, Denial of Service, \
        Elevation of Privilege), risk scoring (likelihood × impact), threat mitigation strategies, \
        security requirements traceability matrix, and incident response scenarios.".to_string(),
        vec![get_agent_id("EMP006"), get_agent_id("EMP005")],
    )
    .with_priority(TaskPriority::Critical);

    let cyber_task2 = WorkspaceTask::new(
        "Generate Security Architecture & Implementation".to_string(),
        "Create security architecture design including: secure boot implementation (signed bootloader, verified firmware), \
        cryptographic key management (TPM/secure element, key hierarchy, rotation policies), \
        secure communication protocols (TLS 1.3, certificate management, mutual authentication), \
        network security architecture (firewall rules, network segmentation, VPN for remote access), \
        access control and authentication (multi-factor authentication, role-based access control), \
        secure software update mechanism (signed updates, anti-rollback protection), \
        data encryption (at-rest: AES-256, in-transit: TLS), intrusion detection/prevention system (IDS/IPS), \
        security logging and audit trails, and vulnerability management process.".to_string(),
        vec![get_agent_id("EMP006"), get_agent_id("EMP004")],
    )
    .with_priority(TaskPriority::Critical);

    let cyber_task3 = WorkspaceTask::new(
        "Generate Security Testing & Compliance Plan".to_string(),
        "Create security validation framework including: penetration testing procedures (OWASP methodology, \
        automated scanners, manual testing), fuzzing test plans for input validation, code security review \
        (static analysis tools: SonarQube, Coverity; manual code review), vulnerability scanning (Nessus, Qualys), \
        security regression testing, red team exercises, bug bounty program design, privacy impact assessment (PIA), \
        GDPR/CCPA compliance documentation (data inventory, processing activities, privacy notices, data subject rights), \
        security certification (IEC 62443 for industrial systems if applicable), incident response plan \
        (detection, containment, eradication, recovery, lessons learned), and security awareness training for developers.".to_string(),
        vec![get_agent_id("EMP005"), get_agent_id("EMP017")],
    )
    .with_priority(TaskPriority::High);

    let cyber_tasks = vec![cyber_task1, cyber_task2, cyber_task3];
    let cyber_results = coordinator
        .coordinate_workspace_project(&platform_ws_id, cyber_tasks)
        .await?;

    println!(
        "   ✅ Completed {} cybersecurity packages\n",
        cyber_results.len()
    );

    // Phase 20: Compute Platform Analysis & Multi-Vendor Sourcing
    println!("💻 PHASE 20: Compute Platform Analysis & Multi-Vendor Hardware Sourcing\n");
    println!("==========================================================\n");

    let hw_task1 = WorkspaceTask::new(
        "Generate Compute Platform Trade-off Analysis".to_string(),
        "Create comprehensive compute platform comparison and trade-off analysis. Include: \
        - Main compute options: NVIDIA Jetson family (AGX Thor: $2499, 2000 TOPS, 100W AI accelerator with Grace CPU, shipping 2025; \
        AGX Orin 64GB: $1999, 275 TOPS, 60W; Orin NX 16GB: $699, 100 TOPS, 25W; Orin Nano 8GB: $499, 40 TOPS, 15W), \
        Intel Core Ultra 9 Series (185H: $1000-1500, with dedicated AI NPU 30 TOPS), AMD Ryzen AI 9 HX 370 ($800-1200, 50 TOPS NPU), \
        Qualcomm Snapdragon X Elite ($600-900, 45 TOPS NPU), Apple M4 (for dev comparison, not robotics), Google Coral + Hailo-8L ($450 total). \
        - Performance benchmarks: AI inference throughput (TOPS), latency, frame rates for vision tasks, transformer performance. \
        - Power analysis: thermal design, battery runtime impact, cooling requirements, peak vs sustained performance. \
        - Cost analysis: unit cost, volume pricing (100+, 1000+), development kit costs, ecosystem costs. \
        - Software ecosystem: SDK maturity, library support (TensorRT, OpenVINO, TFLite, ONNX Runtime), ease of development, ROS2 compatibility. \
        - Supply chain: lead times, availability, EOL roadmaps, multi-sourcing options. \
        - Recommendation matrix: flagship config with Thor ($2599), high-performance config ($2090), mid-range config ($754), budget config ($163). \
        Include performance/watt and performance/dollar charts, 2025 market positioning.".to_string(),
        vec![get_agent_id("EMP004"), get_agent_id("EMP007")],
    )
    .with_priority(TaskPriority::Critical);

    let hw_task2 = WorkspaceTask::new(
        "Generate Microcontroller Selection & Sourcing Matrix".to_string(),
        "Create microcontroller selection guide and multi-vendor sourcing strategy. Include: \
        - MCU options for motor control: STM32H7 (550MHz, $15-25), STM32U5 (ultra-low power, $8-18), STM32G4 (motor-focused, $5-10), \
        NXP i.MX RT1180 (1.2GHz, dual Cortex-M7+M33, $12-20), NXP MCXN947 (150MHz, $6-12), TI TMS320F28P65 (200MHz DSP, $18-30), \
        RP2350 (dual Cortex-M33 + RISC-V, $0.80-1.50, 2024 release), ESP32-C6 (RISC-V, WiFi 6, $2-4). \
        - MCU options for I/O: RP2040/RP2350 (dual-core, excellent for I/O), STM32F1 (legacy, $3-5), STM32C0 (entry-level, $0.50-2), \
        Teensy 4.1 ($30, prototyping), Arduino Portenta C33 ($25-35, dual-core). \
        - Selection criteria: real-time performance (control loop frequency), peripheral count (timers, ADCs, CAN-FD/EtherCAT), \
        development tools (free IDEs, debugger costs), community support, security features (TrustZone, secure boot). \
        - Configuration recommendations: High-perf (4× STM32H7 + 2× RP2350 + STM32G4), Mid-range (4× STM32U5 + 2× RP2350 + NXP MCXN947), \
        Budget (4× RP2350 + 2× ESP32-C6 + STM32C0). \
        - Dual-sourcing strategy: Primary (Mouser), Secondary (DigiKey), Tertiary (Newark/Avnet), Direct (Raspberry Pi for RP2xxx). \
        - Lead time analysis and inventory planning (safety stock for long-lead items, 2025 availability updates). \
        Include pinout compatibility matrix for drop-in replacements and 2025 supply chain updates.".to_string(),
        vec![get_agent_id("EMP006"), get_agent_id("EMP013")],
    )
    .with_priority(TaskPriority::Critical);

    let hw_task3 = WorkspaceTask::new(
        "Generate Second-Source Qualification Plan".to_string(),
        "Create supplier qualification and second-sourcing procedures. Include: \
        - Supplier qualification criteria: quality certifications (ISO 9001, AS9100, IATF 16949), financial stability (Dun & Bradstreet rating), \
        technical support responsiveness, RMA/warranty terms, conflict minerals compliance, sustainability certifications (RoHS, REACH). \
        - Dual-sourcing matrix by component category: Jetson modules including Thor (NVIDIA Direct/Partner Network, DigiKey, Arrow, Mouser), \
        STM32 MCUs (Mouser primary, DigiKey secondary, Newark tertiary, direct from STMicroelectronics for volume), \
        RP2040/RP2350 (Raspberry Pi Direct, DigiKey, Adafruit, SparkFun), \
        Connectors (Molex via McMaster-Carr, TE Connectivity via DigiKey, JST via Mouser, Hirose via Newark), \
        Passives (Yageo/DigiKey, Vishay/Mouser, Samsung/Arrow, Murata/DigiKey for MLCC). \
        - Second-source validation: electrical equivalence testing, software compatibility verification, reliability testing (sample lot evaluation), \
        thermal characterization, EMC/EMI compliance validation. \
        - Supply chain risk mitigation: geopolitical risk assessment (2025 tariff implications), natural disaster contingencies, \
        single-source risk identification, strategic inventory for critical components (6-12 month buffer for Jetson Thor, 3-6 months for MCUs). \
        - Supplier performance monitoring: on-time delivery KPIs, quality PPM targets (<100 PPM), cost competitiveness reviews (quarterly), \
        technology roadmap alignment reviews. \
        - Transition plan: when to trigger second-source activation (lead time >12 weeks, price increase >15%, quality issues), \
        qualification timeline (8-12 weeks for new Jetson Thor, 6-8 weeks for established MCUs). \
        Include approved vendor list (AVL) template with qualification status and 2025 supply chain resilience metrics.".to_string(),
        vec![get_agent_id("EMP013"), get_agent_id("EMP012")],
    )
    .with_priority(TaskPriority::High);

    let hw_task4 = WorkspaceTask::new(
        "Generate Hardware Configuration Management Strategy".to_string(),
        "Create variant BOM management and configuration control system. Include: \
        - Hardware variant definitions: Flagship config (Jetson AGX Thor, STM32H7, $2599 compute, 2025 latest), \
        Premium config (Jetson AGX Orin 64GB, STM32H7, $2090 compute), \
        Standard config (Jetson Orin NX 16GB, STM32U5, $754 compute), Economy config (RPi5 + Coral, RP2350, $163 compute). \
        - BOM management system: PLM tool selection (Arena PLM, Fusion 360 Manage, PTC Windchill, or Odoo), part numbering scheme, \
        revision control (ECO process), where-used analysis, obsolescence tracking, supplier lifecycle management. \
        - Configuration management: variant part matrix, build configurations (Robo-1/2/3 with different compute tiers), \
        option codes (e.g., R1-FLG-THR = Robo-1, Flagship, Jetson Thor; R1-STD-JNX = Robo-1, Standard, Jetson NX). \
        - Change control process: Engineering Change Order (ECO) workflow, change impact analysis, validation requirements, \
        customer notification procedures for product updates, regulatory compliance tracking (FCC, CE, UL). \
        - Cost rollup by configuration: material cost, assembly labor, test time, warranty reserves, volume discounts (100+, 1000+, 10K+). \
        - Supplier part cross-reference: alternate parts matrix (form-fit-function equivalents), preferred parts list, \
        lifecycle status (active, NRND, obsolete), 2025 semiconductor shortage mitigation. \
        - Software compatibility matrix: which firmware versions support which hardware configs, backward compatibility strategy, \
        Jetson Thor software requirements (JetPack 7.x+). \
        Include configuration control board (CCB) charter, ECO template, and 2025 hardware roadmap alignment.".to_string(),
        vec![get_agent_id("EMP020"), get_agent_id("EMP017")],
    )
    .with_priority(TaskPriority::High);

    let hw_tasks = vec![hw_task1, hw_task2, hw_task3, hw_task4];
    let hw_platform_results = coordinator
        .coordinate_workspace_project(&hw_ws_id, hw_tasks)
        .await?;

    println!(
        "   ✅ Completed {} hardware platform analysis packages\n",
        hw_platform_results.len()
    );

    // Phase 21: Alternative Architecture Buildout Plans
    println!("🏗️ PHASE 21: Alternative Architecture Buildout Plans\n");
    println!("==========================================================\n");

    let arch_task1 = WorkspaceTask::new(
        "Generate Flagship Architecture Buildout Plan (Jetson AGX Thor - 2025)".to_string(),
        "Create complete buildout guide for flagship 2025 configuration. Include: \
        - Hardware architecture: NVIDIA Jetson AGX Thor ($2499, 2000 TOPS, Grace CPU + Blackwell GPU), 4× STM32H7 ($80), \
        2× RP2350 ($3), STM32G4 ($8), Total: $2590. \
        - Detailed assembly guide: Thor carrier board (official dev kit, custom carrier availability 2025), advanced thermal solution \
        (dual-fan active cooling with vapor chamber, 120mm radiator option), power supply (48V 10A PoE++ or barrel, USB-C PD 240W), \
        PCIe Gen5 NVMe storage (2TB Samsung 990 PRO), optional 10GbE network card. \
        - MCU interconnect topology: Primary CAN-FD bus (5 Mbps) for motor controllers, Secondary CAN-FD (2 Mbps) for sensors, \
        EtherCAT for high-speed joint control (1 GHz cycle), TSN (Time-Sensitive Networking) for deterministic comms, \
        I2C for auxiliary sensors, SPI for high-speed IMU arrays. \
        - Software stack: JetPack 7.x installation (Thor-optimized), ROS2 Jazzy/Rolling setup, TensorRT 10+ for transformer optimization, \
        CUDA 13+ with Blackwell features, STM32CubeIDE 1.15+ for MCU, real-time kernel (PREEMPT_RT or Xenomai). \
        - Performance targets: Vision at 60 FPS (YOLO v10, ViT transformers), Multi-modal AI (vision+language), Motor control at 2 kHz, \
        AI inference <20ms latency for large models, Power budget: 100W peak, 60W average. \
        - Development workflow: Docker/Podman containers, VS Code with CUDA 13 debugging, Nsight Systems profiling, \
        OpenOCD for STM32, pytest + ROS2 test frameworks. \
        - BOM with US suppliers: Complete parts list with DigiKey, Mouser, Arrow part numbers and 2025 lead times. \
        Include step-by-step bring-up procedure, Thor-specific validation checklist, and migration guide from Orin.".to_string(),
        vec![get_agent_id("EMP004"), get_agent_id("EMP007")],
    )
    .with_priority(TaskPriority::Critical);

    let arch_task2 = WorkspaceTask::new(
        "Generate Premium Architecture Buildout Plan (Jetson AGX Orin - 2025 Optimized)".to_string(),
        "Create complete buildout guide for premium configuration. Include: \
        - Hardware architecture: NVIDIA Jetson AGX Orin 64GB ($1999), 4× STM32H7 ($80), 2× RP2350 ($3), STM32G4 ($8), Total: $2090. \
        - Detailed assembly guide: Orin carrier board (official or ConnectTech Rogue-X), active cooling \
        (40mm Noctua fan, copper heat sink), power supply (19V 6.3A barrel or USB-C PD 100W), M.2 NVMe storage (1TB Samsung 990 EVO). \
        - MCU interconnect topology: Primary CAN-FD bus (2 Mbps) for motor controllers, Secondary CAN (500 kbps) for sensors, \
        EtherCAT for high-speed joint control (100 MHz cycle), I2C for auxiliary sensors, SPI for high-speed IMU. \
        - Software stack: JetPack 6.x installation, ROS2 Jazzy setup, TensorRT 9+ for model optimization, \
        CUDA 12/cuDNN for custom kernels, STM32CubeIDE 1.15+ for MCU, real-time kernel (PREEMPT_RT patch). \
        - Performance targets: Vision at 30 FPS (YOLO v8/v9, semantic segmentation), Motor control at 1 kHz, \
        AI inference <50ms latency, Power budget: 60W peak, 35W average. \
        - Development workflow: Docker containers for reproducible builds, VS Code with CUDA debugging, \
        OpenOCD/ST-Link for STM32, pytest + gtest frameworks. \
        - BOM with US suppliers: Complete parts list with DigiKey, Mouser, Arrow part numbers and 2025 lead times. \
        Include step-by-step bring-up procedure, validation checklist, and Thor upgrade path.".to_string(),
        vec![get_agent_id("EMP005"), get_agent_id("EMP006")],
    )
    .with_priority(TaskPriority::Critical);

    let arch_task3 = WorkspaceTask::new(
        "Generate Standard Architecture Buildout Plan (Jetson Orin NX - 2025)".to_string(),
        "Create complete buildout guide for standard configuration. Include: \
        - Hardware architecture: NVIDIA Jetson Orin NX 16GB ($699), 4× STM32U5 ($52), 2× RP2350 ($3), NXP MCXN947 ($8), Total: $762. \
        - Detailed assembly guide: Orin NX on official carrier or Seeed Studio Recomputer J401, passive cooling (low-profile heatsink), \
        power via barrel jack (12V 5A) or PoE+ (802.3at), microSD card (256GB UHS-I) or optional M.2 2242 NVMe. \
        - MCU interconnect topology: Single CAN-FD bus (1 Mbps) for motor control, I2C for sensors, UART for debug/telemetry, \
        GPIO for safety interlocks (emergency stop, limit switches), USB-C for RP2350 programming. \
        - Software stack: JetPack 6.x, ROS2 Jazzy, TensorRT 9+ for optimized models, OpenCV with CUDA acceleration, \
        STM32CubeIDE for MCU, standard Linux 6.x kernel (not RT, but low-latency config). \
        - Performance targets: Vision at 25 FPS (MobileNet v3, EfficientDet), Motor control at 500 Hz, AI inference <80ms, Power: 25W peak, 15W average. \
        - Cost optimization: MicroPython on RP2350 for rapid I/O prototyping, leverage ROS2 Nav2/MoveIt packages, \
        use quantized INT8 models (TensorRT), community pre-trained weights. \
        - Development workflow: Native development on Jetson, Thonny/Arduino IDE for RP2350, STM32CubeMX code generation. \
        - BOM with cost breakdown and volume pricing (100+, 1000+ unit discounts). \
        Include quick-start guide, common troubleshooting, and upgrade paths.".to_string(),
        vec![get_agent_id("EMP005"), get_agent_id("EMP006")],
    )
    .with_priority(TaskPriority::High);

    let arch_task4 = WorkspaceTask::new(
        "Generate Budget Architecture Buildout Plan (Raspberry Pi 5 + AI Accelerator)".to_string(),
        "Create complete buildout guide for budget configuration. Include: \
        - Hardware architecture: Raspberry Pi 5 8GB ($80), Google Coral USB ($60) or Hailo-8L M.2 ($299), \
        4× RP2350 ($6), 2× ESP32-C6 ($6), STM32C0 ($1), Total: $153-392 depending on AI accelerator. \
        - Detailed assembly guide: RPi5 with official active cooler ($5), USB-C PD (5V 5A, 27W official PSU), NVMe via M.2 HAT+ \
        ($12 + $40 for 256GB WD SN740), Coral USB or Hailo-8L on M.2 slot, RP2350 on breadboard or custom PCB. \
        - MCU interconnect topology: USB-to-CAN adapter (PEAK PCAN-USB, $150) for motor control, I2C over GPIO for sensors, \
        UART for RP2350 communication, SPI for peripherals, WiFi 6 via ESP32-C6 for wireless telemetry. \
        - Software stack: Raspberry Pi OS 64-bit (Debian Bookworm 12), ROS2 Jazzy (ARM64), TensorFlow Lite with Coral/Hailo delegate, \
        Arduino IDE 2.3+ or MicroPython for RP2350/ESP32-C6, Python 3.11+ based development. \
        - Performance targets: Vision at 20 FPS (MobileNetV3 + Coral achieves 100 FPS inference, camera-limited), \
        Motor control at 250 Hz, AI inference 10-25ms with accelerator, Power: 15W total system. \
        - Limitations & workarounds: Limited 8GB RAM (use 4GB swap on NVMe), no hardware H.265 encoding (use CPU), \
        lower single-thread performance (optimize with NumPy + OpenBLAS), USB3 bandwidth sharing (prioritize Coral). \
        - Development workflow: VS Code remote SSH, Jupyter Lab for ML prototyping, GitHub Actions CI/CD, Docker for deployment. \
        - Educational focus: Ideal for universities, maker spaces, prototyping, learning robotics (cost-effective at scale). \
        - BOM with Adafruit, SparkFun, CanaKit, Pimoroni part numbers (2025 availability). \
        Include beginner tutorial, troubleshooting guide, and upgrade path to Standard (Jetson Orin NX).".to_string(),
        vec![get_agent_id("EMP004"), get_agent_id("EMP002")],
    )
    .with_priority(TaskPriority::High);

    let arch_task5 = WorkspaceTask::new(
        "Generate 2025 Architecture Comparison & Migration Guide".to_string(),
        "Create comprehensive comparison matrix and migration guide for all 2025 configurations. Include: \
        - Comparison matrix: Performance (TOPS, FPS, latency, transformer throughput), Power (peak, average, battery runtime), \
        Cost (unit cost at 1/100/1000/10K volumes, TCO over 3 years), Development complexity (toolchain maturity, learning curve, ecosystem). \
        - Use case recommendations: Flagship (Jetson Thor) for cutting-edge AI products, high-end commercial robots, research institutions; \
        Premium (AGX Orin) for production deployment, enterprise robotics, advanced autonomy; \
        Standard (Orin NX) for pilot programs, mid-scale production, cost-performance balance; \
        Budget (RPi5) for education, rapid prototyping, proof-of-concept, makerspaces. \
        - Migration guides: Budget → Standard (reuse RP2350/ESP32-C6 I/O, upgrade compute+MCUs, 85% software compatible), \
        Standard → Premium (carrier board redesign, software highly compatible, unlock advanced features), \
        Premium → Flagship (Thor requires new carrier, JetPack 7.x, leverage 2000 TOPS for multi-modal AI, vision-language models). \
        - Software compatibility: ROS2 packages (which work on all platforms), GPU requirements (TensorRT vs TFLite vs Hailo), \
        RT kernel needs (motor control, sensor fusion), 2025 LLM/VLM support (Thor optimized for transformers). \
        - Hybrid configs: Mix-and-match options (e.g., Orin NX with premium STM32H7 MCUs for specific needs, \
        Thor with budget I/O boards for cost savings). \
        - Field upgrade procedures: Compute module swap in deployed units (firmware compatibility, sensor recalibration, \
        AI model retraining/quantization). \
        - 2025 supply chain: Jetson Thor (12-20 weeks, limited early availability), AGX Orin (4-8 weeks), Orin NX (2-6 weeks), \
        RPi5 (immediate-2 weeks), MCUs (immediate-8 weeks depending on model). \
        - Future-proofing: Thor roadmap (expect updates through 2027+), Orin lifecycle (2028+ EOL), software support timelines. \
        Include decision tree flowchart, ROI calculator, and 2025 market trends analysis.".to_string(),
        vec![get_agent_id("EMP020"), get_agent_id("EMP019")],
    )
    .with_priority(TaskPriority::Critical);

    let arch_tasks = vec![arch_task1, arch_task2, arch_task3, arch_task4, arch_task5];
    let arch_buildout_results = coordinator
        .coordinate_workspace_project(&platform_ws_id, arch_tasks)
        .await?;

    println!(
        "   ✅ Completed {} architecture buildout plans\n",
        arch_buildout_results.len()
    );

    // Summary
    println!("\n==========================================================\n");
    println!("✅ All Projects Completed Across Organization!");
    println!("\n📈 Project Summary:\n");
    println!(
        "   🏠 Robo-1 Development: {} tasks (Critical)",
        robo1_results.len()
    );
    println!(
        "   🏭 Robo-2 Development: {} tasks (Critical)",
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
        "   📈 Executive Strategy: {} tasks (Medium)",
        exec_results.len()
    );
    println!(
        "   📦 Product Strategy: {} tasks (Medium)",
        prod_results.len()
    );
    println!("   🤝 Customer Success: {} tasks (Low)", cust_results.len());
    println!("\n🛠️  Advanced Development Phases:\n");
    println!(
        "   🏭 Manufacturing Work Orders: {} packages (Critical)",
        mfg_wo_results.len()
    );
    println!(
        "   🔩 Assembly Procedures: {} documents (High)",
        asm_results.len()
    );
    println!(
        "   🧪 Validation Testing: {} protocols (Critical)",
        val_results.len()
    );
    println!(
        "   💻 Software Development: {} modules (Critical)",
        sw_results.len()
    );
    println!(
        "   📚 Documentation & Handoff: {} packages (Critical)",
        doc_results.len()
    );
    println!("\n🌎 Production & Commercialization Phases:\n");
    println!(
        "   ⚖️ Regulatory & Compliance: {} packages (Critical)",
        reg_results.len()
    );
    println!(
        "   🌡️ Environmental & Durability Testing: {} plans (Critical)",
        env_results.len()
    );
    println!(
        "   🏭 Production Scaling: {} plans (Critical)",
        scale_results.len()
    );
    println!(
        "   🚀 Field Deployment & Operations: {} packages (High)",
        deploy_results.len()
    );
    println!(
        "   🔒 Cybersecurity & Data Protection: {} packages (Critical)",
        cyber_results.len()
    );
    println!(
        "   💻 Compute Platform Analysis: {} packages (Critical)",
        hw_platform_results.len()
    );
    println!(
        "   🏗️ Alternative Architecture Buildouts: {} plans (Critical)",
        arch_buildout_results.len()
    );

    let total_tasks = robo1_results.len()
        + robo2_results.len()
        + robo3_results.len()
        + ai_research_results.len()
        + platform_results.len()
        + hw_results.len()
        + exec_results.len()
        + prod_results.len()
        + cust_results.len()
        + mfg_wo_results.len()
        + asm_results.len()
        + val_results.len()
        + sw_results.len()
        + doc_results.len()
        + reg_results.len()
        + env_results.len()
        + scale_results.len()
        + deploy_results.len()
        + cyber_results.len()
        + hw_platform_results.len()
        + arch_buildout_results.len();

    println!("\n   📈 Total Tasks Executed: {}", total_tasks);
    println!(
        "   🧠 Knowledge Captured: {} experiences stored in agent memories",
        total_tasks
    );
    println!("\n🎉 Complete Product Lifecycle - Concept to Commercialization!");
    println!("   Design → Manufacturing → Testing → Software → Compliance → Production → Deployment → Security → Multi-Vendor Sourcing\n");

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
async fn generate_summary_report(
    output_dir: &PathBuf,
    coordinator: &AgentCoordinator,
    artifacts: &[Artifact],
) -> Result<()> {
    let org = coordinator.get_organization().await;

    // Generate markdown summary
    let mut summary = String::new();
    summary.push_str("# RoboTech Industries Organization Demo Report\n\n");
    summary.push_str(&format!(
        "**Generated:** {}\n\n",
        chrono::Utc::now().to_rfc3339()
    ));
    summary.push_str("## Organization Overview\n\n");
    summary.push_str(&format!("- **Organization Name:** {}\n", org.name));
    summary.push_str(&format!("- **Total Agents:** {}\n", org.agents.len()));
    summary.push_str(&format!(
        "- **Total Workspaces:** {}\n",
        org.workspaces.len()
    ));
    summary.push_str(&format!(
        "- **Artifacts Generated:** {}\n\n",
        artifacts.len()
    ));

    summary.push_str("## Agents by Role\n\n");
    for (_, agent) in org.agents.iter() {
        summary.push_str(&format!("- **{}**: {:?}\n", agent.name, agent.role));
    }

    summary.push_str("\n## Workspaces\n\n");
    for (_, workspace) in org.workspaces.iter() {
        summary.push_str(&format!("### {}\n", workspace.name));
        summary.push_str(&format!("**Description:** {}\n\n", workspace.description));
        summary.push_str(&format!(
            "**Team Members:** {}\n\n",
            workspace.member_agents.len()
        ));

        let completed = workspace
            .tasks
            .iter()
            .filter(|t| matches!(t.status, the_agency::TaskStatus::Completed))
            .count();
        summary.push_str(&format!(
            "**Tasks:** {}/{} completed\n\n",
            completed,
            workspace.tasks.len()
        ));
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
            summary.push_str(&format!(
                "- **{}** (by {})\n",
                artifact.name, artifact.created_by
            ));
            summary.push_str(&format!("  - Workspace: {}\n", artifact.workspace));
            summary.push_str(&format!(
                "  - File: `artifacts/{}.{}`\n",
                artifact.name, artifact.file_extension
            ));
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
        created_by: "EMP001".to_string(),
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
"#
        .to_string(),
        file_extension: "py".to_string(),
        created_by: "EMP004".to_string(),
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
"#
        .to_string(),
        file_extension: "rs".to_string(),
        created_by: "EMP008".to_string(),
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
"#
        .to_string(),
        file_extension: "yaml".to_string(),
        created_by: "EMP005".to_string(),
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
"#
        .to_string(),
        file_extension: "md".to_string(),
        created_by: "EMP019".to_string(),
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
"#
        .to_string(),
        file_extension: "md".to_string(),
        created_by: "EMP016".to_string(),
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
"#
        .to_string(),
        file_extension: "toml".to_string(),
        created_by: "EMP010".to_string(),
        workspace: "Manufacturing Excellence".to_string(),
    });

    Ok(artifacts)
}

/// Write artifacts to disk
async fn write_artifacts(output_dir: &PathBuf, artifacts: &[Artifact]) -> Result<()> {
    for artifact in artifacts {
        let subdir = match artifact.artifact_type {
            ArtifactType::DesignDocument
            | ArtifactType::ApiSpecification
            | ArtifactType::TechnicalReport => "design_docs",
            ArtifactType::PythonCode | ArtifactType::RustCode => "code",
            ArtifactType::YamlConfig | ArtifactType::TomlConfig => "configs",
            ArtifactType::ArchitectureDiagram => "diagrams",
        };

        let file_path = output_dir
            .join("artifacts")
            .join(subdir)
            .join(format!("{}.{}", artifact.name, artifact.file_extension));

        fs::write(&file_path, &artifact.content)?;
        println!(
            "  📄 {}/{}/{}.{}",
            output_dir.join("artifacts").display(),
            subdir,
            artifact.name,
            artifact.file_extension
        );
    }

    Ok(())
}
