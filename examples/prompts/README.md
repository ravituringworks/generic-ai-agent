# Prompt Configuration System

## Overview

The Agency's prompt configuration system allows you to define task prompts in external configuration files (TOML) instead of hardcoding them in Rust code. This enables:

- **Easy updates**: Modify prompts without recompiling code
- **Version control**: Track prompt changes separately from code
- **Collaboration**: Non-developers can update task requirements
- **Agent-driven generation**: Prompts specify requirements; agents generate actual implementations
- **Template support**: Use variables for dynamic prompt generation

## Philosophy: Requirements, Not Code

**Key Principle**: Prompts should define WHAT needs to be built, not HOW to build it.

### ❌ Old Approach (Hardcoded, Prescriptive)
```rust
let task = WorkspaceTask::new(
    "Create BOM".to_string(),
    "Create a BOM with columns: Part, Qty, Cost. \
    Use this Python code: df = pd.DataFrame(...) \
    Save to bom.csv".to_string(),
    vec![agent_id]
);
```

**Problem**: Locks agents into specific implementations. No room for creativity or optimization.

### ✅ New Approach (Config-based, Requirements-driven)
```toml
[hardware.compute_platform_analysis]
title = "Generate Compute Platform Trade-off Analysis"
priority = "Critical"
requirements = """
Create comprehensive compute platform comparison for 2025.

REQUIREMENTS:
1. Platform Options Analysis
   - Flagship: NVIDIA Jetson AGX Thor (2000 TOPS, $2499)
   - Premium: NVIDIA Jetson AGX Orin 64GB (275 TOPS, $1999)
   - [full specifications...]

7. Deliverables (agents MUST generate):
   - Comparison matrix (spreadsheet format)
   - Performance/watt charts
   - BOM with supplier part numbers
   - Actual test code and analysis scripts

NOTE: Agents should generate actual code, NOT pseudocode.
"""
```

**Benefits**:
- Agents choose best tools/languages
- Implementations evolve with technology
- Prompts remain stable even as code practices change
- Agents can be more creative and optimal

## Directory Structure

```
examples/
├── prompts/
│   ├── README.md                 # This file
│   ├── robotech_prompts.toml     # RoboTech Industries tasks
│   ├── humanoid_prompts.toml     # (future) Humanoid robot tasks
│   └── simulation_prompts.toml   # (future) Simulation tasks
└── robotech_industries_organization_example.rs
```

## Configuration Format

### Basic Structure

```toml
[meta]
version = "1.0.0"
description = "Project description"
last_updated = "2025-01-05"

[category.task_name]
title = "Human-readable task title"
priority = "Critical" | "High" | "Medium" | "Low"
assigned_roles = ["Role1", "Role2"]
requirements = """
Multi-line requirements specification.

REQUIREMENTS:
1. What needs to be built
2. Success criteria
3. Deliverables (be specific!)
4. Example output formats

NOTE: Guidelines for agents
"""
```

### Template Variables

Prompts support variable interpolation for dynamic content:

```toml
[templates]
available_variables = [
    "project_name",
    "budget",
    "timeline",
    "hardware_tier",  # flagship, premium, standard, budget
    "agent_names",
    "software_stack"
]
```

Use in prompts:
```toml
requirements = """
Project: {project_name}
Budget: {budget}
Timeline: {timeline}
Target hardware: {hardware_tier}
"""
```

## Writing Effective Prompts

### DO: Specify Requirements and Deliverables

```toml
requirements = """
REQUIREMENTS:
1. Analyze 4 compute platforms with 2025 pricing
2. Create performance benchmarks (TOPS, FPS, latency)
3. Generate TCO model for volumes: 1, 100, 1000, 10K

DELIVERABLES (agents must generate):
- Comparison spreadsheet with actual formulas
- Python scripts for benchmark automation
- Cost calculator (interactive)
- Decision tree flowchart (Mermaid or Graphviz)
- BOM with Mouser/DigiKey part numbers

Example output structure:
|Platform    |TOPS |Power|Cost  |
|------------|-----|-----|------|
|Thor        |2000 |100W |$2499 |
"""
```

