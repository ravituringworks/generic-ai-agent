//! Enhanced Collaborative Robotics Workspace with Specialized Agents
//!
//! This example demonstrates:
//! - Importing specialized agent capabilities from standalone examples
//! - Type-safe task execution with domain-specific methods
//! - Richer agent collaboration with structured interactions
//! - Better output quality through specialized prompts

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use the_agency::AgentConfig;
use uuid::Uuid;

// Import specialized agent modules
mod scaling_engineer_agent;
mod simulation_engineer_agent;

use scaling_engineer_agent::ScalingEngineerAgent;
use simulation_engineer_agent::SimulationEngineerAgent;

/// Artifact types that agents can produce
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ArtifactType {
    SimulationEnvironment {
        physics_engine: String,
        realism_score: f32,
    },
    ScalingInfrastructure {
        num_gpus: usize,
        throughput: f32,
    },
    Configuration {
        format: String,
        system: String,
    },
    Documentation {
        format: String,
    },
    PerformanceAnalysis {
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
    tasks: Vec<WorkspaceTask>,
}

/// Enhanced task structure with type-safe task types
#[derive(Debug, Clone)]
struct WorkspaceTask {
    id: String,
    description: String,
    task_type: TaskType,
    assigned_to: AgentType,
    status: TaskStatus,
    phase: u32,
    dependencies: Vec<String>,
    priority: TaskPriority,
}

/// Type-safe task types matching specialized agent capabilities
#[derive(Debug, Clone)]
enum TaskType {
    // Simulation Engineer tasks
    DesignEnvironment {
        requirements: Vec<String>,
    },
    AnalyzeSimToRealGap {
        policy_description: String,
    },
    PrototypeHardware {
        hardware_spec: String,
    },

    // Scaling Engineer tasks
    DistributedTraining {
        num_gpus: usize,
        model_type: String,
    },
    OptimizeInference {
        target_latency_ms: f32,
        deployment_target: String,
    },
    DataPipeline {
        throughput_gbps: f32,
        dataset_size_tb: f32,
    },

    // Generic task for coordination
    GenerateReport {
        topic: String,
    },
}

#[derive(Debug, Clone)]
enum AgentType {
    SimulationEngineer,
    ScalingEngineer,
    ProjectCoordinator,
}

#[derive(Debug, Clone, PartialEq)]
enum TaskStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum TaskPriority {
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
        fs::create_dir_all(directory.join("simulations"))?;
        fs::create_dir_all(directory.join("infrastructure"))?;

        Ok(Self {
            name,
            directory,
            artifacts: HashMap::new(),
            tasks: Vec::new(),
        })
    }

    fn add_artifact(&mut self, artifact: Artifact) -> Result<()> {
        let artifact_path = self.get_artifact_path(&artifact);
        fs::write(&artifact_path, &artifact.content)?;

        let metadata_path = artifact_path.with_extension("meta.json");
        let metadata_json = serde_json::to_string_pretty(&artifact)?;
        fs::write(metadata_path, metadata_json)?;

        println!("  📄 Artifact saved: {} ({})", artifact.name, artifact.id);
        self.artifacts.insert(artifact.id.clone(), artifact);
        Ok(())
    }

    fn get_artifact_path(&self, artifact: &Artifact) -> PathBuf {
        let subdir = match &artifact.artifact_type {
            ArtifactType::SimulationEnvironment { .. } => "simulations",
            ArtifactType::ScalingInfrastructure { .. } => "infrastructure",
            ArtifactType::Configuration { .. } => "artifacts",
            ArtifactType::Documentation { .. } | ArtifactType::PerformanceAnalysis { .. } => {
                "reports"
            }
        };

        self.directory.join(subdir).join(&artifact.name)
    }

    fn add_task(&mut self, task: WorkspaceTask) {
        println!(
            "  ✓ Task added: [Phase {}] {} -> {:?}",
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
}

/// Enhanced collaborative orchestrator
struct CollaborativeOrchestrator {
    workspace: Workspace,
    sim_engineer: SimulationEngineerAgent,
    scaling_engineer: ScalingEngineerAgent,
}

impl CollaborativeOrchestrator {
    async fn new(workspace: Workspace, config: AgentConfig) -> Result<Self> {
        println!("🤖 Initializing specialized agents...");

        let sim_engineer = SimulationEngineerAgent::new(config.clone()).await?;
        let scaling_engineer = ScalingEngineerAgent::new(config.clone()).await?;

        println!("  ✓ SimulationEngineer initialized with specialized capabilities");
        println!("  ✓ ScalingEngineer initialized with specialized capabilities");

        Ok(Self {
            workspace,
            sim_engineer,
            scaling_engineer,
        })
    }

    async fn execute_task(&mut self, task: &WorkspaceTask) -> Result<Vec<Artifact>> {
        println!(
            "\n🔨 Executing [Phase {}]: {}",
            task.phase, task.description
        );

        match (&task.assigned_to, &task.task_type) {
            // Simulation Engineer tasks
            (AgentType::SimulationEngineer, TaskType::DesignEnvironment { requirements }) => {
                let response = self
                    .sim_engineer
                    .design_environment(&task.description, requirements.clone())
                    .await?;

                Ok(vec![Artifact::new(
                    format!("phase{}_sim_env_{}.md", task.phase, task.id),
                    ArtifactType::SimulationEnvironment {
                        physics_engine: "MuJoCo".to_string(),
                        realism_score: 0.85,
                    },
                    response,
                    "SimulationEngineer".to_string(),
                )])
            }

            (
                AgentType::SimulationEngineer,
                TaskType::AnalyzeSimToRealGap { policy_description },
            ) => {
                let response = self
                    .sim_engineer
                    .analyze_sim_to_real_gap(policy_description)
                    .await?;

                Ok(vec![Artifact::new(
                    format!("phase{}_sim_to_real_{}.md", task.phase, task.id),
                    ArtifactType::PerformanceAnalysis {
                        metrics: vec![
                            "domain_randomization".to_string(),
                            "reality_gap".to_string(),
                        ],
                    },
                    response,
                    "SimulationEngineer".to_string(),
                )])
            }

            (AgentType::SimulationEngineer, TaskType::PrototypeHardware { hardware_spec }) => {
                let response = self.sim_engineer.prototype_hardware(hardware_spec).await?;

                Ok(vec![Artifact::new(
                    format!("phase{}_hardware_{}.urdf", task.phase, task.id),
                    ArtifactType::Configuration {
                        format: "URDF".to_string(),
                        system: "Robot".to_string(),
                    },
                    response,
                    "SimulationEngineer".to_string(),
                )])
            }

            // Scaling Engineer tasks
            (
                AgentType::ScalingEngineer,
                TaskType::DistributedTraining {
                    num_gpus,
                    model_type,
                },
            ) => {
                let response = self
                    .scaling_engineer
                    .design_distributed_training(*num_gpus, model_type, "100TB")
                    .await?;

                Ok(vec![Artifact::new(
                    format!("phase{}_training_{}.md", task.phase, task.id),
                    ArtifactType::ScalingInfrastructure {
                        num_gpus: *num_gpus,
                        throughput: 1000.0,
                    },
                    response,
                    "ScalingEngineer".to_string(),
                )])
            }

            (
                AgentType::ScalingEngineer,
                TaskType::OptimizeInference {
                    target_latency_ms,
                    deployment_target,
                },
            ) => {
                let response = if deployment_target == "datacenter" {
                    self.scaling_engineer
                        .optimize_datacenter_inference(&task.description, 10000)
                        .await?
                } else {
                    self.scaling_engineer
                        .optimize_edge_deployment(
                            &task.description,
                            *target_latency_ms,
                            "NVIDIA Jetson",
                        )
                        .await?
                };

                Ok(vec![Artifact::new(
                    format!("phase{}_inference_{}.md", task.phase, task.id),
                    ArtifactType::PerformanceAnalysis {
                        metrics: vec!["latency".to_string(), "throughput".to_string()],
                    },
                    response,
                    "ScalingEngineer".to_string(),
                )])
            }

            // Default handler for unmatched tasks
            _ => {
                println!("  ⚠️  Task type not implemented, using generic execution");
                Ok(vec![])
            }
        }
    }

    async fn run(&mut self) -> Result<()> {
        println!("\n🔄 Executing multi-phase workflow...");
        println!("{}", "=".repeat(80));

        let total_tasks = self.workspace.tasks.len();
        let mut completed = 0;

        while completed < total_tasks {
            let ready_tasks: Vec<WorkspaceTask> = self
                .workspace
                .get_ready_tasks()
                .into_iter()
                .cloned()
                .collect();

            if ready_tasks.is_empty() {
                break;
            }

            for task in ready_tasks {
                self.workspace
                    .update_task_status(&task.id, TaskStatus::InProgress);

                let artifacts = self.execute_task(&task).await?;

                println!("\n📋 Artifacts produced: {}", artifacts.len());
                for artifact in artifacts {
                    self.workspace.add_artifact(artifact)?;
                }

                self.workspace
                    .update_task_status(&task.id, TaskStatus::Completed);
                completed += 1;

                println!("\n✓ Task completed ({}/{})", completed, total_tasks);
            }
        }

        println!("\n✅ All tasks completed!");
        println!(
            "📂 Artifacts location: {}",
            self.workspace.directory.display()
        );

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("🚀 Enhanced Collaborative Robotics Workspace");
    println!("{}", "=".repeat(80));

    // Create workspace
    let workspace_dir = PathBuf::from("examples/robotics_workspace_enhanced");
    let mut workspace = Workspace::new("humanoid_system_v2".to_string(), &workspace_dir)?;

    println!("\n📁 Workspace created: {}", workspace.directory.display());

    // Load configuration
    let config_path = "examples/collaborative_workspace_config.toml";
    let mut config = AgentConfig::from_file(config_path)
        .unwrap_or_else(|_| AgentConfig::from_file("config.toml").unwrap_or_default());

    // Disable memory for now
    config.agent.use_memory = false;

    // Define tasks with type-safe task types
    println!("\n📋 Creating enhanced project plan...");

    // Phase 1: Simulation Design
    let task1_id = Uuid::new_v4().to_string();
    workspace.add_task(WorkspaceTask {
        id: task1_id.clone(),
        description: "Design physics-accurate humanoid simulation environment".to_string(),
        task_type: TaskType::DesignEnvironment {
            requirements: vec![
                "Accurate contact dynamics".to_string(),
                "Real-time rendering".to_string(),
                "Sensor simulation".to_string(),
            ],
        },
        assigned_to: AgentType::SimulationEngineer,
        status: TaskStatus::Pending,
        phase: 1,
        dependencies: vec![],
        priority: TaskPriority::Critical,
    });

    // Phase 2: Hardware Prototyping
    let task2_id = Uuid::new_v4().to_string();
    workspace.add_task(WorkspaceTask {
        id: task2_id.clone(),
        description: "Prototype humanoid robot with dexterous hands".to_string(),
        task_type: TaskType::PrototypeHardware {
            hardware_spec: "Humanoid with 7-DOF arms and 5-finger hands".to_string(),
        },
        assigned_to: AgentType::SimulationEngineer,
        status: TaskStatus::Pending,
        phase: 2,
        dependencies: vec![task1_id.clone()],
        priority: TaskPriority::High,
    });

    // Phase 3: Distributed Training
    let task3_id = Uuid::new_v4().to_string();
    workspace.add_task(WorkspaceTask {
        id: task3_id.clone(),
        description: "Design distributed training for manipulation policy".to_string(),
        task_type: TaskType::DistributedTraining {
            num_gpus: 256,
            model_type: "Transformer-based policy".to_string(),
        },
        assigned_to: AgentType::ScalingEngineer,
        status: TaskStatus::Pending,
        phase: 3,
        dependencies: vec![task2_id.clone()],
        priority: TaskPriority::High,
    });

    // Phase 4: Edge Optimization
    let task4_id = Uuid::new_v4().to_string();
    workspace.add_task(WorkspaceTask {
        id: task4_id,
        description: "Optimize policy for on-robot deployment".to_string(),
        task_type: TaskType::OptimizeInference {
            target_latency_ms: 10.0,
            deployment_target: "edge".to_string(),
        },
        assigned_to: AgentType::ScalingEngineer,
        status: TaskStatus::Pending,
        phase: 4,
        dependencies: vec![task3_id],
        priority: TaskPriority::Critical,
    });

    // Initialize orchestrator and run
    let mut orchestrator = CollaborativeOrchestrator::new(workspace, config).await?;
    orchestrator.run().await?;

    println!("\n🎉 Enhanced collaborative workspace demonstration complete!");

    Ok(())
}
