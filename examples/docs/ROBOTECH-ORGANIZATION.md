# RoboTech Industries - Multi-Agent Organization Example

## Mission: Build 3 Humanoid Robot Variants

This example demonstrates a comprehensive multi-agent organization working to develop three humanoid robot variants for different use cases.

### The Robot Variants

#### 🏠 Robo-1: Home Companion

**Target Market:** Residential households
**Capabilities:**

- **Household Chores:** Cleaning, organizing, basic maintenance
- **Home Security:** Monitoring, alert systems, secure premises
- **Emotional Companionship:** Social interaction, emotional support, entertainment
**Key Features:** Safe, quiet operation | Human-friendly design | Privacy-focused

#### 🏗️ Robo-2: Construction Assistant

**Target Market:** Construction industry
**Capabilities:**

- **All Robo-1 Features** (base platform)
- **Heavy Lifting:** 50+ kg load handling
- **Construction Site Operations:** Material transport, tool handling
- **Site Safety:** Hazard detection, safety protocol enforcement
**Key Features:** Heavy-duty actuators | Ruggedized design | Site-hardened

#### 🚒 Robo-3: Rescue Operations

**Target Market:** Emergency services (wildfire, coastguard)
**Capabilities:**

- **All Robo-1 & Robo-2 Features** (advanced platform)
- **Extreme Environments:** High-heat resistance (wildfire), water-resistant (coastguard)
- **Victim Detection:** Thermal imaging, AI-powered search
- **Emergency Response:** Fire suppression, water rescue, medical assistance
**Key Features:** Military-grade systems | Advanced AI perception | Fail-safe design

---

## Organization Structure

### 25 Specialized Agents Across 8 Workspaces

#### Development Workspaces (Robot Variants)

1. **Robo-1: Home Companion** - 5 agents
2. **Robo-2: Construction Assistant** - 5 agents  
3. **Robo-3: Rescue Operations** - 6 agents (most complex)

#### Support Workspaces

1. **Manufacturing Excellence** - 3 agents
2. **Supply Chain & Analytics** - 2 agents
3. **Executive Leadership** - 4 agents
4. **Product Strategy** - 3 agents
5. **Customer & Market Success** - 4 agents

### Key Roles

**Research & AI:**

- EMP001 (Scaling)
- EMP002 (Autonomy)
- EMP003 (World Models)

**Software Engineering:**

- EMP004 (Simulation)
- EMP005 (Platforms)
- EMP006 (Embedded Systems)

**Hardware:**

- EMP007 (Electrical)
- EMP008 (Robotics Controls)
- EMP009 (Mechanical)

**Manufacturing:**

- EMP010 (Manufacturing Engineer)
- EMP011 (Automation)
- EMP012 (Quality)

**Executive:**

- EMP015 (CEO)
- EMP016 (CTO)
- EMP018 (CPO)
- EMP017 (VP Engineering)

**Product & Customer:**

- EMP019 (Product Manager)
- EMP022 (VP Sales)
- EMP023 (Customer Success)

---

## Technical Architecture

### A2A Messaging

- **Protocol:** Local A2A using flume MPMC channels
- **Performance:** < 1μs latency for agent-to-agent communication
- **Capacity:** 100 messages per agent channel

### Knowledge Management

- **Learning:** Agents query past experiences before tasks
- **Quality Tracking:** Automatic 0.3-0.9 scoring based on success
- **Task Classification:** Auto-categorize (design, implementation, testing, etc.)
- **Prompt Enhancement:** Context-aware execution with historical data

### Task Execution Flow

```text
1. Task Assignment → A2A message to agent
2. Knowledge Query → Retrieve similar past tasks
3. Enhanced Prompt → Build context with history
4. Execute Task → Agent processes with LLM
5. Store Learning → Create knowledge entry
6. Quality Score → Calculate based on outcome
```

---

## Running the Example

### Prerequisites