### DON'T: Prescribe Implementation Details

```toml
requirements = """
Use pandas to create DataFrame with columns...
Import matplotlib and plot...
Save to file using this code: df.to_csv(...)
"""
```

**Why?**: Agents should choose the best tools. Today it might be pandas, tomorrow it could be Polars or DuckDB.

### DO: Provide Context and Examples

```toml
requirements = """
Create microcontroller selection matrix for 2025.

CONTEXT:
- Target: Humanoid robot with 25-30 DOF
- Motor control: 1-2 kHz update rate required
- Budget constraints: $50-100 for all MCUs

MCU OPTIONS:
- High-perf: STM32H7 (550MHz, $15-25)
- Mid-range: STM32U5 (ultra-low power, $8-18)
- Budget: RP2350 (dual-core, $0.80-1.50)

EXAMPLE CONFIGURATION:
Flagship: 4× STM32H7 ($80) + 2× RP2350 ($3) = $83 total

DELIVERABLES:
- Selection scoring rubric (generate actual spreadsheet)
- Pinout compatibility matrix
- Lead time tracking dashboard
"""
```

### DON'T: Include Example Code Snippets

```toml
requirements = """
Here's an example in Python:

def calculate_cost():
    return sum(parts)

Use this template for your implementation.
"""
```

**Why?**: Limits agent creativity. They might find better algorithms or use different languages.

## Agent Behavior Guidelines

The config includes guidelines for how agents should approach different types of tasks:

### Code Generation
```toml
[agent_guidelines]
code_generation = """
When generating code, agents must:
1. Create complete, executable implementations
2. Include proper error handling and logging
3. Add comprehensive unit tests
4. Provide usage examples
5. Follow language best practices
6. Include dependency management (requirements.txt, Cargo.toml)
7. Generate CI/CD scripts
"""
```

### Analysis Generation
```toml
analysis_generation = """
When generating analysis, agents must:
1. Use data-driven insights with actual metrics
2. Create visualizations (charts, graphs)
3. Provide actionable recommendations
4. Reference specific products with part numbers
5. Generate comparison matrices
6. Create cost models and ROI calculators
"""
```

### Documentation Generation
```toml
documentation_generation = """
When generating documentation, agents must:
1. Create step-by-step procedures
2. Include diagrams and screenshots
3. Provide troubleshooting guides
4. Add quick reference guides
5. Generate FAQ sections
"""
```

## Loading Prompts in Code

### Rust Example (Future Implementation)

```rust
use the_agency::prompts::PromptLoader;

// Load prompts from config
let loader = PromptLoader::from_file("examples/prompts/robotech_prompts.toml")?;

// Get a specific task prompt
let task_prompt = loader.get_prompt("hardware.compute_platform_analysis")?;

// Interpolate variables
let context = HashMap::from([
    ("project_name", "RoboTech Humanoid"),
    ("budget", "$100,000"),
    ("hardware_tier", "flagship"),
]);
let prompt = task_prompt.render(&context)?;

// Create task from prompt
let task = WorkspaceTask::new(
    task_prompt.title.clone(),
    prompt,
    task_prompt.assigned_roles.clone(),
).with_priority(task_prompt.priority);
```

### Python Example (for API usage)

```python
import toml
from pathlib import Path

# Load prompts
prompts = toml.load("examples/prompts/robotech_prompts.toml")

# Get specific prompt
hw_analysis = prompts["hardware"]["compute_platform_analysis"]

# Create task
task = {
    "title": hw_analysis["title"],
    "requirements": hw_analysis["requirements"],
    "priority": hw_analysis["priority"],
    "assigned_roles": hw_analysis["assigned_roles"]
}

# Send to agent
response = agent_client.process(task)
```

## 2025 Hardware Examples

The `robotech_prompts.toml` includes comprehensive 2025 hardware specifications:

