# Demo Scripts

This directory contains demonstration scripts for the-agency framework.

## Organization Demo Script

### `demo-organization.sh`

Demonstrates the multi-agent organization system with collaborative workspaces.

#### Usage

```bash
./scripts/demo-organization.sh
```

#### What It Does

1. **Runs the Organization Example**
   - Executes the RoboTech Industries organization demo
   - Creates 14 agents across multiple roles
   - Sets up 5 collaborative workspaces
   - Coordinates 8 tasks across workspaces

2. **Captures Execution Output**
   - Full execution log with agent interactions
   - Task assignment and completion tracking
   - Workspace coordination details

3. **Generates Reports**
   - **demo-report.md**: Comprehensive markdown report
   - **organization-execution.log**: Full execution trace
   - **workspace-summary.txt**: Task distribution breakdown

#### Output Location

All outputs are saved to timestamped directories:
```
demo-outputs/organization-YYYYMMDD-HHMMSS/
├── organization-execution.log
├── demo-report.md
└── workspace-summary.txt
```

#### Demo Report Contents

The generated report includes:
- Execution metrics (agents, workspaces, tasks)
- Organization structure overview
- Agent category breakdown
- Workspace descriptions
- Key features demonstrated
- Completed task list
- Architecture diagram
- Technical implementation details

#### Example Output

```
═══════════════════════════════════════════════════════════
🚀 RoboTech Industries Multi-Agent Organization Demo
═══════════════════════════════════════════════════════════

✅ Created output directory: ./demo-outputs/organization-20250101-120000

🏭 Running organization example...

🤖 Multi-Agent Organization Demo

==================================================

✅ Organization created: RoboTech Industries
   Total roles available: 60+
   Agent count: 14

✅ Workspaces configured: 5

🚀 Spawning AI agents...

...

📊 Demo Summary
═══════════════════════════════════════════════════════════

✅ Organization Metrics:
   - Total Agents: 14
   - Total Workspaces: 5
   - Task Executions: 4

📚 Generated demo report: ./demo-outputs/organization-20250101-120000/demo-report.md
```

#### Requirements

- Rust toolchain (cargo)
- zsh shell (default on macOS)
- Unix-like environment (macOS, Linux, WSL)

#### Viewing Results

After running the demo:

```bash
# View the full report
cat demo-outputs/organization-YYYYMMDD-HHMMSS/demo-report.md

# View execution logs
less demo-outputs/organization-YYYYMMDD-HHMMSS/organization-execution.log

# View workspace summary
cat demo-outputs/organization-YYYYMMDD-HHMMSS/workspace-summary.txt
```

#### Features Demonstrated

1. **Multi-Agent Coordination**
   - 14 specialized agents with different roles
   - Role-based capabilities (Research, Engineering, Manufacturing, etc.)
   - Agent status tracking (Available, Busy, Offline)

2. **Collaborative Workspaces**
   - 5 workspaces: AI Research, Software Platform, Hardware Integration, Manufacturing, Supply Chain
   - Shared context between agents
   - Cross-workspace collaboration

3. **Task Management**
   - Priority-based scheduling (Critical, High, Medium, Low)
   - Task dependencies
   - Status tracking (Pending, InProgress, Completed)

4. **Communication**
   - Message queue for agent-to-agent communication
   - Task assignment notifications
   - Result reporting

5. **Organization Management**
   - Agent availability tracking
   - Workspace membership management
   - Role categorization (11 categories)

#### Troubleshooting

**Script won't run:**
```bash
chmod +x scripts/demo-organization.sh
```

**Cargo errors:**
```bash
# Make sure you're in the project root
cd /path/to/the-agency
cargo check
```

**No output directory:**
The script creates `demo-outputs/` automatically. Ensure you have write permissions in the project directory.

## Adding More Demo Scripts

To add additional demo scripts:

1. Create a new `.sh` file in this directory
2. Make it executable: `chmod +x scripts/your-script.sh`
3. Add documentation to this README
4. Follow the existing script structure for consistency

## License

Part of the-agency project.
