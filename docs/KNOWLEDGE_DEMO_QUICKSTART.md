# Knowledge Management Demo - Quick Start Guide

## Overview

This guide helps you run the RoboTech Industries multi-agent organization example, which demonstrates comprehensive knowledge management and organizational learning capabilities.

## Quick Start

### Run the Demo

```bash
cd /Users/ravindraboddipalli/sources/the-agency
cargo run --example robotech_industries_organization_example
```

### What You'll See

The demo simulates RoboTech Industries executing 9 concurrent projects:

```
🤖 RoboTech Industries - Multi-Agent Organization Demo
==========================================================
🎯 MISSION: Build 3 Humanoid Robot Variants

   Robo-1: Home Companion (chores, security, emotional support)
   Robo-2: Construction Assistant (Robo-1 + heavy lifting)
   Robo-3: Rescue Operations (wildfire + coastguard)
```

### Projects Executed

1. **🏠 Robo-1 Development** (Critical) - 3 tasks
2. **🏗️ Robo-2 Development** (Critical) - 3 tasks
3. **🚒 Robo-3 Development** (Critical) - 4 tasks
4. **🧠 AI Research** (High) - 2 tasks
5. **💻 Platform Development** (High) - 2 tasks
6. **⚙️ Hardware Integration** (Medium) - 2 tasks
7. **📊 Executive Strategy** (Medium) - 1 task
8. **📦 Product Strategy** (Medium) - 1 task
9. **🤝 Customer Success** (Low) - 2 tasks

**Total: 20 tasks across 25+ agents**

## Key Features Demonstrated

### 1. Knowledge Capture
Every task execution automatically:
- Creates a structured knowledge entry
- Stores it in the agent's persistent memory
- Makes it available for future task context

### 2. Context-Aware Execution
Before executing tasks, agents:
- Query their memory for similar past experiences
- Build enhanced prompts with relevant context
- Execute with improved understanding

### 3. Multi-Project Coordination
The organization handles:
- Multiple concurrent projects
- Different priority levels (Critical, High, Medium, Low)
- Cross-functional team collaboration
- Complex dependencies and workflows

### 4. Organizational Structure
- **25+ specialized agents** (engineers, researchers, executives, product managers)
- **8 collaborative workspaces** (robot variants, manufacturing, executive, product, customer success)
- **Multiple departments** (R&D, Engineering, Manufacturing, Executive, Product, Sales)

## Understanding the Output

### Agent Spawning
```
🚀 Spawning AI agents...
  ✓ Spawned: Alice Chen (ResearchEngineerScaling) with learning capabilities
  ✓ Spawned: Bob Martinez (ResearchEngineerAutonomy) with learning capabilities
  ...
```

### Project Execution
```
🏠 Project 1: Robo-1 Home Companion Development
   ✅ Completed 3 Robo-1 development tasks
```

### Final Summary
```
📊 Project Summary:
   🏠 Robo-1 Development: 3 tasks (Critical)
   🏗️ Robo-2 Development: 3 tasks (Critical)
   ...
   📈 Total Tasks Executed: 20
   🧠 Knowledge Captured: 20 experiences stored in agent memories
```

## Code Structure

### Example Organization

```rust
// 1. Create organization with agents
let mut org = create_organization().await?;

// 2. Setup workspaces
setup_workspaces(&mut org).await?;

// 3. Initialize coordinator (with knowledge management)
let coordinator = AgentCoordinator::new(org.clone());

// 4. Spawn agents with memory enabled
spawn_agents(&coordinator, &org).await?;

// 5. Execute projects (knowledge is captured automatically)
execute_projects(&coordinator, &org).await?;
```

### Knowledge Management Flow

```rust
// In execute_task():
// 1. Query past experiences
let memories = agent.list_memories(&task.description, 5).await?;

// 2. Build enhanced prompt
let enhanced_prompt = build_knowledge_enhanced_prompt(
    &task.title,
    &task.description,
    memories
)?;

// 3. Execute with context
let result = agent.process(enhanced_prompt).await?;

// In handle_task_completion():
// 4. Create knowledge entry
let knowledge = build_knowledge_entry(
    &task.title,
    &task.description,
    &agent_role,
    &approach,
    &outcome
)?;

// 5. Store in agent memory
agent.store_knowledge(knowledge).await?;
```

## Customization

### Modify Projects

Edit `execute_projects()` in the example file to:
- Add/remove projects
- Change task priorities
- Assign different agents
- Modify task descriptions

### Adjust Organization

Edit `create_organization()` to:
- Add more agents
- Change agent roles
- Create different team structures

### Configure Workspaces

Edit `setup_workspaces()` to:
- Add new collaborative spaces
- Change team assignments
- Modify workspace purposes

## Troubleshooting

### Build Issues

```bash
# Clean build
cargo clean
cargo build --example robotech_industries_organization_example

# Check for errors
cargo check --example robotech_industries_organization_example
```

### Memory Issues

If the example uses too much memory:
- Reduce the number of concurrent projects
- Decrease task count per project
- Limit memory query results (change `limit` parameter)

### Slow Execution

The example makes real LLM calls, so execution time depends on:
- LLM provider response time
- Number of concurrent tasks
- Network latency

To speed up:
- Reduce task count
- Use faster LLM models
- Enable parallel execution (already done in coordinator)

## Next Steps

### Experiment with Knowledge

1. **Run Multiple Times**
   - First run: No prior knowledge
   - Second run: Agents use knowledge from first run
   - Compare performance improvement

2. **Analyze Memories**
   - Check agent memory stores
   - Review knowledge entries
   - Track learning patterns

3. **Extend Functionality**
   - Add cross-agent knowledge sharing
   - Implement knowledge analytics
   - Create organizational knowledge base

### Modify the Example

1. **Change Mission**
   - Replace robot variants with your domain
   - Adjust agent roles accordingly
   - Modify task definitions

2. **Add Complexity**
   - Introduce task dependencies
   - Add failure scenarios
   - Implement retry logic

3. **Enhance Knowledge**
   - Add success metrics
   - Track improvement over time
   - Implement best practices library

## Resources

- **Full Documentation**: See `KNOWLEDGE_MANAGEMENT_SUMMARY.md`
- **Source Code**: `examples/robotech_industries_organization_example.rs`
- **Core Implementation**: `src/organization/knowledge.rs`
- **Coordinator Logic**: `src/organization/coordinator.rs`

## Support

For questions or issues:
1. Check the knowledge management summary documentation
2. Review inline code comments
3. Examine the coordinator implementation
4. Analyze the example code structure

## Performance Notes

- **Memory Usage**: Moderate (depends on memory store configuration)
- **Execution Time**: ~1-5 minutes (depends on LLM provider)
- **Concurrency**: Tasks execute in parallel within workspaces
- **Scalability**: Tested with 25+ agents and 20 tasks

## Success Indicators

✅ All agents spawn successfully  
✅ All 9 projects execute without errors  
✅ 20 tasks complete across organization  
✅ Knowledge entries stored in agent memories  
✅ Final summary shows completed work  

## Conclusion

This demo showcases a production-ready knowledge management system integrated into a realistic multi-agent organization. The RoboTech Industries example demonstrates complex coordination, persistent learning, and organizational memory in action.

**Happy exploring! 🚀**
