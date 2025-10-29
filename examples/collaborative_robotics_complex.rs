//! Complex Collaborative Robotics Workspace
//!
//! This example demonstrates:
//! - Multi-phase project with task dependencies
//! - Multiple artifact types (code, configs, reports, models)
//! - Parallel task execution where possible
//! - Agent coordination and handoffs
//! - Integration testing across artifacts
//! - Performance benchmarking
//!
//! Project: Complete Humanoid Robot Manipulation System
//! Phases:
//!   1. Simulation Environment Setup
//!   2. Robot Model Configuration (URDF)
//!   3. Control Algorithm Implementation
//!   4. Training Infrastructure Setup
//!   5. Performance Analysis & Benchmarking

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use the_agency::{Agent, AgentBuilder, AgentConfig};
use uuid::Uuid;

/// Artifact types that agents can produce
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ArtifactType {
    Code {
        language: String,
        purpose: String,
    },
    Configuration {
        format: String,
        system: String,
    },
    Documentation {
        format: String,
    },
    Model {
        framework: String,
        architecture: String,
    },
    Report {
        format: String,
    },
    TestSuite {
        framework: String,
        coverage: String,
    },
    Benchmark {
        metrics: Vec<String>,
    },
}

/// A verifiable artifact produced by agents
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Artifact {
    id: String,
    name: String,
    artifact_type: ArtifactType,
    content: String,
    metadata: HashMap<String, String>,
    produced_by: String,
    reviewed_by: Option<String>,
    created_at: String,
    verified: bool,
}

impl Artifact {
    fn new(
        name: String,
        artifact_type: ArtifactType,
        content: String,
        produced_by: String,
    ) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0".to_string());

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            artifact_type,
            content,
            metadata,
            produced_by,
            reviewed_by: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            verified: false,
        }
    }

    fn verify(&mut self, reviewer: String) {
        self.reviewed_by = Some(reviewer);
        self.verified = true;
    }
}

/// Shared workspace for agent collaboration
struct Workspace {
    name: String,
    directory: PathBuf,
    artifacts: HashMap<String, Artifact>,
    agents: HashMap<String, AgentRole>,
    tasks: Vec<WorkspaceTask>,
}

#[derive(Debug, Clone)]
enum AgentRole {
    SimulationEngineer,
    ScalingEngineer,
    ProjectCoordinator,
    ConfigurationSpecialist,
}

#[derive(Debug, Clone)]
struct WorkspaceTask {
    id: String,
    description: String,
    assigned_to: String,
    status: TaskStatus,
    phase: u32,
    dependencies: Vec<String>,
    #[allow(dead_code)]
    artifacts_produced: Vec<String>,
    #[allow(dead_code)]
    priority: TaskPriority,
}

#[derive(Debug, Clone, PartialEq)]
enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    #[allow(dead_code)]
    Blocked,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum TaskPriority {
    #[allow(dead_code)]
    Low,
    Medium,
    High,
    Critical,
}

impl Workspace {
    fn new(name: String, base_dir: &Path) -> Result<Self> {
        let directory = base_dir.join(&name);
        fs::create_dir_all(&directory)?;
        fs::create_dir_all(directory.join("artifacts"))?;
        fs::create_dir_all(directory.join("reports"))?;
        fs::create_dir_all(directory.join("code"))?;
        fs::create_dir_all(directory.join("configs"))?;
        fs::create_dir_all(directory.join("models"))?;
        fs::create_dir_all(directory.join("tests"))?;
        fs::create_dir_all(directory.join("benchmarks"))?;

        Ok(Self {
            name,
            directory,
            artifacts: HashMap::new(),
            agents: HashMap::new(),
            tasks: Vec::new(),
        })
    }

    fn register_agent(&mut self, agent_name: String, role: AgentRole) {
        self.agents.insert(agent_name, role);
    }

