# Multi-Agent Organization System

A comprehensive framework for managing multi-agent organizations with collaborative workspaces, designed for complex engineering teams in robotics and advanced technology companies.

## Overview

The Organization system provides:

- **Organizational Structure**: 60+ specialized roles across 11 categories
- **Collaborative Workspaces**: Shared environments where agents work together
- **Agent Coordination**: Inter-agent communication and task delegation
- **Task Management**: Priority-based task assignment with dependency tracking
- **Organization Daemon**: Background service managing the agent environment

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Organization Daemon                        │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Agent Coordinator                         │  │
│  │  - Message Queue                                       │  │
│  │  - Task Routing                                        │  │
│  │  - Workspace Orchestration                             │  │
│  └───────────────────────────────────────────────────────┘  │
│                          │                                   │
│      ┌───────────────────┼───────────────────┐             │
│      │                   │                   │             │
│  ┌───▼────┐      ┌───────▼───────┐  ┌──────▼──────┐      │
│  │ Agents │      │  Workspaces   │  │    Tasks    │      │
│  │        │◄────►│               │◄─┤             │      │
│  └────────┘      └───────────────┘  └─────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Organization Roles

### 1. Research & AI (7 roles)
- Research Engineer, Scaling
- Research Engineer, Autonomy
- Research Engineer, World Models
- Research Engineer, Reinforcement Learning
- Research Engineer, Data Infrastructure
- Research Engineer, Robot Character
- AI Resident

### 2. Software Engineering (10 roles)
- Software Engineer, Simulation
- Software Engineer, Teleoperation
- Software Engineer, Platforms
- Software Engineer, Operating System
- Software Engineer, DevOps
- Software Engineer, Embedded Systems
- Software Engineer, Systems
- Software Engineer, Frontend
- Software Engineer, Cloud & Infrastructure
- Software Engineer, ERP Systems

### 3. Security (4 roles)
- Product Security Engineer, Operating System
- Product Security Engineer, Cloud & Infrastructure
- Product Security Engineer, Cryptography & PKI
- Network Security Engineer

### 4. Hardware Engineering (7 roles)
- Electrical Engineer (Entry Level, Battery & Charger, Hardcore, Technical Lead)
- EMI/EMC Engineer
- Mechanical Engineer, All Levels
- R&D Engineer, Humanoid Core Technologies

### 5. Robotics Engineering (4 roles)
- Senior Robotics Engineer, Controls
- Senior Robotics Engineer, Software
- Robotics Engineer, Controls & Testing
- Senior Audio Systems Engineer

### 6. Manufacturing & Production (10 roles)
- Manufacturing Engineer
- Automation Engineer, Manufacturing
- Test Engineer, Manufacturing
- Build Quality Engineer (Electrical, Mechanical)
- Production Lead
- Senior Manager, Production
- Assembly Technician
- CNC Operator
- CNC Programmer

### 7. Supply Chain & Quality (10 roles)
- Global Supply Manager (Structures, Motors & Magnets)
- Supplier Development Engineer (Structures, Motors & Magnets, EEE)
- NPI Planner
- NPI Project Manager
- Quality Inspection Specialist
- Quality Engineer, Manufacturing
- Data Analyst

### 8. Infrastructure & IT (2 roles)
- Enterprise Engineer
- Principal Enterprise IT Engineer

### 9. Service & Support (3 roles)
- Sr. Service Training Engineer
- Sr. Robot Service Technician
- Robot Operator

### 10. Engineering Specializations (6 roles)
- Softgoods Engineer, Prototyping
- Wiring & Harnessing Engineer
- Electrical Engineering Internship
- Mechanical Engineering Internship
- Test Engineer, R&D
- Head of Physical Robot Safety

### 11. Legal & Finance (3 roles)
- Counsel, Employment & Compensation
- Counsel, Commercial, Trade & Compliance
- Payroll Accountant

## Usage

### Creating an Organization

```rust
use the_agency::{Organization, OrganizationAgent, OrganizationRole};

let mut org = Organization::new("RoboTech Industries".to_string());

// Add agents
let alice = OrganizationAgent::new(
    "Alice".to_string(),
    OrganizationRole::ResearchEngineerScaling,
);
let alice_id = org.add_agent(alice);
```

### Creating Workspaces

```rust
use the_agency::CollaborativeWorkspace;

let mut workspace = CollaborativeWorkspace::new(
    "AI Research".to_string(),
    "Develop next-gen AI models".to_string(),
);
let ws_id = org.create_workspace(workspace);

// Assign agents to workspace
org.assign_agent_to_workspace(&alice_id, &ws_id)?;
```

### Task Management

```rust
use the_agency::{WorkspaceTask, TaskPriority};

let task = WorkspaceTask::new(
    "Design World Model".to_string(),
    "Create architecture for world models".to_string(),
    vec![alice_id.clone()],
)
.with_priority(TaskPriority::Critical)
.with_dependencies(vec![other_task_id]);
```

### Agent Coordination