- Ollama running locally (default: <http://localhost:11434>)
- At least 8GB RAM available
- Rust toolchain installed

### Quick Start

```bash
cargo run --example robotech_industries_organization_example
```

### Expected Output

```text
🤖 RoboTech Industries - Multi-Agent Organization Demo

==========================================================

🎯 MISSION: Build 3 Humanoid Robot Variants

   Robo-1: Home Companion (chores, security, emotional support)
   Robo-2: Construction Assistant (Robo-1 + heavy lifting)
   Robo-3: Rescue Operations (wildfire + coastguard)

==========================================================

✅ Organization created: RoboTech Industries
   Total roles available: 110+
   Agent count: 25

✅ Workspaces configured: 8

🚀 Spawning AI agents...

  ✓ Spawned: EMP001 (ResearchEngineerScaling) with learning capabilities
  ✓ Spawned: EMP002 (ResearchEngineerAutonomy) with learning capabilities
  ...

✅ All agents spawned and ready

🎯 Executing Multi-Workspace Projects

==========================================================

🚀 Starting Development of 3 Humanoid Robot Variants

🏠 Project 1: Robo-1 Home Companion Development

   ✅ Completed 3 Robo-1 development tasks

🏗️ Project 2: Robo-2 Construction Assistant Development

   ✅ Completed 3 Robo-2 development tasks

🚒 Project 3: Robo-3 Rescue Operations Development

   ✅ Completed 4 Robo-3 development tasks

==========================================================

✅ All 3 Humanoid Robot Variants Development Initiated!

   🏠 Robo-1: 3 tasks completed
   🏗️ Robo-2: 3 tasks completed
   🚒 Robo-3: 4 tasks completed

   Total: 10 development tasks executed

📊 Final Organization State

🤖 Robot Variant Development Summary:

   📦 Robo-1: Home Companion
      Description: Develop home assistance capabilities: chores, security, emotional companionship
      Team: 5 agents
      Progress: 3/3 tasks completed

   📦 Robo-2: Construction Assistant
      Description: Extend Robo-1 with heavy-duty actuators and construction capabilities
      Team: 5 agents
      Progress: 3/3 tasks completed

   📦 Robo-3: Rescue Operations
      Description: Advanced capabilities for wildfire rescue and coastguard operations
      Team: 6 agents
      Progress: 4/4 tasks completed

✅ Demo complete!
```

---

## Development Tasks by Variant

### Robo-1 Tasks

1. **Design Home Assistant AI** (Critical)
   - Household chores AI: cleaning, organizing, maintenance
   - Agent: EMP001

2. **Build Security & Emotional Intelligence** (Critical)
   - Security monitoring + emotional companionship
   - Agent: EMP004

3. **Design Safe Home-Use Actuators** (High)
   - Safe, quiet actuators for home environment
   - Agent: EMP007

### Robo-2 Tasks

1. **Design Heavy-Duty Actuator System** (Critical)
   - 50+ kg lifting capacity
   - Agent: EMP002

2. **Develop Load-Balancing Control System** (Critical)
   - Stable load handling + construction navigation
   - Agent: EMP008

3. **Build Construction Safety Features** (High)
   - Safety protocols for construction sites
   - Agent: EMP010

### Robo-3 Tasks

1. **Design Extreme Environment Systems** (Critical)
   - High-heat (wildfire) + marine environment protection
   - Agent: EMP003

2. **Build Advanced Perception for Rescue** (Critical)
   - Victim detection, smoke/water navigation, threat assessment
   - Agent: EMP005

3. **Implement Emergency Response Protocols** (Critical)
   - Fail-safe systems + emergency automation
   - Agent: EMP006

4. **Design Rescue Equipment Integration** (High)
    - Thermal imaging, water pumps, rescue tools, communications
    - Agent: EMP008

---

## Key Features Demonstrated

### Multi-Agent Coordination

✅ 25 agents across 8 workspaces
✅ Cross-functional teams
✅ Role-specific expertise
✅ Concurrent task execution

### A2A Communication

✅ High-performance messaging (flume channels)
✅ Type-safe agent-to-agent communication
✅ Automatic agent registration
✅ Message prioritization

### Organizational Learning

✅ Knowledge query before tasks
✅ Enhanced prompts with history
✅ Quality scoring (0.3-0.9)
✅ Task type classification
✅ Learning storage after completion

### Advanced Capabilities

✅ 110+ organizational roles available
✅ Role-specific system prompts
✅ Task dependency management
✅ Priority-based execution
✅ Collaborative workspaces

---

## Architecture Highlights

### Performance

- **Latency:** < 1μs per message (in-memory A2A)
- **Throughput:** Hundreds of agents supported
- **Memory:** < 1MB for 25 agents
- **Concurrency:** Lock-free message passing

### Scalability

- **Horizontal:** Add more agents easily

- **Vertical:** Increase workspace complexity
- **Distributed:** Can extend to Redis/RabbitMQ A2A

### Learning

- **Context-Aware:** Tasks executed with historical knowledge
- **Quality Tracking:** Automatic scoring of outcomes
- **Continuous Improvement:** Agents learn from past work
- **Knowledge Sharing:** Cross-workspace learning

---

## Extending the Example

### Add More Robot Variants

```rust
let robo4_ws = CollaborativeWorkspace::new(
    "Robo-4: Medical Assistant".to_string(),
    "Healthcare and medical assistance capabilities".to_string(),
);
```

### Add More Agents

```rust
let doctor = OrganizationAgent::new(
    "Dr. Smith".to_string(),
    OrganizationRole::MedicalEngineer, // Custom role
);
org.add_agent(doctor);
```

### Add Knowledge Management

```rust
let learning_config = LearningConfig {
    soft_limit_best_practices: 1000,
    hard_limit_best_practices: 5000,
    enable_auto_consolidation: true,
    ..Default::default()
};

let knowledge_manager = AdaptiveKnowledgeManager::new(learning_config);
let coordinator = AgentCoordinator::new(org)
    .with_knowledge_manager(knowledge_manager);
```

---

## Related Documentation

- [Organization System Overview](../docs/ORGANIZATION.md)
- [A2A Communication](../docs/A2A_COMMUNICATION.md)
- [Organizational Learning](../docs/ORGANIZATIONAL-LEARNING.md)
- [A2A & Knowledge Integration](../docs/ORGANIZATION-A2A-KNOWLEDGE.md)

---

## Success Criteria

When you run this example successfully, you should see:

- ✅ All 25 agents spawned with learning capabilities
- ✅ 10 development tasks executed across 3 robot variants
- ✅ All tasks completed successfully
- ✅ Knowledge-enhanced execution (check logs for context size)
- ✅ Final organization state showing progress

---

## Notes

- This is a **simulation** - no actual LLM calls unless Ollama is running
- The example demonstrates **organization structure and coordination**
- **A2A messaging** is fully functional
- **Knowledge infrastructure** is in place (query/store)
- **Learning context** is added to prompts automatically