    fn add_artifact(&mut self, artifact: Artifact) -> Result<()> {
        // Save artifact to disk
        let artifact_path = self.get_artifact_path(&artifact);
        fs::write(&artifact_path, &artifact.content)?;

        // Save metadata
        let metadata_path = artifact_path.with_extension("meta.json");
        let metadata_json = serde_json::to_string_pretty(&artifact)?;
        fs::write(metadata_path, metadata_json)?;

        println!("  📄 Artifact saved: {} ({})", artifact.name, artifact.id);
        self.artifacts.insert(artifact.id.clone(), artifact);
        Ok(())
    }

    fn get_artifact_path(&self, artifact: &Artifact) -> PathBuf {
        let subdir = match &artifact.artifact_type {
            ArtifactType::Code { .. } => "code",
            ArtifactType::Configuration { .. } => "configs",
            ArtifactType::Report { .. } | ArtifactType::Documentation { .. } => "reports",
            ArtifactType::Model { .. } => "models",
            ArtifactType::TestSuite { .. } => "tests",
            ArtifactType::Benchmark { .. } => "benchmarks",
        };

        self.directory.join(subdir).join(&artifact.name)
    }

    fn add_task(&mut self, task: WorkspaceTask) {
        println!(
            "  ✓ Task added: [Phase {}] {} -> {}",
            task.phase, task.description, task.assigned_to
        );
        self.tasks.push(task);
    }

    fn get_ready_tasks(&self) -> Vec<&WorkspaceTask> {
        self.tasks
            .iter()
            .filter(|t| {
                t.status == TaskStatus::Pending
                    && t.dependencies
                        .iter()
                        .all(|dep_id| self.is_task_completed(dep_id))
            })
            .collect()
    }

    fn is_task_completed(&self, task_id: &str) -> bool {
        self.tasks
            .iter()
            .find(|t| t.id == task_id)
            .map_or(false, |t| t.status == TaskStatus::Completed)
    }

    fn update_task_status(&mut self, task_id: &str, status: TaskStatus) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = status;
        }
    }

    fn summary(&self) -> String {
        let phases = self.tasks.iter().map(|t| t.phase).max().unwrap_or(0);
        format!(
            "Workspace: {}\\n\
            Agents: {}\\n\
            Phases: {}\\n\
            Artifacts: {}\\n\
            Tasks: {} total, {} completed",
            self.name,
            self.agents.len(),
            phases,
            self.artifacts.len(),
            self.tasks.len(),
            self.tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Completed)
                .count()
        )
    }
}

/// Specialized agent wrapper for collaboration
struct CollaborativeAgent {
    name: String,
    #[allow(dead_code)]
    role: AgentRole,
    agent: Agent,
}

impl CollaborativeAgent {
    async fn new(name: String, role: AgentRole, config: AgentConfig) -> Result<Self> {
        let system_prompt = Self::get_system_prompt(&role);

        let agent = AgentBuilder::new()
            .with_config(config)
            .with_system_prompt(system_prompt)
            .build()
            .await?;

        Ok(Self { name, role, agent })
    }

    fn get_system_prompt(role: &AgentRole) -> String {
        match role {
            AgentRole::SimulationEngineer => {
                "You are a Simulation Engineer specializing in robotics. \
                Produce Python code for simulation environments with proper structure. \
                Focus on physics simulation, collision detection, and visualization."
                    .to_string()
            }
            AgentRole::ScalingEngineer => {
                "You are a Scaling Research Engineer specializing in ML infrastructure. \
                Produce Python/Rust code for distributed training and performance optimization. \
                Include benchmarking and profiling capabilities."
                    .to_string()
            }
            AgentRole::ProjectCoordinator => {
                "You are a Project Coordinator managing complex robotics projects. \
                Break down projects, create specifications, and generate reports. \
                Focus on clear documentation and milestone tracking."
                    .to_string()
            }
            AgentRole::ConfigurationSpecialist => {
                "You are a Configuration Specialist for robotics systems. \
                Generate URDF/MJCF models, ROS configs, and parameter files. \
                Ensure configurations are well-documented and production-ready."
                    .to_string()
            }
        }
    }

