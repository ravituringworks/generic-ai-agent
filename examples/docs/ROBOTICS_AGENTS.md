# Robotics AI Agent Examples

This directory contains specialized AI agents designed for robotics and AI research roles, inspired by real job descriptions from leading robotics companies.

## 🤖 Available Agents

### 1. Simulation Engineer Agent (`simulation_engineer_agent.rs`)

**Role**: Design and build simulation environments and real-time infrastructure to accelerate robot learning.

**Capabilities**:

- Design diverse, physically realistic simulation environments
- Bridge sim-to-real gap between policies and real robots
- Scale up simulation data production for research
- Prototype virtual robot hardware
- Optimize performance of physics simulators and rendering pipelines
- Generate comprehensive test suites for simulation correctness

**Key Expertise**:

- Physics simulators (MuJoCo, PyBullet, Isaac Sim)
- OpenGL rendering pipelines
- Sim-to-real transfer techniques
- Large-scale data generation
- Performance optimization
- Testing and validation

**Example Tasks**:

1. Design home environment simulation with realistic physics
2. Analyze sim-to-real gap for grasping policies
3. Scale data production to 1M samples
4. Prototype 3-finger adaptive gripper hardware
5. Optimize rendering bottlenecks
6. Generate physics correctness test suites

**Run the example**:

```bash
cargo run --example simulation_engineer_agent
```

---

### 2. Scaling Research Engineer Agent (`scaling_engineer_agent.rs`)

**Role**: Scale training, evaluation, and deployment infrastructure at massive scale (1000+ GPUs) across robot fleets.

**Capabilities**:

- Design and lead scaling of distributed training systems
- Optimize inference for datacenter and edge deployment
- Ensure fault tolerance and reliability at scale
- Handle massive datasets and data pipeline optimization
- Analyze scaling laws and predict performance
- Optimize CUDA/Triton kernels for maximum performance

**Key Expertise**:

- Distributed training (PyTorch FSDP/DDP, DeepSpeed, TorchTitan)
- Multi-node debugging and experiment tracking
- Inference optimization (TensorRT, ONNX, vLLM)
- Quantization strategies (INT8/FP8, PTQ, QAT)
- CUDA kernel optimization
- Hardware utilization (tensor cores, memory hierarchies)
- Scaling laws and bottleneck analysis

**Example Tasks**:

1. Design 1024 GPU distributed training system
2. Optimize datacenter inference for 10K req/s
3. Optimize edge deployment with 10ms latency budget
4. Design fault-tolerant training for 512 GPU cluster
5. Optimize data pipeline for 500TB dataset
6. Analyze scaling laws for 10B parameter models
7. Optimize Flash Attention CUDA kernels

**Run the example**:

```bash
cargo run --example scaling_engineer_agent
```

---

## 🎯 Job Requirements Covered

### Simulation Engineer

- ✅ **Languages**: Rust, Python, C++ support
- ✅ **Simulators**: MuJoCo, PyBullet, Isaac Sim expertise
- ✅ **Rendering**: OpenGL pipeline optimization
- ✅ **Testing**: Comprehensive simulation stack testing
- ✅ **Technical challenges**: Sim-to-real gap, performance optimization
- ✅ **Collaboration**: Cross-functional with AI and hardware teams

### Scaling Research Engineer

- ✅ **Languages**: Rust, Python, C++ support
- ✅ **Distributed Training**: PyTorch, DeepSpeed, FSDP/ZeRO
- ✅ **Scaling mindset**: 1000+ GPU scale, fault tolerance
- ✅ **Inference**: TensorRT, quantization, batching strategies
- ✅ **Low-level optimization**: CUDA/Triton kernel tuning
- ✅ **Hardware understanding**: Tensor cores, memory hierarchies
- ✅ **Production systems**: Edge deployment, OTA updates

---

## 🏗️ Architecture

Both agents are built using The Agency's core components:

```rust
use the_agency::{Agent, AgentBuilder, AgentConfig};

// Create specialized agent with custom system prompt
let agent = AgentBuilder::new()
    .with_config(config)
    .with_system_prompt(expert_knowledge.to_string())
    .build()
    .await?;

// Process tasks with domain expertise
let response = agent.process(&task_description).await?;
```

### Key Features

- **Task-specific LLM models**: Automatically select best model per task
- **Response caching**: Speed up repeated queries
- **Memory system**: Context retention across conversations
- **Workflow integration**: Complex multi-step reasoning
- **Tool calling**: Extensible with MCP tools

---

## 🚀 Getting Started

### Prerequisites

1. **Ollama** must be running with appropriate models:

```bash
ollama serve
ollama pull llama3.2          # General model
ollama pull qwen2.5-coder:7b  # Code generation
ollama pull qwen3-coder:480b-cloud  # Code generation
ollama pull qwen2.5:7b        # Technical analysis
```

1. **Configuration**: Update `config.toml` if needed

```toml
[llm]
text_model = "qwen3-coder:480b-cloud"
temperature = 0.7

[llm.cache]
enabled = true
max_entries = 1000
```

### Running the Examples

**Simulation Engineer**:

