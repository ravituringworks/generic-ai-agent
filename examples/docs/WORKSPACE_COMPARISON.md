# Collaborative Workspace Comparison

## Two Implementations Available

The Agency framework provides two collaborative workspace examples demonstrating different complexity levels and use cases.

---

## 📊 Quick Comparison

| Feature | Simple Workspace | Complex Workspace |
|---------|------------------|-------------------|
| **File** | `collaborative_robotics_workspace.rs` | `collaborative_robotics_complex.rs` |
| **Agents** | 3 (EMP001, EMP002, EMP003) | 4 (EMP001, EMP002, Dana, EMP003) |
| **Roles** | Simulation, Scaling, Coordinator | + Configuration Specialist |
| **Phases** | 1 (single execution) | 4 (multi-phase pipeline) |
| **Tasks** | 1 | 5 |
| **Dependencies** | None | Multi-level DAG |
| **Execution** | Sequential only | Parallel + Sequential |
| **Artifact Types** | 2 (Code, Docs) | 7 (Code, Config, Model, Report, Test, Benchmark, Docs) |
| **Runtime** | ~30 seconds | ~2-3 minutes |
| **Output** | ~200 lines code+docs | ~1000+ lines across types |
| **LOC** | 481 lines | 674 lines |
| **Complexity** | Beginner-friendly | Production-ready |

---

## Simple Workspace (Basic)

### Use Case (Basic)

Perfect for **learning** and **prototyping** multi-agent collaboration.

### What It Does (Basic)

Creates a single robotics simulation class with:

- ✅ Basic Python implementation
- ✅ Documentation
- ✅ Cross-agent review
- ✅ Artifact verification

### Architecture (Basic)

```text
SimulationEngineer_EMP001
    ↓ (generates)
[Python Code + Docs]
    ↓ (reviews)
ScalingEngineer_EMP002
    ↓ (approves)
[Verified Artifacts]
```

### When to Use (Basic)

- 🎓 Learning multi-agent systems
- 🔬 Prototyping collaboration patterns
- ⚡ Quick demonstrations
- 🧪 Testing agent interactions
- 📚 Educational examples

### Running the Simple Workspace

```bash
cargo run --example collaborative_robotics_workspace
```

**Output**: 2 artifacts (1 Python file + 1 Markdown doc)  
**Time**: 30 seconds
**Workspace**: `output/robotics_workspace/humanoid_robot_project/`

---

## Complex Workspace (Advanced)

### Use Case (Advanced)

Demonstrates **production-grade** multi-agent workflows for complex projects.

### What It Does (Advanced)

Builds a complete humanoid robot manipulation system across 4 phases:

#### Phase 1: Foundation (Parallel)

- **SimulationEngineer**: Creates 3D simulation environment
- **ConfigSpecialist**: Generates URDF robot model
- Both execute **simultaneously**

#### Phase 2: Control (Sequential)

- **SimulationEngineer**: Implements inverse kinematics controller
- Waits for Phase 1 completion

#### Phase 3: Training (Sequential)

- **ScalingEngineer**: Builds distributed training pipeline
- Waits for Phase 2 completion

#### Phase 4: Documentation (Sequential)

- **Coordinator**: Generates comprehensive project report
- Waits for all phases

### Architecture (Advanced)

```text
Phase 1 (Parallel):
    SimulationEngineer_EMP001 → [Environment Code]
    ConfigSpecialist_Dana    → [URDF Model]
         ↓                          ↓
Phase 2 (Sequential):
    SimulationEngineer_EMP001 → [IK Controller] (depends on Phase 1)
         ↓
Phase 3 (Sequential):
    ScalingEngineer_EMP002 → [Training Pipeline] (depends on Phase 2)
         ↓
Phase 4 (Sequential):
    Coordinator_EMP003 → [Final Report] (depends on all)
```

### Key Features

#### Dependency Management

- Automatic dependency resolution
- Blocked tasks wait for prerequisites
- Topological sorting for execution order

#### Parallel Execution

- Independent tasks run simultaneously
- Reduces total execution time
- Maximizes agent utilization

#### Diverse Artifacts

- Python/Rust code implementations
- URDF/XML robot configurations
- Training pipelines and benchmarks
- Integration guides and reports

#### Advanced Tracking

- Phase-based organization
- Priority scheduling (Critical → High → Medium → Low)
- Full artifact lineage

### When to Use (Advanced)