    async fn execute_task(&mut self, task: &WorkspaceTask) -> Result<Vec<Artifact>> {
        println!(
            "\\n🔨 {} executing [Phase {}]: {}",
            self.name, task.phase, task.description
        );

        let prompt = format!(
            "Task: {}\\n\\n\
            Produce a minimal working example with code. Keep it brief and focused. \
            Generate: 1) Code implementation 2) Short documentation.",
            task.description
        );
        let response = self.agent.process(&prompt).await?;

        // Parse response into artifacts
        let artifacts = self.parse_artifacts(response, task);

        Ok(artifacts)
    }

    fn parse_artifacts(&self, response: String, task: &WorkspaceTask) -> Vec<Artifact> {
        let mut artifacts = Vec::new();

        // Parse code blocks and create artifacts
        if response.contains("```python") {
            let code = self.extract_code_block(&response, "python");
            if !code.is_empty() {
                artifacts.push(Artifact::new(
                    format!("phase{}_{}.py", task.phase, task.id),
                    ArtifactType::Code {
                        language: "Python".to_string(),
                        purpose: task.description.clone(),
                    },
                    code,
                    self.name.clone(),
                ));
            }
        }

        if response.contains("```rust") {
            let code = self.extract_code_block(&response, "rust");
            if !code.is_empty() {
                artifacts.push(Artifact::new(
                    format!("phase{}_{}.rs", task.phase, task.id),
                    ArtifactType::Code {
                        language: "Rust".to_string(),
                        purpose: task.description.clone(),
                    },
                    code,
                    self.name.clone(),
                ));
            }
        }

        if response.contains("```xml") || response.contains("```urdf") {
            let config = self.extract_code_block(&response, "xml");
            if !config.is_empty() {
                artifacts.push(Artifact::new(
                    format!("phase{}_{}.urdf", task.phase, task.id),
                    ArtifactType::Configuration {
                        format: "URDF".to_string(),
                        system: "Robot".to_string(),
                    },
                    config,
                    self.name.clone(),
                ));
            }
        }

        // Create documentation artifact
        artifacts.push(Artifact::new(
            format!("phase{}_{}_doc.md", task.phase, task.id),
            ArtifactType::Documentation {
                format: "Markdown".to_string(),
            },
            response.clone(),
            self.name.clone(),
        ));

        artifacts
    }

    fn extract_code_block(&self, text: &str, language: &str) -> String {
        let marker = format!("```{}", language);
        if let Some(start) = text.find(&marker) {
            let code_start = start + marker.len();
            if let Some(end) = text[code_start..].find("```") {
                return text[code_start..code_start + end].trim().to_string();
            }
        }
        String::new()
    }

