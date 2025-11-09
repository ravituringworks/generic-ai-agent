# Examples Documentation

This directory contains documentation for all the-agency examples.

## Documentation Files

### RoboTech Industries Organization

- **[ROBOTECH-ORGANIZATION.md](ROBOTECH-ORGANIZATION.md)** - Overview of the RoboTech Industries multi-agent organization example
- **[ROBOTECH_ARTIFACTS.md](ROBOTECH_ARTIFACTS.md)** - Generated artifacts from the organization (code, configs, design docs)

### Collaborative Robotics Workspace

- **[COLLABORATIVE_WORKSPACE_SUCCESS.md](COLLABORATIVE_WORKSPACE_SUCCESS.md)** - Success story and results from basic workspace
- **[COLLABORATIVE_RESULTS.md](COLLABORATIVE_RESULTS.md)** - Detailed results analysis from complex workspace
- **[COLLABORATIVE_COMPARISON.md](COLLABORATIVE_COMPARISON.md)** - Comparison between workspace versions
- **[WORKSPACE_COMPARISON.md](WORKSPACE_COMPARISON.md)** - Feature comparison across all workspace examples
- **[COMPLEX_WORKSPACE.md](COMPLEX_WORKSPACE.md)** - Complex workspace implementation details

### Configuration & Setup

- **[MODEL_PRESETS.md](MODEL_PRESETS.md)** - LLM model presets and configurations
- **[MULTI_MODEL_CONFIG.md](MULTI_MODEL_CONFIG.md)** - Multi-model configuration guide
- **[PRESET_VERIFICATION.md](PRESET_VERIFICATION.md)** - Model preset verification results

### Specialized Topics

- **[ROBOTICS_AGENTS.md](ROBOTICS_AGENTS.md)** - Robotics-specific agent roles and capabilities
- **[SCALING_ENGINEER_TASKS.md](SCALING_ENGINEER_TASKS.md)** - Scaling engineer role and tasks

## Output Directory Structure

All examples now output to a unified `output/` directory at the project root:

```text
output/
├── robotech_organization_output/        # RoboTech Industries demo
│   ├── reports/
│   ├── artifacts/
│   └── logs/
├── robotics_workspace/                   # Basic collaborative robotics
├── robotics_workspace_enhanced/          # Enhanced version
└── robotics_workspace_complex/           # Complex version
```

## Running Examples

```bash
# RoboTech Industries multi-agent organization
cargo run --example robotech_industries_organization_example

# Basic collaborative workspace
cargo run --example collaborative_robotics_workspace

# Enhanced workspace
cargo run --example collaborative_robotics_enhanced

# Complex workspace with phases
cargo run --example collaborative_robotics_complex
```

## Viewing Results

After running examples, outputs are in the `output/` directory:

```bash
# View RoboTech artifacts
ls -R output/robotech_organization_output/artifacts/

# View workspace results
ls -R output/robotics_workspace_complex/humanoid_manipulation_system/
```

---

For more information, see the main [repository README](../../README.md).
