# Model Presets for Collaborative Workspace

This document explains how to use model presets in the collaborative robotics workspace example.

## Overview

The collaborative workspace example supports multiple model presets that allow you to easily switch between different LLM configurations for your agents. This is useful for:

- **Testing different model combinations** without editing code
- **Environment-specific configurations** (development vs production)
- **Cost optimization** by using faster/cheaper models for iteration
- **Performance comparison** across different model providers

## Available Presets

The presets are defined in `examples/collaborative_workspace_config.toml`:

### 1. `specialized` (default)

**Description**: Current configuration with specialized models per role

- **SimulationEngineer**: `gpt-oss:120b-cloud` (code generation specialist)
- **ScalingEngineer**: `gpt-oss:120b-cloud` (performance & distributed systems)
- **ConfigSpecialist**: `deepseek-v3.1:671b-cloud` (URDF/XML configuration)
- **Coordinator**: `gpt-oss:120b-cloud` (integration reasoning & documentation)

### 2. `all_gpt_oss`

**Description**: All agents use gpt-oss:120b-cloud for consistent reasoning

- All agents: `gpt-oss:120b-cloud`

### 3. `all_deepseek`

**Description**: All agents use deepseek for balanced performance

- All agents: `deepseek-v3.1:671b-cloud`

### 4. `fast`

**Description**: Fast cloud models for quick iteration

- **SimulationEngineer**: `gpt-oss:20b-cloud`
- **ScalingEngineer**: `gpt-oss:20b-cloud`
- **ConfigSpecialist**: `glm-4.6:cloud`
- **Coordinator**: `glm-4.6:cloud`

## Usage

### Method 1: Environment Variable (Recommended)

Set the `MODEL_PRESET` environment variable before running the example:

```bash
# Use the specialized preset (default)
MODEL_PRESET=specialized cargo run --example collaborative_robotics_complex

# Use all GPT-OSS models
MODEL_PRESET=all_gpt_oss cargo run --example collaborative_robotics_complex

# Use all DeepSeek models
MODEL_PRESET=all_deepseek cargo run --example collaborative_robotics_complex

# Use fast models for quick iteration
MODEL_PRESET=fast cargo run --example collaborative_robotics_complex
```

### Method 2: Command Line Argument

Pass the preset name as the first argument:

```bash
# Use specialized preset
cargo run --example collaborative_robotics_complex specialized

# Use all GPT-OSS models
cargo run --example collaborative_robotics_complex all_gpt_oss

# Use all DeepSeek models
cargo run --example collaborative_robotics_complex all_deepseek

# Use fast models
cargo run --example collaborative_robotics_complex fast
```

### Method 3: Default Behavior

If no environment variable or CLI argument is provided, the example defaults to the `specialized` preset:

```bash
# Uses 'specialized' preset by default
cargo run --example collaborative_robotics_complex
```

## Adding Custom Presets

You can add your own presets by editing `examples/collaborative_workspace_config.toml`:

```toml
[model_presets.my_custom_preset]
description = "My custom model configuration"
max_tokens = 2048          # Optional: defaults to 1024
timeout = 120              # Optional: defaults to 60 seconds
simulation_engineer = "my-model:version"
scaling_engineer = "my-model:version"
config_specialist = "my-model:version"
coordinator = "my-model:version"
```

Then use it:

```bash
MODEL_PRESET=my_custom_preset cargo run --example collaborative_robotics_complex
```

### Configuration Options

Each preset supports the following options:

- **`description`** (required): Human-readable description of the preset
- **`max_tokens`** (optional): Maximum tokens for generation (default: 1024)
- **`timeout`** (optional): Request timeout in seconds (default: 60)
- **`simulation_engineer`** (required): Model for SimulationEngineer agent
- **`scaling_engineer`** (required): Model for ScalingEngineer agent
- **`config_specialist`** (required): Model for ConfigSpecialist agent
- **`coordinator`** (required): Model for Coordinator agent

## Output

When a preset is applied, you'll see output like:

```text
🎨 Applying model preset: 'all_deepseek'
   Description: All agents use deepseek for balanced performance
   From config: examples/collaborative_workspace_config.toml

🤖 Agent model assignments:
  • SimulationEngineer → deepseek-v3.1:671b-cloud
  • ScalingEngineer → deepseek-v3.1:671b-cloud
  • ConfigSpecialist → deepseek-v3.1:671b-cloud
  • Coordinator → deepseek-v3.1:671b-cloud
```

If an invalid preset name is provided, the system falls back to the `specialized` configuration and lists available presets:

```text
⚠️  Preset 'invalid_name' not found, using default 'specialized' configuration
   Available presets: ["specialized", "all_gpt_oss", "all_deepseek", "fast"]
   From config: examples/collaborative_workspace_config.toml
```

## Implementation Details

The preset system:

1. **Loads the TOML config** including all preset definitions
2. **Checks for preset selection** via environment variable or CLI argument
3. **Applies the selected preset** to all agent configurations
4. **Falls back to hardcoded defaults** if preset not found

The precedence order is:

1. `MODEL_PRESET` environment variable
2. First CLI argument
3. Default preset: `specialized`

## Benefits

- **No code changes required** - switch configurations without recompiling
- **Environment-specific** - use different presets in different deployment scenarios
- **Easy comparison** - quickly test different model combinations
- **Production-ready** - configure once, deploy anywhere with environment variables
