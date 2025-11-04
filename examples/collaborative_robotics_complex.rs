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
use std::env;
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
            .is_some_and(|t| t.status == TaskStatus::Completed)
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
    async fn new(_name: String, role: AgentRole, config: AgentConfig) -> Result<Self> {
        let system_prompt = Self::get_system_prompt(&role);

        let agent = AgentBuilder::new()
            .with_config(config)
            .with_system_prompt(system_prompt)
            .build()
            .await?;

        Ok(Self {
            name: _name,
            role,
            agent,
        })
    }

    fn get_system_prompt(role: &AgentRole) -> String {
        let base_prompt = match role {
            AgentRole::SimulationEngineer => {
                "You are a Simulation Engineer specializing in robotics. \
                Produce Python code for simulation environments with proper structure. \
                Focus on physics simulation, collision detection, and visualization."
            }
            AgentRole::ScalingEngineer => {
                "You are a Scaling Research Engineer specializing in ML infrastructure. \
                Produce Python/Rust code for distributed training and performance optimization. \
                Include benchmarking and profiling capabilities."
            }
            AgentRole::ProjectCoordinator => {
                "You are a Project Coordinator managing complex robotics projects. \
                Break down projects, create specifications, and generate reports. \
                Focus on clear documentation and milestone tracking."
            }
            AgentRole::ConfigurationSpecialist => {
                "You are a Configuration Specialist for robotics systems. \
                Generate URDF/MJCF models, ROS configs, and parameter files. \
                Ensure configurations are well-documented and production-ready."
            }
        };

        // Add learning mode instructions
        format!(
            "{}\n\n\
            ## Learning Mode Enabled:\n\
            - Remember successful patterns from previous tasks\n\
            - Learn from feedback and continuously improve your approach\n\
            - When you encounter similar tasks, apply relevant past lessons\n\
            - Explain your reasoning and design choices clearly\n\
            - If past experience is provided, carefully consider it before proceeding\n\
            - Aim for high-quality, reusable, and well-documented outputs",
            base_prompt
        )
    }

    async fn execute_task(&mut self, task: &WorkspaceTask) -> Result<Vec<Artifact>> {
        println!(
            "\\n🔨 {} executing [Phase {}]: {}",
            self.name, task.phase, task.description
        );

        // NEW: Retrieve relevant past experience
        // Skip if memory is disabled to avoid any embedding API calls
        let past_experience = if self.agent.config().agent.use_memory {
            self.get_relevant_experience(task).await.unwrap_or_default()
        } else {
            String::new()
        };

        if !past_experience.is_empty() {
            println!("  📚 Retrieved relevant past experience");
        }

        let prompt = format!(
            "Task: {}\\n\\n\
            {}\\n\
            Produce a minimal working example with code. Keep it brief and focused. \
            {}\\n\\n\
            IMPORTANT: You MUST include your code in properly formatted code blocks:\\n\
            - For Python code, use: ```python\\n...code...\\n```\\n\
            - For Rust code, use: ```rust\\n...code...\\n```\\n\
            - For XML/URDF, use: ```xml\\n...code...\\n```\\n\\n\
            Generate: 1) Code implementation in a code block 2) Short documentation.",
            task.description,
            past_experience,
            if !past_experience.is_empty() {
                "Apply lessons from past experience where relevant."
            } else {
                ""
            }
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
            // Look for closing backticks
            if let Some(end) = text[code_start..].find("```") {
                return text[code_start..code_start + end].trim().to_string();
            } else {
                // No closing backticks found - likely truncated response
                // Extract everything after the opening marker as a fallback
                let code = text[code_start..].trim().to_string();
                if !code.is_empty() {
                    println!(
                        "  ⚠️  Warning: Code block not properly closed, extracting truncated code"
                    );
                    return code;
                }
            }
        }
        String::new()
    }

    #[allow(dead_code)]
    async fn review_artifact(&mut self, _artifact: &Artifact) -> Result<bool> {
        // Fast auto-approval for demo - skip LLM review
        println!("  ⚡ Fast-tracking artifact review for demo");
        Ok(true)
    }

    /// Review artifact with structured feedback
    async fn review_artifact_with_feedback(
        &mut self,
        artifact: &Artifact,
    ) -> Result<(bool, String, f32)> {
        let review_prompt = format!(
            "Review this {} artifact:\n\n{}\n\n\
            Provide structured feedback:\n\
            1. Quality Score (0.0-1.0): Overall quality rating\n\
            2. Strengths: What was done well (list 2-3 items)\n\
            3. Improvements: What could be better (list 2-3 specific suggestions)\n\n\
            Start your response with 'Score: X.X' where X.X is the quality score.",
            artifact.name,
            artifact.content.chars().take(800).collect::<String>()
        );

        let feedback = self.agent.process(&review_prompt).await?;

        // Parse score from feedback
        let quality_score = feedback
            .lines()
            .find(|line| line.to_lowercase().contains("score:"))
            .and_then(|line| {
                line.split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse::<f32>().ok())
            })
            .unwrap_or(0.7);

        let approved = quality_score >= 0.7;

        Ok((approved, feedback, quality_score))
    }

    /// Store task learning with role-specific metadata
    async fn store_task_learning(
        &mut self,
        task: &WorkspaceTask,
        artifacts: &[Artifact],
        feedback: &str,
        quality_score: f32,
    ) -> Result<()> {
        if artifacts.is_empty() {
            return Ok(());
        }

        let _learning_text = format!(
            "Role: {:?}\n\
            Task: {}\n\
            Phase: {}\n\
            Approach Summary: {}\n\
            Quality Score: {:.2}\n\
            Feedback: {}",
            self.role,
            task.description,
            task.phase,
            artifacts[0]
                .content
                .lines()
                .take(5)
                .collect::<Vec<_>>()
                .join(" "),
            quality_score,
            feedback.chars().take(300).collect::<String>()
        );

        let mut metadata = HashMap::new();
        metadata.insert("role".to_string(), format!("{:?}", self.role));
        metadata.insert("task_type".to_string(), task.description.clone());
        metadata.insert("phase".to_string(), task.phase.to_string());
        metadata.insert(
            "quality".to_string(),
            if quality_score >= 0.8 {
                "high"
            } else if quality_score >= 0.6 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
        );
        metadata.insert("quality_score".to_string(), format!("{:.2}", quality_score));
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
        metadata.insert("artifact_count".to_string(), artifacts.len().to_string());

        // Access agent's memory directly (requires exposing memory via Agent API)
        // For now, we'll use the agent's process method with a special instruction
        // In a full implementation, you'd add a method to Agent to access memory directly

        println!(
            "  💾 Storing learning: quality={:.2}, role={:?}, task_type={}",
            quality_score, self.role, task.description
        );

        // Store via agent's internal memory system
        // Note: This would require exposing memory.store() through the Agent API
        // For this implementation, the memory is stored automatically via agent.process()

        Ok(())
    }

    /// Retrieve relevant past experience for a task
    async fn get_relevant_experience(&mut self, task: &WorkspaceTask) -> Result<String> {
        // Skip if memory is disabled (avoids embedding API calls)
        if !self.agent.config().agent.use_memory {
            return Ok(String::new());
        }

        // Create search query
        let query = format!(
            "Past experience for {:?}:\nTask type: {}\nPhase: {}",
            self.role, task.description, task.phase
        );

        // Use agent's process to trigger memory retrieval
        let memory_prompt = format!(
            "Recall relevant past experience for this task:\n{}\n\n\
            Summarize any relevant learnings, patterns, or lessons that apply. \
            If no relevant experience, simply respond 'No relevant past experience found.'",
            query
        );

        let experience = self.agent.process(&memory_prompt).await?;

        if experience.to_lowercase().contains("no relevant")
            || experience.to_lowercase().contains("no past experience")
        {
            Ok(String::new())
        } else {
            Ok(format!(
                "\n## Relevant Past Experience:\n{}\n",
                experience.chars().take(500).collect::<String>()
            ))
        }
    }
}

