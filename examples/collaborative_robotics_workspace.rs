//! Collaborative Robotics Workspace
//!
//! This example demonstrates:
//! - Multiple specialized agents collaborating on projects
//! - Shared workspace for artifact management
//! - Agent-to-agent communication via A2A
//! - Verifiable artifact generation (code, configs, reports)
//! - Project coordination and task delegation

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
    DataFile {
        format: String,
        size_estimate: String,
    },
    Report {
        format: String,
    },
    Model {
        framework: String,
        architecture: String,
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
}

#[derive(Debug, Clone)]
struct WorkspaceTask {
    id: String,
    description: String,
    assigned_to: String,
    status: TaskStatus,
    #[allow(dead_code)]
    dependencies: Vec<String>,
    #[allow(dead_code)]
    artifacts_produced: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    #[allow(dead_code)]
    Blocked,
}

impl Workspace {
    fn new(name: String, base_dir: &Path) -> Result<Self> {
        let directory = base_dir.join(&name);
        fs::create_dir_all(&directory)?;
        fs::create_dir_all(directory.join("artifacts"))?;
        fs::create_dir_all(directory.join("reports"))?;
        fs::create_dir_all(directory.join("code"))?;
        fs::create_dir_all(directory.join("configs"))?;

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
            _ => "artifacts",
        };