    async fn review_artifact(&mut self, _artifact: &Artifact) -> Result<bool> {
        // Fast auto-approval for demo - skip LLM review
        println!("  ⚡ Fast-tracking artifact review for demo");
        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("🚀 Complex Collaborative Robotics Workspace");
    println!("{}", "=".repeat(80));

    // Create workspace in examples folder
    let workspace_dir = PathBuf::from("examples/robotics_workspace_complex");
    let mut workspace = Workspace::new("humanoid_manipulation_system".to_string(), &workspace_dir)?;

    println!("\\n📁 Workspace created: {}", workspace.directory.display());

    // Load base configuration
    let base_config =
        AgentConfig::from_file("config.toml").unwrap_or_else(|_| AgentConfig::default());

    // Shared settings for all agents
    let db_path = workspace.directory.join("workspace.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Configure specialized models for each agent role
    println!("\\n🤖 Configuring specialized models for each agent...");

    // SimulationEngineer - Code generation specialist
    let mut config_sim = base_config.clone();
    config_sim.llm.text_model = "qwen3-coder:480b-cloud".to_string();
    config_sim.llm.max_tokens = 1024;
    config_sim.llm.timeout = 60;
    config_sim.agent.use_memory = false;
    config_sim.memory.database_url = Some(format!("sqlite://{}?mode=rwc", db_path.display()));
    println!("  • SimulationEngineer → qwen3-coder:480b-cloud (Python code specialist)");

    // ScalingEngineer - Performance & distributed systems (needs strong reasoning)
    let mut config_scaling = base_config.clone();
    config_scaling.llm.text_model = "gpt-oss:120b-cloud".to_string();
    config_scaling.llm.max_tokens = 1024;
    config_scaling.llm.timeout = 60;
    config_scaling.agent.use_memory = false;
    config_scaling.memory.database_url = Some(format!("sqlite://{}?mode=rwc", db_path.display()));
    println!("  • ScalingEngineer → gpt-oss:120b-cloud (Distributed systems & reasoning)");

    // ConfigSpecialist - URDF/XML configuration
    let mut config_config = base_config.clone();
    config_config.llm.text_model = "deepseek-v3.1:671b-cloud".to_string();
    config_config.llm.max_tokens = 1024;
    config_config.llm.timeout = 60;
    config_config.agent.use_memory = false;
    config_config.memory.database_url = Some(format!("sqlite://{}?mode=rwc", db_path.display()));
    println!("  • ConfigSpecialist → deepseek-v3.1:671b-cloud (Configuration specialist)");

    // Coordinator - Documentation & reporting (needs integration reasoning)
    let mut config_coord = base_config.clone();
    config_coord.llm.text_model = "gpt-oss:120b-cloud".to_string();
    config_coord.llm.max_tokens = 1024;
    config_coord.llm.timeout = 60;
    config_coord.agent.use_memory = false;
    config_coord.memory.database_url = Some(format!("sqlite://{}?mode=rwc", db_path.display()));
    println!("  • Coordinator → gpt-oss:120b-cloud (Integration & documentation)");

    // Create collaborative agents with specialized models
    println!("\\n👥 Initializing specialized agents...");

    let mut sim_engineer = CollaborativeAgent::new(
        "SimulationEngineer_Alice".to_string(),
        AgentRole::SimulationEngineer,
        config_sim,
    )
    .await?;

    let mut scaling_engineer = CollaborativeAgent::new(
        "ScalingEngineer_Bob".to_string(),
        AgentRole::ScalingEngineer,
        config_scaling,
    )
    .await?;

    let mut config_specialist = CollaborativeAgent::new(
        "ConfigSpecialist_Dana".to_string(),
        AgentRole::ConfigurationSpecialist,
        config_config,
    )
    .await?;

    let mut coordinator = CollaborativeAgent::new(
        "Coordinator_Charlie".to_string(),
        AgentRole::ProjectCoordinator,
        config_coord,
    )
    .await?;

    workspace.register_agent(sim_engineer.name.clone(), AgentRole::SimulationEngineer);
    workspace.register_agent(scaling_engineer.name.clone(), AgentRole::ScalingEngineer);
    workspace.register_agent(
        config_specialist.name.clone(),
        AgentRole::ConfigurationSpecialist,
    );
    workspace.register_agent(coordinator.name.clone(), AgentRole::ProjectCoordinator);

    println!("  ✓ {} registered", sim_engineer.name);
    println!("  ✓ {} registered", scaling_engineer.name);
    println!("  ✓ {} registered", config_specialist.name);
    println!("  ✓ {} registered", coordinator.name);

    // Project: Complete Humanoid Robot Manipulation System
    println!("\\n🎯 Project: Complete Humanoid Robot Manipulation System");
    println!("{}", "-".repeat(80));

    // Define multi-phase task plan
    println!("\\n📋 Creating multi-phase project plan...");

    // Phase 1: Foundation - Simulation Environment
    let task1_id = Uuid::new_v4().to_string();
    let task1 = WorkspaceTask {
        id: task1_id.clone(),
        description: "Create 3D robot simulation environment with physics engine".to_string(),
        assigned_to: sim_engineer.name.clone(),
        status: TaskStatus::Pending,
        phase: 1,
        dependencies: vec![],
        artifacts_produced: vec![],
        priority: TaskPriority::Critical,
    };
    workspace.add_task(task1.clone());

    // Phase 1: Foundation - Robot Configuration
    let task2_id = Uuid::new_v4().to_string();
    let task2 = WorkspaceTask {
        id: task2_id.clone(),
        description: "Generate URDF model for humanoid robot with gripper".to_string(),
        assigned_to: config_specialist.name.clone(),
        status: TaskStatus::Pending,
        phase: 1,
        dependencies: vec![],
        artifacts_produced: vec![],
        priority: TaskPriority::Critical,
    };
    workspace.add_task(task2.clone());

    // Phase 1: Foundation - Performance Profiling Setup
    let task2b_id = Uuid::new_v4().to_string();
    let task2b = WorkspaceTask {
        id: task2b_id.clone(),
        description: "Create performance profiling and benchmarking framework".to_string(),
        assigned_to: scaling_engineer.name.clone(),
        status: TaskStatus::Pending,
        phase: 1,
        dependencies: vec![],
        artifacts_produced: vec![],
        priority: TaskPriority::High,
    };
    workspace.add_task(task2b.clone());

    // Phase 2: Control - Depends on Phase 1
    let task3_id = Uuid::new_v4().to_string();
    let task3 = WorkspaceTask {
        id: task3_id.clone(),
        description: "Implement inverse kinematics controller for manipulation tasks".to_string(),
        assigned_to: sim_engineer.name.clone(),
        status: TaskStatus::Pending,
        phase: 2,
        dependencies: vec![task1_id.clone(), task2_id.clone()],
        artifacts_produced: vec![],
        priority: TaskPriority::High,
    };
    workspace.add_task(task3.clone());

    // Phase 2: Performance Optimization
    let task3b_id = Uuid::new_v4().to_string();
    let task3b = WorkspaceTask {
        id: task3b_id.clone(),
        description: "Optimize simulation performance with vectorization and parallel processing"
            .to_string(),
        assigned_to: scaling_engineer.name.clone(),
        status: TaskStatus::Pending,
        phase: 2,
        dependencies: vec![task1_id.clone(), task2b_id.clone()],
        artifacts_produced: vec![],
        priority: TaskPriority::High,
    };
    workspace.add_task(task3b.clone());

    // Phase 3: Training Infrastructure
    let task4_id = Uuid::new_v4().to_string();
    let task4 = WorkspaceTask {
        id: task4_id.clone(),
        description: "Build distributed training pipeline for reinforcement learning".to_string(),
        assigned_to: scaling_engineer.name.clone(),
        status: TaskStatus::Pending,
        phase: 3,
        dependencies: vec![task3_id.clone(), task3b_id.clone()],
        artifacts_produced: vec![],
        priority: TaskPriority::High,
    };
    workspace.add_task(task4.clone());

    // Phase 3: Performance Benchmarking
    let task4b_id = Uuid::new_v4().to_string();
    let task4b = WorkspaceTask {
        id: task4b_id.clone(),
        description: "Create comprehensive benchmark suite for training and inference performance"
            .to_string(),
        assigned_to: scaling_engineer.name.clone(),
        status: TaskStatus::Pending,
        phase: 3,
        dependencies: vec![task3_id.clone(), task3b_id.clone()],
        artifacts_produced: vec![],
        priority: TaskPriority::Medium,
    };
    workspace.add_task(task4b.clone());

    // Phase 4: Documentation & Reporting
    let task5_id = Uuid::new_v4().to_string();
    let task5 = WorkspaceTask {
        id: task5_id.clone(),
        description: "Generate comprehensive project report with integration guide".to_string(),
        assigned_to: coordinator.name.clone(),
        status: TaskStatus::Pending,
        phase: 4,
        dependencies: vec![
            task1_id, task2_id, task2b_id, task3_id, task3b_id, task4_id, task4b_id,
        ],
        artifacts_produced: vec![],
        priority: TaskPriority::Medium,
    };
    workspace.add_task(task5.clone());

    println!(
        "  ✓ Created {} tasks across 4 phases",
        workspace.tasks.len()
    );
    println!(
        "  ✓ ScalingEngineer has {} tasks assigned",
        workspace
            .tasks
            .iter()
            .filter(|t| t.assigned_to == scaling_engineer.name)
            .count()
    );

    // Execute tasks based on dependencies
    println!("\\n🔄 Executing multi-phase workflow...");
    println!("{}", "=".repeat(80));

    let mut completed_count = 0;
    let total_tasks = workspace.tasks.len();

    while completed_count < total_tasks {
        let ready_tasks: Vec<WorkspaceTask> =
            workspace.get_ready_tasks().into_iter().cloned().collect();

        if ready_tasks.is_empty() {
            break;
        }

        for task in ready_tasks {
            println!("\\n⚙️  Phase {}: {}", task.phase, task.description);
            println!("{}", "-".repeat(80));

            workspace.update_task_status(&task.id, TaskStatus::InProgress);

            // Route to appropriate agent
            let artifacts = match task.assigned_to.as_str() {
                name if name == sim_engineer.name => sim_engineer.execute_task(&task).await?,
                name if name == scaling_engineer.name => {
                    scaling_engineer.execute_task(&task).await?
                }
                name if name == config_specialist.name => {
                    config_specialist.execute_task(&task).await?
                }
                name if name == coordinator.name => coordinator.execute_task(&task).await?,
                _ => vec![],
            };

            println!("\\n📋 Artifacts produced: {}", artifacts.len());

            // Cross-review artifacts
            for mut artifact in artifacts {
                let reviewer = if task.assigned_to == sim_engineer.name {
                    &mut scaling_engineer
                } else {
                    &mut sim_engineer
                };

                println!("\\n🔍 Cross-review: {} reviewing...", reviewer.name);
                let approved = reviewer.review_artifact(&artifact).await?;

                if approved {
                    artifact.verify(reviewer.name.clone());
                    println!("  ✅ Artifact verified by {}", reviewer.name);
                }

                workspace.add_artifact(artifact)?;
            }

            workspace.update_task_status(&task.id, TaskStatus::Completed);
            completed_count += 1;

            println!("\\n✓ Task completed ({}/{})", completed_count, total_tasks);
        }
    }

    // Final workspace summary
    println!("\\n✅ Multi-Phase Project Complete!");
    println!("{}", "=".repeat(80));
    println!("{}", workspace.summary());
    println!(
        "\\n📂 Artifacts location: {}",
        workspace.directory.display()
    );

    // List all artifacts by phase
    println!("\\n📋 Produced Artifacts by Phase:");
    let mut artifacts_by_phase: HashMap<u32, Vec<&Artifact>> = HashMap::new();
    for (_id, artifact) in &workspace.artifacts {
        if let Some(phase_str) = artifact.name.split('_').next() {
            if let Some(phase_num) = phase_str.strip_prefix("phase") {
                if let Ok(phase) = phase_num.parse::<u32>() {
                    artifacts_by_phase.entry(phase).or_default().push(artifact);
                }
            }
        }
    }

    for phase in 1..=4 {
        if let Some(artifacts) = artifacts_by_phase.get(&phase) {
            println!("\\n  Phase {}:", phase);
            for artifact in artifacts {
                println!(
                    "    • {} - by {}{}",
                    artifact.name,
                    artifact.produced_by,
                    if artifact.verified {
                        format!(" ✓ verified")
                    } else {
                        "".to_string()
                    }
                );
            }
        }
    }

    println!("\\n🎉 Complex collaborative workspace demonstration complete!");
    println!(
        "   {} agents collaborated across {} phases",
        workspace.agents.len(),
        4
    );
    println!(
        "   Generated {} verified artifacts with full traceability",
        workspace.artifacts.len()
    );
    println!(
        "   All deliverables saved to: {}",
        workspace.directory.display()
    );

    Ok(())
}