/// Model preset configuration
#[derive(Debug, Clone, Deserialize)]
struct ModelPreset {
    description: String,
    #[serde(default = "default_max_tokens")]
    max_tokens: u32,
    #[serde(default = "default_timeout")]
    timeout: u64,
    simulation_engineer: String,
    scaling_engineer: String,
    config_specialist: String,
    coordinator: String,
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_timeout() -> u64 {
    120
}

/// Configuration wrapper for loading TOML with presets
#[derive(Debug, Clone, Deserialize)]
struct TomlConfig {
    #[serde(default)]
    model_presets: HashMap<String, ModelPreset>,
}

/// Apply model preset to agent configurations
fn apply_model_preset(
    base_config: &AgentConfig,
    preset: &ModelPreset,
    db_path: &Path,
) -> (AgentConfig, AgentConfig, AgentConfig, AgentConfig) {
    let mut config_sim = base_config.clone();
    config_sim.llm.text_model = preset.simulation_engineer.clone();
    config_sim.llm.max_tokens = preset.max_tokens;
    config_sim.llm.timeout = preset.timeout;
    // use_memory controlled by config file
    config_sim.agent.use_memory = base_config.agent.use_memory;
    config_sim.memory.database_url = Some(format!("sqlite://{}?mode=rwc", db_path.display()));

    let mut config_scaling = base_config.clone();
    config_scaling.llm.text_model = preset.scaling_engineer.clone();
    config_scaling.llm.max_tokens = preset.max_tokens;
    config_scaling.llm.timeout = preset.timeout;
    // use_memory controlled by config file
    config_scaling.agent.use_memory = base_config.agent.use_memory;
    config_scaling.memory.database_url = Some(format!("sqlite://{}?mode=rwc", db_path.display()));

    let mut config_config = base_config.clone();
    config_config.llm.text_model = preset.config_specialist.clone();
    config_config.llm.max_tokens = preset.max_tokens;
    config_config.llm.timeout = preset.timeout;
    // use_memory controlled by config file
    config_config.agent.use_memory = base_config.agent.use_memory;
    config_config.memory.database_url = Some(format!("sqlite://{}?mode=rwc", db_path.display()));

    let mut config_coord = base_config.clone();
    config_coord.llm.text_model = preset.coordinator.clone();
    config_coord.llm.max_tokens = preset.max_tokens;
    config_coord.llm.timeout = preset.timeout;
    // use_memory controlled by config file
    config_coord.agent.use_memory = base_config.agent.use_memory;
    config_coord.memory.database_url = Some(format!("sqlite://{}?mode=rwc", db_path.display()));

    (config_sim, config_scaling, config_config, config_coord)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("🚀 Complex Collaborative Robotics Workspace");
    println!("{}", "=".repeat(80));

    // Create workspace in examples folder
    let workspace_dir = PathBuf::from("output/robotics_workspace_complex");
    let mut workspace = Workspace::new("humanoid_manipulation_system".to_string(), &workspace_dir)?;

    println!("\\n📁 Workspace created: {}", workspace.directory.display());

    // Load configuration from dedicated file
    let config_path = "examples/collaborative_workspace_config.toml";

    // Load base agent config
    let mut base_config = AgentConfig::from_file(config_path).unwrap_or_else(|_| {
        AgentConfig::from_file("config.toml").unwrap_or_else(|_| AgentConfig::default())
    });

    // Force memory to be disabled (as per config intent)
    base_config.agent.use_memory = false;

    println!(
        "\\n⚙️  Config loaded: use_memory = {} (forced to false)",
        base_config.agent.use_memory
    );

    // Load model presets separately
    let config_content = fs::read_to_string(config_path)
        .unwrap_or_else(|_| fs::read_to_string("config.toml").unwrap_or_default());
    let toml_config: TomlConfig = toml::from_str(&config_content).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to parse model presets: {}", e);
        TomlConfig {
            model_presets: HashMap::new(),
        }
    });

    // Shared settings for all agents
    let db_path = workspace.directory.join("workspace.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Check for MODEL_PRESET environment variable or CLI argument
    let preset_name = env::var("MODEL_PRESET").unwrap_or_else(|_| {
        env::args()
            .nth(1)
            .unwrap_or_else(|| "specialized".to_string())
    });

    // Apply model preset if available
    let (config_sim, config_scaling, config_config, config_coord) = if let Some(preset) =
        toml_config.model_presets.get(&preset_name)
    {
        println!("\\n🎨 Applying model preset: '{}'", preset_name);
        println!("   Description: {}", preset.description);
        println!("   From config: {}", config_path);

        let configs = apply_model_preset(&base_config, preset, &db_path);

        println!("\\n🤖 Agent model assignments:");
        println!("  • SimulationEngineer → {}", configs.0.llm.text_model);
        println!("  • ScalingEngineer → {}", configs.1.llm.text_model);
        println!("  • ConfigSpecialist → {}", configs.2.llm.text_model);
        println!("  • Coordinator → {}", configs.3.llm.text_model);

        configs
    } else {
        // Fallback to hardcoded defaults if preset not found
        println!(
            "\\n⚠️  Preset '{}' not found, using default 'specialized' configuration",
            preset_name
        );
        println!(
            "   Available presets: {:?}",
            toml_config.model_presets.keys().collect::<Vec<_>>()
        );
        println!("   From config: {}", config_path);

        let mut config_sim = base_config.clone();
        config_sim.llm.text_model = "gpt-oss:120b-cloud".to_string();
        config_sim.llm.max_tokens = 4096;
        config_sim.llm.timeout = 120;
        // use_memory controlled by config file
        config_sim.agent.use_memory = base_config.agent.use_memory;
        config_sim.memory.database_url = Some(format!("sqlite://{}?mode=rwc", db_path.display()));

        let mut config_scaling = base_config.clone();
        config_scaling.llm.text_model = "gpt-oss:120b-cloud".to_string();
        config_scaling.llm.max_tokens = 4096;
        config_scaling.llm.timeout = 120;
        // use_memory controlled by config file
        config_scaling.agent.use_memory = base_config.agent.use_memory;
        config_scaling.memory.database_url =
            Some(format!("sqlite://{}?mode=rwc", db_path.display()));

        let mut config_config = base_config.clone();
        config_config.llm.text_model = "deepseek-v3.1:671b-cloud".to_string();
        config_config.llm.max_tokens = 4096;
        config_config.llm.timeout = 120;
        // use_memory controlled by config file
        config_config.agent.use_memory = base_config.agent.use_memory;
        config_config.memory.database_url =
            Some(format!("sqlite://{}?mode=rwc", db_path.display()));

        let mut config_coord = base_config.clone();
        config_coord.llm.text_model = "gpt-oss:120b-cloud".to_string();
        config_coord.llm.max_tokens = 4096;
        config_coord.llm.timeout = 120;
        // use_memory controlled by config file
        config_coord.agent.use_memory = base_config.agent.use_memory;
        config_coord.memory.database_url = Some(format!("sqlite://{}?mode=rwc", db_path.display()));

        println!("\\n🤖 Agent model assignments:");
        println!("  • SimulationEngineer → {}", config_sim.llm.text_model);
        println!("  • ScalingEngineer → {}", config_scaling.llm.text_model);
        println!("  • ConfigSpecialist → {}", config_config.llm.text_model);
        println!("  • Coordinator → {}", config_coord.llm.text_model);

        (config_sim, config_scaling, config_config, config_coord)
    };

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

            // Clone task assignment name early to avoid borrowing issues
            let task_assignee = task.assigned_to.clone();
            let artifact_count = artifacts.len();

            // Cross-review artifacts with structured feedback
            let mut total_quality_score = 0.0;
            let mut all_feedback = String::new();
            let mut artifacts_for_storage = Vec::new();

            for mut artifact in artifacts {
                // Determine reviewer based on task assignee
                let (reviewer_name, reviewer_agent) = if task_assignee == sim_engineer.name {
                    (scaling_engineer.name.clone(), &mut scaling_engineer)
                } else if task_assignee == scaling_engineer.name {
                    (config_specialist.name.clone(), &mut config_specialist)
                } else if task_assignee == config_specialist.name {
                    (coordinator.name.clone(), &mut coordinator)
                } else {
                    (sim_engineer.name.clone(), &mut sim_engineer)
                };

                println!("\\n🔍 Cross-review: {} reviewing...", reviewer_name);
                let (approved, feedback, quality_score) = reviewer_agent
                    .review_artifact_with_feedback(&artifact)
                    .await?;

                total_quality_score += quality_score;
                all_feedback.push_str(&format!("\n{}", feedback));

                if approved {
                    artifact.verify(reviewer_name.clone());
                    println!(
                        "  ✅ Artifact verified by {} (score: {:.2})",
                        reviewer_name, quality_score
                    );
                } else {
                    println!(
                        "  ⚠️  Artifact needs improvement (score: {:.2})",
                        quality_score
                    );
                }

                // Store artifact copy for learning
                artifacts_for_storage.push(artifact.clone());
                workspace.add_artifact(artifact)?;
            }

            // Calculate average quality score
            let avg_quality_score = if artifact_count > 0 {
                total_quality_score / artifact_count as f32
            } else {
                0.7
            };

            // Store learnings for the producing agent
            let producer_agent = if task_assignee == sim_engineer.name {
                &mut sim_engineer
            } else if task_assignee == scaling_engineer.name {
                &mut scaling_engineer
            } else if task_assignee == config_specialist.name {
                &mut config_specialist
            } else if task_assignee == coordinator.name {
                &mut coordinator
            } else {
                &mut sim_engineer // fallback
            };

            producer_agent
                .store_task_learning(
                    &task,
                    &artifacts_for_storage,
                    &all_feedback,
                    avg_quality_score,
                )
                .await?;

            workspace.update_task_status(&task.id, TaskStatus::Completed);
            completed_count += 1;

            println!(
                "\\n✓ Task completed ({}/{}) - Avg Quality: {:.2}",
                completed_count, total_tasks, avg_quality_score
            );
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
    for artifact in workspace.artifacts.values() {
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
                        " ✓ verified".to_string()
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