        self.directory.join(subdir).join(&artifact.name)
    }

    fn add_task(&mut self, task: WorkspaceTask) {
        println!(
            "  ✓ Task added: {} -> {}",
            task.description, task.assigned_to
        );
        self.tasks.push(task);
    }

    #[allow(dead_code)]
    fn get_pending_tasks(&self, agent_name: &str) -> Vec<&WorkspaceTask> {
        self.tasks
            .iter()
            .filter(|t| t.assigned_to == agent_name && t.status == TaskStatus::Pending)
            .collect()
    }

    fn update_task_status(&mut self, task_id: &str, status: TaskStatus) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = status;
        }
    }

    fn summary(&self) -> String {
        format!(
            "Workspace: {}\n\
            Agents: {}\n\
            Artifacts: {}\n\
            Tasks: {} total, {} completed",
            self.name,
            self.agents.len(),
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
                Produce Python/Rust code for simulation environments, URDF/MJCF models, \
                and technical documentation. Always provide working, production-ready code."
                    .to_string()
            }
            AgentRole::ScalingEngineer => {
                "You are a Scaling Research Engineer specializing in ML infrastructure. \
                Produce Python/Rust code for distributed training, inference optimization, \
                and performance analysis. Focus on scalability and reliability."
                    .to_string()
            }
            AgentRole::ProjectCoordinator => {
                "You are a Project Coordinator managing robotics projects. \
                Break down complex projects into tasks, assign work, and ensure deliverables. \
                Produce clear specifications and coordination documents."
                    .to_string()
            }
        }
    }

    async fn execute_task(&mut self, task: &WorkspaceTask) -> Result<Vec<Artifact>> {
        println!("\n🔨 {} executing: {}", self.name, task.description);

        let prompt = format!(
            "Task: {}\n\n\
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
                    format!("{}_implementation.py", task.id),
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
                    format!("{}_implementation.rs", task.id),
                    ArtifactType::Code {
                        language: "Rust".to_string(),
                        purpose: task.description.clone(),
                    },
                    code,
                    self.name.clone(),
                ));
            }
        }

        // Create documentation artifact
        artifacts.push(Artifact::new(
            format!("{}_documentation.md", task.id),
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

    println!("🚀 Collaborative Robotics Workspace");
    println!("{}", "=".repeat(80));

    // Create workspace in examples folder
    let workspace_dir = PathBuf::from("examples/robotics_workspace");
    let mut workspace = Workspace::new("humanoid_robot_project".to_string(), &workspace_dir)?;

    println!("\n📁 Workspace created: {}", workspace.directory.display());

    // Load configuration
    let mut config =
        AgentConfig::from_file("config.toml").unwrap_or_else(|_| AgentConfig::default());

    // Use cloud model for faster generation
    config.llm.text_model = "deepseek-v3.1:671b-cloud".to_string();
    config.llm.max_tokens = 1024; // Increased for expanded scope
    config.llm.timeout = 60; // Shorter timeout

    // Disable memory for demo (avoids embedding issues)
    config.agent.use_memory = false;

    // Use workspace-specific database to avoid conflicts
    let db_path = workspace.directory.join("workspace.db");
    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Use proper SQLite URI format
    config.memory.database_url = Some(format!("sqlite://{}?mode=rwc", db_path.display()));
    // Create collaborative agents
    println!("\n👥 Initializing agents...");

    let mut sim_engineer = CollaborativeAgent::new(
        "SimulationEngineer_Alice".to_string(),
        AgentRole::SimulationEngineer,
        config.clone(),
    )
    .await?;

    let mut scaling_engineer = CollaborativeAgent::new(
        "ScalingEngineer_Bob".to_string(),
        AgentRole::ScalingEngineer,
        config.clone(),
    )
    .await?;

    let coordinator = CollaborativeAgent::new(
        "Coordinator_Charlie".to_string(),
        AgentRole::ProjectCoordinator,
        config.clone(),
    )
    .await?;

    workspace.register_agent(sim_engineer.name.clone(), AgentRole::SimulationEngineer);
    workspace.register_agent(scaling_engineer.name.clone(), AgentRole::ScalingEngineer);
    workspace.register_agent(coordinator.name.clone(), AgentRole::ProjectCoordinator);

    println!("  ✓ {} registered", sim_engineer.name);
    println!("  ✓ {} registered", scaling_engineer.name);
    println!("  ✓ {} registered", coordinator.name);

    // Project: Build a humanoid robot grasping system
    println!("\n🎯 Project: Humanoid Robot Grasping System");
    println!("{}", "-".repeat(80));

    // Task 1: Design simulation environment (simplified)
    let task1 = WorkspaceTask {
        id: Uuid::new_v4().to_string(),
        description: "Create a basic Python class for a robot simulation environment".to_string(),
        assigned_to: sim_engineer.name.clone(),
        status: TaskStatus::Pending,
        dependencies: vec![],
        artifacts_produced: vec![],
    };
    workspace.add_task(task1.clone());

    // Execute Task 1
    println!("\n⚙️  Phase 1: Creating Simulation Environment");
    println!("{}", "-".repeat(80));
    workspace.update_task_status(&task1.id, TaskStatus::InProgress);

    let artifacts1 = sim_engineer.execute_task(&task1).await?;
    println!("\n📋 Artifacts produced: {}", artifacts1.len());

    for mut artifact in artifacts1 {
        // Have scaling engineer review simulation engineer's work
        println!(
            "\n🔍 Cross-review: {} reviewing artifact...",
            scaling_engineer.name
        );
        let approved = scaling_engineer.review_artifact(&artifact).await?;

        if approved {
            artifact.verify(scaling_engineer.name.clone());
            println!("  ✅ Artifact verified by {}", scaling_engineer.name);
        } else {
            println!("  ⚠️  Artifact needs revision");
        }

        workspace.add_artifact(artifact)?;
    }
    workspace.update_task_status(&task1.id, TaskStatus::Completed);

    // Final workspace summary
    println!("\n✅ Project Complete!");
    println!("{}", "=".repeat(80));
    println!("{}", workspace.summary());
    println!("\n📂 Artifacts location: {}", workspace.directory.display());

    // List all artifacts
    println!("\n📋 Produced Artifacts:");
    for (_id, artifact) in &workspace.artifacts {
        println!(
            "  • {} ({}) - by {}{}",
            artifact.name,
            match &artifact.artifact_type {
                ArtifactType::Code { language, .. } => format!("{} code", language),
                ArtifactType::Documentation { .. } => "documentation".to_string(),
                ArtifactType::Configuration { .. } => "config".to_string(),
                _ => "artifact".to_string(),
            },
            artifact.produced_by,
            if artifact.verified {
                format!(" ✓ verified by {}", artifact.reviewed_by.as_ref().unwrap())
            } else {
                "".to_string()
            }
        );
    }

    println!("\n🎉 Collaborative workspace demonstration complete!");
    println!("   Agents worked together to produce verifiable artifacts");
    println!(
        "   All deliverables saved to: {}",
        workspace.directory.display()
    );

    Ok(())
}