```bash
# Full demo with all tasks
cargo run --example simulation_engineer_agent

# Expected output:
# 🚀 Simulation Engineer Agent Demo
# ================================================================================
# Role: Design and build simulation environments for robot learning
#
# 📋 Task 1: Design Home Environment Simulation
# --------------------------------------------------------------------------------
# 🏗️  New Environment Designed:
# ...
```

**Scaling Engineer**:

```bash
# Full demo with all tasks
cargo run --example scaling_engineer_agent

# Expected output:
# 🚀 Scaling Research Engineer Agent Demo
# ================================================================================
# Role: Scale training, evaluation, and deployment infrastructure
#
# 📋 Task 1: Design Large-Scale Distributed Training
# --------------------------------------------------------------------------------
# 🔧 Distributed Training Design:
# ...
```

---

## 📊 Performance Characteristics

### Simulation Engineer Agent (Performance)

| Task | Complexity | Avg Response Time* |
|------|-----------|-------------------|
| Environment Design | High | 15-30s |
| Sim-to-Real Analysis | Medium | 10-20s |
| Scaling Plan | High | 20-40s |
| Hardware Prototype | Medium | 15-25s |
| Performance Optimization | High | 15-30s |
| Test Suite Generation | Medium | 10-20s |

### Scaling Engineer Agent (Performance)

| Task | Complexity | Avg Response Time* |
|------|-----------|-------------------|
| Distributed Training | Very High | 30-60s |
| Datacenter Inference | High | 20-40s |
| Edge Optimization | High | 20-40s |
| Fault Tolerance | High | 25-45s |
| Data Pipeline | Medium | 15-30s |
| Scaling Laws | Medium | 15-30s |
| CUDA Optimization | Very High | 30-60s |

*With caching enabled, repeated queries return in <1ms

---

## 🛠️ Customization

### Add New Tasks

Both agents can be extended with new methods:

```rust
impl SimulationEngineerAgent {
    /// Custom task method
    pub async fn design_custom_sensor(&mut self, spec: &str) -> Result<String> {
        let prompt = format!(
            "Design a custom sensor for robotics:\n{}\n\
            Provide: specifications, simulation model, and integration code",
            spec
        );
        
        let response = self.agent.process(&prompt).await?;
        println!("🔬 Custom Sensor Design:\n{}", response);
        Ok(response)
    }
}
```

### Customize System Prompts

Modify the agent's expertise by updating the system prompt:

```rust
.with_system_prompt(
    "You are an expert in [DOMAIN]. \
    Your expertise includes:\n\
    - [Skill 1]\n\
    - [Skill 2]\n\
    ...\n\
    Provide detailed, production-ready solutions.".to_string()
)
```

### Use Task-Specific Models

Configure different models for different task types in `config.toml`:

```toml
[llm.task_models.code_generation]
model = "qwen2.5-coder:7b"
temperature = 0.2
keywords = ["code", "implement", "program"]

[llm.task_models.data_analysis]
model = "qwen2.5:7b"
temperature = 0.3
keywords = ["analyze", "performance", "optimization"]
```

---

## 💡 Use Cases

### Simulation Engineer Agent

1. **Research Teams**: Rapidly prototype simulation environments
2. **ML Engineers**: Analyze and close sim-to-real gaps
3. **Hardware Teams**: Virtual prototyping before physical builds
4. **Data Generation**: Scale up training data production
5. **Performance Teams**: Optimize simulation infrastructure

### Scaling Engineer Agent

1. **ML Infrastructure Teams**: Design training clusters
2. **Research Scientists**: Predict scaling behavior
3. **Deployment Engineers**: Optimize edge inference
4. **Platform Teams**: Build fault-tolerant systems
5. **Performance Engineers**: Low-level kernel optimization

---

## 🤝 Collaborative Workspace

See the [Collaborative Workspace Example](collaborative_robotics_workspace.rs) for agents working together:

```bash
cargo run --example collaborative_robotics_workspace
```

Features:

- **Multi-agent collaboration**: Agents working on shared projects
- **Artifact management**: Verifiable code, configs, and docs
- **Cross-review**: Quality assurance through peer review
- **Persistent storage**: All artifacts saved with metadata

Read more: [Collaborative Workspace Documentation](../docs/COLLABORATIVE_WORKSPACE.md)

---

## 📚 Related Documentation

- [Collaborative Workspace System](../docs/COLLABORATIVE_WORKSPACE.md)
- [Task-Based LLM Configuration](../docs/TASK_BASED_LLM.md)
- [LLM Response Caching](../docs/LLM_CACHE.md)
- [Agent Architecture](../README.md#agent-architecture)
- [Workflow System](../docs/WORKFLOWS.md)
- [MCP Integration](../docs/MCP.md)

---

## 🤝 Contributing

To add new robotics agents:

1. Create a new file: `examples/[role]_agent.rs`
2. Define the agent structure with domain expertise
3. Implement task-specific methods
4. Add comprehensive demo in `main()`
5. Update this README
6. Test with: `cargo run --example [role]_agent`

---

## 📝 License

These examples are part of The Agency project and follow the same MIT license.

---

## 🔗 Acknowledgments

Agent designs inspired by real job descriptions from leading AI and robotics companies working on humanoid robots and autonomous systems.