- 🏭 Production robotics projects
- 🔄 Multi-stage software pipelines
- 📊 Research workflows with phases
- 🤖 ML model development lifecycles
- 🏗️ System architecture projects
- 📈 Scalable agent coordination

### Running the Complex Workspace

```bash
cargo run --example collaborative_robotics_complex
```

**Output**: 10-15 artifacts across 4 phases  
**Time**: 2-3 minutes
**Workspace**: `output/robotics_workspace_complex/humanoid_manipulation_system/`

---

## Feature Breakdown

### Simple Workspace Includes

✅ Basic multi-agent collaboration  
✅ Artifact generation and storage  
✅ Cross-agent review workflow  
✅ Verification tracking  
✅ JSON metadata  
✅ SQLite persistence  

### Complex Workspace Adds

✅ Multi-phase workflows  
✅ Task dependency graphs (DAG)  
✅ Parallel task execution  
✅ 4th agent (Configuration Specialist)  
✅ Priority-based scheduling  
✅ Phase-aware artifact routing  
✅ Extended artifact types  
✅ Automatic dependency resolution  

---

## Performance Comparison

### Simple Workspace Performance

```text
Initialization:   5s
Task Execution:  25s
Review & Save:    5s
──────────────────────
Total:          ~30s
```

### Complex Workspace Performance

```text
Initialization:      5s
Phase 1 (2 tasks):  45s  ← Parallel execution
Phase 2 (1 task):   30s
Phase 3 (1 task):   30s
Phase 4 (1 task):   30s
──────────────────────────
Total:            ~140s
```

**Efficiency Gain**: Phase 1 runs 2 tasks in ~45s instead of 60s (25% faster)

---

## Choosing the Right Workspace

### Start with Simple If

- 🌱 New to multi-agent systems
- 📖 Learning The Agency framework
- ⚡ Need quick results
- 🎯 Single-phase workflow
- 🧪 Prototyping concepts

### Use Complex If

- 🏭 Building production systems
- 🔄 Multi-phase projects
- 📊 Complex dependencies
- ⚡ Need parallelization
- 🎯 Diverse artifact types
- 📈 Scaling to many agents

---

## Migration Path

Start simple, grow complex:

1. **Learn**: Run simple workspace
2. **Understand**: Study collaboration patterns
3. **Extend**: Add a phase to simple workspace
4. **Advance**: Switch to complex workspace
5. **Customize**: Adapt to your use case

---

## Common Use Cases by Type

### Simple Workspace

```text
✓ Code generation demos
✓ Tutorial examples
✓ Single-feature development
✓ Quick prototypes
✓ Educational content
```

### Complex Workspace

```text
✓ Full robotics stack development
✓ ML pipeline orchestration
✓ Research project workflows
✓ System integration projects
✓ Multi-team collaboration
```

---

## Extensibility

Both workspaces are designed to be extended:

### Add an Agent

```rust
let new_agent = CollaborativeAgent::new(
    "SecurityAuditor_Eve".to_string(),
    AgentRole::SecurityAnalyst,
    config.clone(),
).await?;
```

### Add a Task

```rust
let new_task = WorkspaceTask {
    id: Uuid::new_v4().to_string(),
    description: "Run security audit".to_string(),
    assigned_to: new_agent.name.clone(),
    status: TaskStatus::Pending,
    phase: 5,  // New phase
    dependencies: vec![task4_id],  // Depends on previous
    ...
};
```

### Add an Artifact Type

```rust
enum ArtifactType {
    // Existing types...
    SecurityAudit {
        severity: String,
        findings: Vec<String>,
    },
}
```

---

## Documentation

- **Simple**: [examples/COLLABORATIVE_WORKSPACE_SUCCESS.md](COLLABORATIVE_WORKSPACE_SUCCESS.md)
- **Complex**: [examples/COMPLEX_WORKSPACE.md](COMPLEX_WORKSPACE.md)
- **Architecture**: [docs/COLLABORATIVE_WORKSPACE.md](../docs/COLLABORATIVE_WORKSPACE.md)

---

## Try Both

```bash
# Simple workspace (30 seconds)
cargo run --example collaborative_robotics_workspace

# Complex workspace (2-3 minutes)
cargo run --example collaborative_robotics_complex
```

Both examples showcase The Agency's powerful multi-agent collaboration capabilities - pick the one that matches your needs!

---

**Recommendation**: Start with the simple workspace to understand the concepts, then explore the complex workspace for production-ready patterns.
