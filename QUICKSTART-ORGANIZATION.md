# Quick Start: Multi-Agent Organization System

## 🚀 Run the Demo

```bash
./scripts/demo-organization.sh
```

This will:
- Run the complete organization example
- Generate timestamped outputs in `demo-outputs/`
- Create comprehensive reports and logs

## 📂 Demo Outputs

After running the demo, check:
```
demo-outputs/organization-YYYYMMDD-HHMMSS/
├── organization-execution.log  # Full execution trace
├── demo-report.md             # Comprehensive markdown report
└── workspace-summary.txt      # Task distribution breakdown
```

## 🎯 What You'll See

### Organization
- **14 agents** across 11 specialized categories
- **5 collaborative workspaces**
- **8 coordinated tasks**

### Agent Categories
1. Research & AI
2. Software Engineering
3. Hardware Engineering
4. Robotics Engineering
5. Manufacturing & Production
6. Supply Chain & Quality
7. Infrastructure & IT
8. Service & Support
9. Security
10. Engineering Specializations
11. Legal & Finance

### Workspaces
1. **AI & Autonomy Research** - 3 agents
2. **Software Platform** - 3 agents
3. **Hardware Integration** - 3 agents
4. **Manufacturing Excellence** - 3 agents
5. **Supply Chain & Analytics** - 2 agents

## 💻 Run Examples Manually

### Organization Example
```bash
cargo run --example robotech_industries_organization_example
```

### Organization Daemon
```bash
cargo run --bin organization-daemon
```

## 📖 Documentation

- Full documentation: `docs/ORGANIZATION.md`
- Script documentation: `scripts/README.md`
- API reference in `docs/ORGANIZATION.md`

## 🔧 Quick Code Examples

### Create an Organization
```rust
use the_agency::{Organization, OrganizationAgent, OrganizationRole};

let mut org = Organization::new("My Company".to_string());

let agent = OrganizationAgent::new(
    "Alice".to_string(),
    OrganizationRole::SoftwareEngineerSimulation,
);
let agent_id = org.add_agent(agent);
```

### Create a Workspace
```rust
use the_agency::CollaborativeWorkspace;

let workspace = CollaborativeWorkspace::new(
    "Engineering Team".to_string(),
    "Software development workspace".to_string(),
);
let ws_id = org.create_workspace(workspace);

org.assign_agent_to_workspace(&agent_id, &ws_id)?;
```

### Coordinate Tasks
```rust
use the_agency::organization::coordinator::AgentCoordinator;
use the_agency::{WorkspaceTask, TaskPriority};

let coordinator = AgentCoordinator::new(org);

// Spawn agents
coordinator.spawn_agent(agent_id.clone(), config).await?;

// Create and assign task
let task = WorkspaceTask::new(
    "Build Feature".to_string(),
    "Implement new functionality".to_string(),
    vec![agent_id.clone()],
).with_priority(TaskPriority::High);

coordinator.assign_task(&agent_id, &ws_id, task).await?;
coordinator.process_messages().await?;
```

## 🎨 Key Features

### ✅ Multi-Agent Coordination
- Specialized roles with capabilities
- Agent status tracking
- Workspace membership

### ✅ Task Management
- Priority levels (Critical, High, Medium, Low)
- Task dependencies
- Status tracking

### ✅ Communication
- Message queue
- Agent-to-agent messaging
- Status updates

### ✅ Work Products
- Task completion logs
- Execution traces
- Performance metrics
- Workspace summaries

## 🐛 Troubleshooting

**Demo script won't run:**
```bash
chmod +x scripts/demo-organization.sh
```

**Compilation errors:**
```bash
cargo clean
cargo check
```

**Can't find outputs:**
```bash
ls -la demo-outputs/
```

## 📚 Learn More

- [Full Documentation](docs/ORGANIZATION.md)
- [Script Documentation](scripts/README.md)
- [Example Code](examples/robotech_industries_organization_example.rs)

## 🌟 Next Steps

1. Run the demo script
2. Review generated reports
3. Explore the example code
4. Build your own organization
5. Customize roles and workspaces

---

**Need help?** Check `docs/ORGANIZATION.md` for detailed API reference and examples.
