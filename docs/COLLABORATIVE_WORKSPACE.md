# Collaborative Workspace System

The Agency provides a sophisticated system for multiple specialized agents to collaborate on projects, produce verifiable artifacts, and work together in shared workspaces.

## Overview

The Collaborative Workspace system enables:
- **Multi-agent collaboration**: Multiple specialized agents working on shared projects
- **Artifact management**: Verifiable code, configurations, and documentation
- **Cross-review**: Agents reviewing each other's work for quality assurance
- **Project coordination**: Task delegation and dependency management
- **Persistent storage**: All artifacts saved to disk with metadata

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Workspace Manager                        │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Shared Filesystem                                    │  │
│  │  ├── artifacts/     (binary files, models)            │  │
│  │  ├── code/          (Python, Rust, etc.)              │  │
│  │  ├── configs/       (YAML, TOML, JSON)                │  │
│  │  └── reports/       (Markdown, documentation)         │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Agent 1    │  │   Agent 2    │  │   Agent 3    │      │
│  │  Sim Eng.    │  │  Scaling     │  │  Coordinator │      │
│  │              │  │  Engineer    │  │              │      │
│  │  - Execute   │  │  - Execute   │  │  - Plan      │      │
│  │  - Produce   │  │  - Produce   │  │  - Review    │      │
│  │  - Review    │  │  - Review    │  │  - Report    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Key Components

### 1. Workspace

The `Workspace` manages:
- **Directory structure**: Organized folders for different artifact types
- **Artifact registry**: Metadata and content tracking
- **Agent roster**: Registered agents and their roles
- **Task queue**: Task assignment and dependency tracking

```rust
struct Workspace {
    name: String,
    directory: PathBuf,
    artifacts: HashMap<String, Artifact>,
    agents: HashMap<String, AgentRole>,
    tasks: Vec<WorkspaceTask>,
}
```

### 2. Artifacts

Artifacts are verifiable outputs produced by agents:

```rust
enum ArtifactType {
    Code { language: String, purpose: String },
    Configuration { format: String, system: String },
    Documentation { format: String },
    DataFile { format: String, size_estimate: String },
    Report { format: String },
    Model { framework: String, architecture: String },
}

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
```

Each artifact includes:
- **Unique ID**: For tracking and reference
- **Metadata**: Version, timestamps, etc.
- **Provenance**: Which agent produced it
- **Verification**: Review status and reviewer
- **Content**: The actual artifact data

### 3. Collaborative Agents

Agents with specific roles and expertise:

```rust
enum AgentRole {
    SimulationEngineer,
    ScalingEngineer,
    ProjectCoordinator,
}

struct CollaborativeAgent {
    name: String,
    role: AgentRole,
    agent: Agent,
}
```

Each agent can:
- **Execute tasks**: Generate artifacts based on requirements
- **Review artifacts**: Verify correctness and quality
- **Collaborate**: Work with other agents on shared goals

### 4. Tasks

Work items with dependencies and tracking:

```rust
struct WorkspaceTask {
    id: String,
    description: String,
    assigned_to: String,
    status: TaskStatus,
    dependencies: Vec<String>,
    artifacts_produced: Vec<String>,
}

enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}
```

## Usage Example

### Basic Workflow

```rust
// 1. Create workspace
let workspace_dir = PathBuf::from("/tmp/project");
let mut workspace = Workspace::new("my_project".to_string(), &workspace_dir)?;

// 2. Create collaborative agents
let sim_engineer = CollaborativeAgent::new(
    "Alice".to_string(),
    AgentRole::SimulationEngineer,
    config.clone(),
).await?;

let scaling_engineer = CollaborativeAgent::new(
    "Bob".to_string(),
    AgentRole::ScalingEngineer,
    config.clone(),
).await?;

// 3. Register agents
workspace.register_agent("Alice".to_string(), AgentRole::SimulationEngineer);
workspace.register_agent("Bob".to_string(), AgentRole::ScalingEngineer);

// 4. Create and assign tasks
let task = WorkspaceTask {
    id: Uuid::new_v4().to_string(),
    description: "Design MuJoCo simulation environment".to_string(),
    assigned_to: "Alice".to_string(),
    status: TaskStatus::Pending,
    dependencies: vec![],
    artifacts_produced: vec![],
};
workspace.add_task(task);

// 5. Execute task and produce artifacts
let artifacts = sim_engineer.execute_task(&task).await?;
for artifact in artifacts {
    workspace.add_artifact(artifact)?;
}

// 6. Cross-review
for artifact in workspace.artifacts.values() {
    let approved = scaling_engineer.review_artifact(artifact).await?;
    if approved {
        artifact.verify("Bob".to_string());
    }
}
```