### Compute Platforms
- **Flagship**: NVIDIA Jetson AGX Thor (2000 TOPS, $2499)
- **Premium**: NVIDIA Jetson AGX Orin 64GB (275 TOPS, $1999)
- **Standard**: NVIDIA Jetson Orin NX 16GB (100 TOPS, $699)
- **Budget**: Raspberry Pi 5 + AI Accelerator ($153-392)

### Microcontrollers
- **High-performance**: STM32H7 (550MHz), NXP i.MX RT1180 (1.2GHz dual-core)
- **Mid-range**: STM32U5 (ultra-low power), NXP MCXN947
- **Budget**: RP2350 (dual Cortex-M33 + RISC-V), ESP32-C6 (RISC-V, WiFi 6)

### Software Stack
- JetPack 7.x (for Thor), JetPack 6.x (for Orin)
- ROS2 Jazzy/Rolling
- TensorRT 10+, CUDA 13+
- STM32CubeIDE 1.15+

## Versioning and Updates

### Semantic Versioning

```toml
[meta]
version = "1.2.0"  # major.minor.patch
```

- **Major (1.x.x)**: Breaking changes to prompt structure
- **Minor (x.2.x)**: New prompts added, non-breaking updates
- **Patch (x.x.0)**: Typo fixes, clarifications

### Changelog

Keep a changelog in the TOML file:

```toml
[changelog]
"1.2.0" = "Added Jetson AGX Thor support, updated 2025 hardware specs"
"1.1.0" = "Added supply chain qualification prompts"
"1.0.0" = "Initial release"
```

## Best Practices

### 1. Be Specific About Deliverables
❌ "Create a report about the platforms"
✅ "Generate: (1) comparison spreadsheet with formulas, (2) Python benchmark scripts, (3) cost calculator, (4) decision tree diagram"

### 2. Include Real Data
❌ "Analyze various platforms"
✅ "Analyze: Thor ($2499, 2000 TOPS), Orin ($1999, 275 TOPS), include DigiKey part numbers"

### 3. Specify Output Formats
❌ "Make a comparison"
✅ "Create comparison matrix in CSV format with columns: Platform, TOPS, Power, Cost, Lead_Time"

### 4. Set Quality Standards
❌ "Write tests"
✅ "Generate unit tests with >80% coverage, include integration tests, add CI/CD workflow"

### 5. Provide Context
❌ "Select MCUs"
✅ "Select MCUs for 25-DOF humanoid robot, 1 kHz motor control, $50-100 budget, 2025 availability"

## Migration Guide

### From Hardcoded Prompts to Config

**Step 1**: Extract prompt text
```rust
// Before
let task = WorkspaceTask::new(
    "My Task".to_string(),
    "Long prompt text here...".to_string(),
    vec![agent_id]
);
```

**Step 2**: Create TOML entry
```toml
[category.my_task]
title = "My Task"
priority = "High"
assigned_roles = ["RoleName"]
requirements = """
Long prompt text here...
"""
```

**Step 3**: Load from config (future)
```rust
let task_prompt = loader.get_prompt("category.my_task")?;
let task = WorkspaceTask::from_prompt(&task_prompt)?;
```

## Examples

See `robotech_prompts.toml` for production-ready examples of:

- Hardware platform analysis (Phase 20)
- Architecture buildout plans (Phase 21)
- Supply chain management
- Configuration management
- Migration guides

Each example demonstrates:
- Clear requirements specification
- Specific deliverables with formats
- Real 2025 hardware data
- Agent behavior expectations
- Example output structures

## Contributing

When adding new prompts:

1. Follow the established structure
2. Be specific about requirements
3. List concrete deliverables
4. Include example output formats
5. Update the version number
6. Add changelog entry
7. Test with actual agents

## Support

For questions or issues:
- Check examples in `robotech_prompts.toml`
- Review agent behavior guidelines
- Consult the main Agency documentation

---

**Remember**: Prompts are specifications, not implementations. Let agents be creative!