```rust
use the_agency::organization::coordinator::AgentCoordinator;

// Create coordinator
let coordinator = AgentCoordinator::new(org);

// Spawn AI agents
coordinator.spawn_agent(alice_id.clone(), agent_config).await?;

// Assign and execute tasks
coordinator.assign_task(&alice_id, &ws_id, task).await?;
coordinator.process_messages().await?;

// Coordinate workspace project
let results = coordinator
    .coordinate_workspace_project(&ws_id, tasks)
    .await?;
```

### Running the Organization Daemon

```rust
use the_agency::organization::OrganizationDaemon;

// Create daemon
let daemon = OrganizationDaemon::new(org);
daemon.start().await?;

// Spawn agents and assign tasks
// ... (setup code)

// Run event loop
tokio::spawn(async move {
    daemon.run().await
});

// Stop when done
daemon.stop().await?;
```

## Examples

### Basic Organization Example

Run the organization example:

```bash
cargo run --example robotech_industries_organization_example
```

This demonstrates:
- 14 agents across multiple roles
- 5 collaborative workspaces
- 8 coordinated tasks
- Cross-workspace dependencies

### Organization Daemon

Run the daemon:

```bash
cargo run --bin organization-daemon
```

This shows:
- Real-time agent coordination
- Message queue processing
- Multi-workspace task orchestration
- Organization state management

## Key Features

### 1. Role-Based Agent System

Each agent has a specific organizational role with:
- Predefined capabilities
- Category classification
- Typical collaborators

```rust
let role = OrganizationRole::SoftwareEngineerSimulation;
let capabilities = role.capabilities();  // ["python", "robotics_simulation", ...]
let category = role.category();          // RoleCategory::SoftwareEngineering
let collaborators = role.typical_collaborators();  // [ResearchEngineerAutonomy, ...]
```

### 2. Collaborative Workspaces

Workspaces provide:
- Shared context between agents
- Task organization
- Artifact storage
- Member management

### 3. Task Dependencies

Tasks support:
- Priority levels (Low, Medium, High, Critical)
- Dependencies on other tasks
- Status tracking (Pending, InProgress, Blocked, UnderReview, Completed)
- Timestamps for creation and completion

### 4. Agent Coordination

The coordinator handles:
- Message routing between agents
- Task assignment and execution
- Status updates
- Cross-workspace orchestration

### 5. Intelligent Task Routing

Tasks are routed based on:
- Agent role and capabilities
- Agent availability
- Workspace membership
- Task requirements

## API Reference

### Organization

```rust
pub struct Organization {
    pub name: String,
    pub agents: HashMap<String, OrganizationAgent>,
    pub workspaces: HashMap<String, CollaborativeWorkspace>,
}

impl Organization {
    pub fn new(name: String) -> Self;
    pub fn add_agent(&mut self, agent: OrganizationAgent) -> String;
    pub fn create_workspace(&mut self, workspace: CollaborativeWorkspace) -> String;
    pub fn assign_agent_to_workspace(&mut self, agent_id: &str, workspace_id: &str) -> Result<()>;
    pub fn get_available_agents(&self, role: Option<OrganizationRole>) -> Vec<&OrganizationAgent>;
    pub fn get_workspace_agents(&self, workspace_id: &str) -> Vec<&OrganizationAgent>;
}
```

### AgentCoordinator

```rust
pub struct AgentCoordinator { /* ... */ }

impl AgentCoordinator {
    pub fn new(organization: Organization) -> Self;
    pub async fn spawn_agent(&self, agent_id: String, config: AgentConfig) -> Result<()>;
    pub async fn assign_task(&self, agent_id: &str, workspace_id: &str, task: WorkspaceTask) -> Result<()>;
    pub async fn execute_task(&self, agent_id: &str, task: &WorkspaceTask) -> Result<TaskResult>;
    pub async fn process_messages(&self) -> Result<()>;
    pub async fn coordinate_workspace_project(&self, workspace_id: &str, tasks: Vec<WorkspaceTask>) -> Result<Vec<TaskResult>>;
    pub async fn get_organization(&self) -> Organization;
}
```

### OrganizationDaemon

```rust
pub struct OrganizationDaemon { /* ... */ }

impl OrganizationDaemon {
    pub fn new(organization: Organization) -> Self;
    pub async fn start(&self) -> Result<()>;
    pub async fn stop(&self) -> Result<()>;
    pub async fn is_running(&self) -> bool;
    pub async fn run(&self) -> Result<()>;
    pub fn coordinator(&self) -> &AgentCoordinator;
}
```

## Future Enhancements

Potential improvements:
- [ ] Persistent organization state (database storage)
- [ ] Advanced task scheduling algorithms
- [ ] Load balancing across agents
- [ ] Real-time monitoring dashboard
- [ ] Agent performance metrics
- [ ] Workspace templates
- [ ] Role-based access control
- [ ] Integration with external project management tools
- [ ] Agent learning from task outcomes
- [ ] Dynamic role assignment

## Integration with Existing Systems

The organization system integrates with:

- **A2A Communication**: Agents can communicate using the existing A2A protocol
- **MCP Tools**: Agents use MCP for tool calling during task execution
- **Memory System**: Shared knowledge base across workspaces
- **Workflow Engine**: Complex workflows within tasks
- **SAGA Patterns**: Distributed transactions across agents

## License

Part of the-agency project.