## Collaboration Patterns

### 1. Sequential Workflow

Tasks with dependencies execute in order:

```
Task 1 (Alice) → Task 2 (Bob) → Task 3 (Charlie)
       ↓              ↓               ↓
   Artifact A    Artifact B      Report
```

### 2. Parallel Workflow

Independent tasks execute concurrently:

```
Task 1 (Alice) ─────→ Artifact A
Task 2 (Bob)   ─────→ Artifact B  ─→  Integration Task
Task 3 (Carol) ─────→ Artifact C
```

### 3. Review Workflow

Cross-functional review for quality:

```
Producer Agent → Artifact → Reviewer Agent → Verified Artifact
   (Alice)                      (Bob)            ✓
```

## Artifact Types and Examples

### Code Artifacts

```python
# Python code artifact
# File: simulation_env.py
# Produced by: SimulationEngineer_Alice
# Verified by: ScalingEngineer_Bob

import mujoco
import numpy as np

class GraspingEnvironment:
    def __init__(self, num_objects=10):
        self.model = mujoco.MjModel.from_xml_path("scene.xml")
        self.data = mujoco.MjData(self.model)
        # ... implementation
```

### Configuration Artifacts

```yaml
# training_config.yaml
# Produced by: ScalingEngineer_Bob
# Verified by: SimulationEngineer_Alice

distributed_training:
  strategy: "FSDP"
  num_gpus: 8
  batch_size: 256
  gradient_accumulation: 4
  mixed_precision: true
```

### Documentation Artifacts

```markdown
# Project Report
Produced by: Coordinator_Charlie
Date: 2024-01-15

## Overview
This report summarizes the humanoid grasping project...

## Artifacts Produced
1. simulation_env.py - MuJoCo environment
2. training_config.yaml - Distributed training setup
3. performance_analysis.md - Benchmarking results
```

## Running the Example

```bash
# Run the collaborative workspace demo
cargo run --example collaborative_robotics_workspace

# Output structure:
# 🚀 Collaborative Robotics Workspace
# ================================================================================
# 
# 📁 Workspace created: /tmp/robotics_workspace/humanoid_robot_project
# 
# 👥 Initializing agents...
#   ✓ SimulationEngineer_Alice registered
#   ✓ ScalingEngineer_Bob registered
#   ✓ Coordinator_Charlie registered
# 
# 🎯 Project: Humanoid Robot Grasping System
# --------------------------------------------------------------------------------
#   ✓ Task added: Design MuJoCo simulation environment... -> SimulationEngineer_Alice
#   ✓ Task added: Design PyTorch distributed training... -> ScalingEngineer_Bob
# 
# ⚙️  Phase 1: Simulation Environment Design
# --------------------------------------------------------------------------------
# 🔨 SimulationEngineer_Alice executing: Design MuJoCo simulation environment...
#   📄 Artifact saved: task_id_implementation.py
#   📄 Artifact saved: task_id_documentation.md
# 
# ⚙️  Phase 2: Distributed Training Setup
# --------------------------------------------------------------------------------
# 🔨 ScalingEngineer_Bob executing: Design PyTorch distributed training...
#   📄 Artifact saved: task_id_implementation.py
# 🔍 Cross-review: SimulationEngineer_Alice reviewing artifact...
#   ✅ Artifact verified
# 
# 📊 Generating Project Report
# --------------------------------------------------------------------------------
# 🔨 Coordinator_Charlie executing: Generate project summary report...
#   📄 Artifact saved: summary_report.md
# 
# ✅ Project Complete!
# ================================================================================
# Workspace: humanoid_robot_project
# Agents: 3
# Artifacts: 5
# Tasks: 3 total, 3 completed
```

## Workspace Directory Structure

After running the example:

```
/tmp/robotics_workspace/humanoid_robot_project/
├── artifacts/
├── code/
│   ├── task_1_implementation.py
│   ├── task_1_implementation.py.meta.json
│   ├── task_2_implementation.py
│   └── task_2_implementation.py.meta.json
├── configs/
├── reports/
    ├── task_1_documentation.md
    ├── task_1_documentation.md.meta.json
    ├── task_2_documentation.md
    ├── task_2_documentation.md.meta.json
    ├── summary_report.md
    └── summary_report.md.meta.json
```

