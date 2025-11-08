# The Agency Examples

This directory contains comprehensive examples demonstrating various features and capabilities of The Agency framework. Examples are organized by category and complexity level.

## Quick Start Examples

### Basic Usage
- **`minimal_org_test.rs`** - Minimal multi-agent organization test
- **`simple_coordinator_test.rs`** - Simple agent coordination example

### LLM Integration
- **`multi_provider_example.rs`** - Multi-provider LLM usage with fallback
- **`multi_provider_usage.rs`** - Advanced multi-provider configuration
- **`test_ollama_connection.rs`** - Testing Ollama connectivity

## Agent Examples

### Specialized Agents
- **`robotics_research_engineer_agent.rs`** - Research engineer for robotics
- **`robotics_research_engineer_example.rs`** - Complete robotics research workflow
- **`robotics_scientist_agent.rs`** - Scientific research and analysis agent
- **`simulation_engineer_agent.rs`** - Engineering simulation and modeling
- **`scaling_engineer_agent.rs`** - System scaling and optimization

### Agent Communication
- **`a2a_communication.rs`** - Agent-to-Agent communication protocols
- **`agent_network_system.rs`** - Complete agent network system

## Workflow Examples

### Saga Patterns
- **`saga_workflow.rs`** - Basic saga workflow with compensation
- **`saga_llm_workflow.rs`** - Saga workflow with LLM integration

### Workflow Integration
- **`workflow_agent_tool_integration.rs`** - Agent and tool integration
- **`workflow_integration_simple.rs`** - Simple workflow integration
- **`control_flow_example.rs`** - Control flow patterns
- **`control_flow_simple.rs`** - Basic control flow

## Knowledge & Memory

### RAG Systems
- **`rag_system_comprehensive.rs`** - Comprehensive RAG implementation
- **`rag_system_working.rs`** - Working RAG system example
- **`pdf_rag_with_tables.rs`** - PDF processing with table extraction

### Knowledge Management
- **`knowledge_rag.rs`** - Knowledge-based RAG (if exists)
- **`unified_storage_system.rs`** - Unified storage system

## Multi-Agent Organization

### Collaborative Workspaces
- **`collaborative_robotics_complex.rs`** - Complex collaborative robotics
- **`collaborative_robotics_enhanced.rs`** - Enhanced collaborative features
- **`collaborative_robotics_workspace.rs`** - Robotics workspace collaboration
- **`collaborative_workspace_config.toml`** - Workspace configuration

### Organization Examples
- **`robotech_industries_organization_example.rs`** - Complete organization example
- **`humanoid_robot_project.rs`** - Humanoid robot project organization

## Tools & Integration

### MCP Integration
- **`datetime_location_tools.rs`** - Date/time and location tools
- **`mcp_integration.feature`** - MCP integration features (BDD)

### API & Services
- **`daemon_api_example.rs`** - Daemon API usage
- **`api.rs`** - API integration examples

## Testing & Configuration

### Test Examples
- **`test_config.rs`** - Configuration testing
- **`unit_tests.rs`** - Unit testing examples
- **`bdd_steps.rs`** - BDD testing steps

### Documentation Examples
Located in `examples/docs/`:
- **COLLABORATIVE_COMPARISON.md** - Collaborative workspace comparisons
- **COLLABORATIVE_RESULTS.md** - Results from collaborative experiments
- **MODEL_PRESETS.md** - LLM model preset configurations
- **ROBOTICS_AGENTS.md** - Robotics agent documentation
- **WORKSPACE_COMPARISON.md** - Workspace performance comparisons

## Running Examples

### Prerequisites

1. **Install Rust** (1.75+):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install and run Ollama**:
   ```bash
   # Install Ollama
   curl -fsSL https://ollama.ai/install.sh | sh

   # Pull required models
   ollama pull llama3.2
   ollama pull nomic-embed-text
   ```

3. **Build the project**:
   ```bash
   cargo build
   ```

### Running Individual Examples

```bash
# Basic multi-provider LLM example
cargo run --example multi_provider_example

# Saga workflow with LLM integration
cargo run --example saga_llm_workflow

# Robotics research engineer
cargo run --example robotics_research_engineer_example

# Agent-to-Agent communication
cargo run --example a2a_communication

# PDF RAG with table extraction
cargo run --example pdf_rag_with_tables

# Unified storage system
cargo run --example unified_storage_system
```

### Running with Custom Configuration

```bash
# Use custom config file
cargo run --example multi_provider_example -- --config path/to/config.toml

# Enable debug logging
RUST_LOG=debug cargo run --example saga_llm_workflow
```

## Example Categories

### Beginner Examples
- `minimal_org_test.rs` - Start here for basic concepts
- `simple_coordinator_test.rs` - Simple agent coordination
- `multi_provider_example.rs` - Basic LLM provider usage

### Intermediate Examples
- `saga_workflow.rs` - Workflow patterns
- `a2a_communication.rs` - Multi-agent communication
- `rag_system_comprehensive.rs` - Knowledge retrieval

### Advanced Examples
- `collaborative_robotics_workspace.rs` - Complex multi-agent systems
- `robotech_industries_organization_example.rs` - Large-scale organization
- `pdf_rag_with_tables.rs` - Advanced document processing

## Feature Tags

Examples are tagged with features they demonstrate:

- **LLM**: Language model integration
- **A2A**: Agent-to-Agent communication
- **Memory**: Vector storage and retrieval
- **Tools**: MCP tool integration
- **Workflow**: Workflow orchestration
- **Saga**: Distributed transaction patterns
- **RAG**: Retrieval-augmented generation
- **Organization**: Multi-agent organization
- **PDF**: Document processing
- **Storage**: Data persistence

## Contributing Examples

When adding new examples:

1. **Categorize properly** - Place in appropriate category directory
2. **Add comprehensive comments** - Explain what each section does
3. **Include error handling** - Show proper error handling patterns
4. **Update this README** - Add your example to the appropriate section
5. **Test thoroughly** - Ensure examples run with default configuration

## Support

- [Main Documentation](../README.md)
- [Report Issues](https://github.com/ravituringworks/the-agency/issues)
- [Community Discussions](https://github.com/ravituringworks/the-agency/discussions)</content>
</xai:function_call">\
<xai:function_call name="read">
<parameter name="filePath">examples/docs/README.md