## Metadata Format

Each artifact has associated metadata:

```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "name": "simulation_env.py",
  "artifact_type": {
    "Code": {
      "language": "Python",
      "purpose": "MuJoCo simulation environment"
    }
  },
  "metadata": {
    "version": "1.0"
  },
  "produced_by": "SimulationEngineer_Alice",
  "reviewed_by": "ScalingEngineer_Bob",
  "created_at": "2024-01-15T10:30:00Z",
  "verified": true
}
```

## Advanced Features

### Task Dependencies

Tasks can depend on completion of other tasks:

```rust
let task2 = WorkspaceTask {
    // ...
    dependencies: vec![task1.id.clone()],
    // ...
};
```

The system ensures tasks execute in the correct order.

### Artifact Versioning

Artifacts include version metadata for tracking evolution:

```rust
artifact.metadata.insert("version".to_string(), "2.1".to_string());
artifact.metadata.insert("prev_version".to_string(), "2.0".to_string());
```

### Multi-level Review

Artifacts can be reviewed by multiple agents:

```rust
// Initial review
artifact.verify("Alice".to_string());

// Second-level review
if needs_additional_review {
    let approved = senior_agent.review_artifact(&artifact).await?;
}
```

## Best Practices

### 1. Clear Task Descriptions

```rust
// ✓ Good
"Design MuJoCo simulation environment for grasping tasks with 10+ diverse objects"

// ✗ Bad
"Do simulation stuff"
```

### 2. Atomic Artifacts

Each artifact should be self-contained and focused:

```rust
// ✓ Good: Separate artifacts
- simulation_env.py
- environment_config.yaml
- README.md

// ✗ Bad: Monolithic
- everything.py (thousands of lines)
```

### 3. Verification Standards

Always verify artifacts before marking as complete:

```rust
let approved = reviewer.review_artifact(&artifact).await?;
if approved {
    artifact.verify(reviewer.name.clone());
    workspace.add_artifact(artifact)?;
}
```

### 4. Meaningful Metadata

Include relevant context in artifact metadata:

```rust
artifact.metadata.insert("model_version".to_string(), "llama3.2".to_string());
artifact.metadata.insert("generation_time_ms".to_string(), "1523".to_string());
artifact.metadata.insert("token_count".to_string(), "450".to_string());
```

## Integration with A2A

For distributed agent networks, integrate with the A2A system:

```rust
use the_agency::{A2AClient, HttpA2AClient, MessagePayload};

// Notify other agents of artifact completion
let message = A2AMessage {
    from: agent_id.clone(),
    to: coordinator_id.clone(),
    payload: MessagePayload::Event {
        event_type: "artifact_completed".to_string(),
        data: serde_json::to_value(&artifact)?,
    },
    // ...
};

a2a_client.send_message(message).await?;
```

## Troubleshooting

### Issue: Artifacts not being saved

**Solution**: Check filesystem permissions and disk space

```bash
ls -la /tmp/robotics_workspace/
df -h /tmp
```

### Issue: Tasks blocked on dependencies

**Solution**: Verify dependency task IDs are correct

```rust
// Check task dependencies
for task in &workspace.tasks {
    println!("Task: {} depends on: {:?}", task.id, task.dependencies);
}
```

### Issue: Review failures

**Solution**: Provide more context in review prompts

```rust
let prompt = format!(
    "Review this artifact:\n{}\n\n\
    Context: This is part of the {} project\n\
    Requirements: {}\n\
    Verify: correctness, completeness, production-readiness",
    artifact.content,
    project_name,
    requirements
);
```

## Performance Considerations

- **Concurrent execution**: Use async/await for parallel task processing
- **Artifact caching**: Enable LLM response caching for faster iterations
- **Batch operations**: Group similar tasks for efficiency
- **Incremental verification**: Review artifacts as produced, not all at once

## See Also

- [Robotics Agents Examples](../examples/ROBOTICS_AGENTS.md)
- [A2A Communication](A2A.md)
- [Agent Architecture](../README.md#agent-architecture)
- [Workflow System](WORKFLOWS.md)

---

**Note**: The collaborative workspace system is designed for production use. All artifacts are persisted to disk with full metadata and verification trails.
