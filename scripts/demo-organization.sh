#!/usr/bin/env zsh

# Multi-Agent Organization Demo Script
# Demonstrates the RoboTech Industries organization system

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Emoji support
ROCKET="🚀"
CHECK="✅"
ROBOT="🤖"
FACTORY="🏭"
DOCS="📚"
CHART="📊"

echo "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo "${BLUE}${ROCKET} RoboTech Industries Multi-Agent Organization Demo${NC}"
echo "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""

# The example now creates its own output directory
EXAMPLE_OUTPUT_DIR="./output/robotech_organization_output"

# Create additional demo output directory for script-generated files
OUTPUT_DIR="./demo-outputs/organization-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$OUTPUT_DIR"

echo "${GREEN}${CHECK} Example will output to: ${EXAMPLE_OUTPUT_DIR}${NC}"
echo "${GREEN}${CHECK} Script output directory: ${OUTPUT_DIR}${NC}"
echo ""

# Run the organization example and capture output
echo "${YELLOW}${FACTORY} Running organization example...${NC}"
echo ""

cargo run --example robotech_industries_organization_example 2>&1 | tee "$OUTPUT_DIR/organization-execution.log"

echo ""
echo "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo "${BLUE}${CHART} Demo Summary${NC}"
echo "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Extract key metrics from the log
TOTAL_AGENTS=$(grep -o "Total Agents: [0-9]*" "$OUTPUT_DIR/organization-execution.log" | tail -1 | grep -o "[0-9]*" || echo "0")
TOTAL_WORKSPACES=$(grep -o "Total Workspaces: [0-9]*" "$OUTPUT_DIR/organization-execution.log" | tail -1 | grep -o "[0-9]*" || echo "0")
COMPLETED_TASKS=$(grep -o "Completed [0-9]* [A-Za-z ]*tasks" "$OUTPUT_DIR/organization-execution.log" | wc -l || echo "0")

echo "${GREEN}${CHECK} Organization Metrics:${NC}"
echo "   - Total Agents: ${TOTAL_AGENTS}"
echo "   - Total Workspaces: ${TOTAL_WORKSPACES}"
echo "   - Task Executions: ${COMPLETED_TASKS}"
echo ""

# Copy example outputs to demo directory
if [ -d "${EXAMPLE_OUTPUT_DIR}" ]; then
    echo "${GREEN}${CHECK} Copying example outputs...${NC}"
    cp -r "${EXAMPLE_OUTPUT_DIR}/reports" "$OUTPUT_DIR/" 2>/dev/null || true
    cp -r "${EXAMPLE_OUTPUT_DIR}/logs" "$OUTPUT_DIR/" 2>/dev/null || true
    echo ""
fi

# Create a summary report
REPORT_FILE="$OUTPUT_DIR/demo-report.md"

cat > "$REPORT_FILE" << EOF
# RoboTech Industries Organization Demo Report

**Generated:** $(date '+%Y-%m-%d %H:%M:%S')

## Overview

This demo showcases the multi-agent organization system with collaborative workspaces.

## Execution Summary

- **Total Agents:** ${TOTAL_AGENTS}
- **Total Workspaces:** ${TOTAL_WORKSPACES}
- **Task Executions:** ${COMPLETED_TASKS}

## Organization Structure

### Agent Categories

The organization includes agents from 11 specialized categories:
1. Research & AI
2. Software Engineering
3. Security
4. Hardware Engineering
5. Robotics Engineering
6. Manufacturing & Production
7. Supply Chain & Quality
8. Infrastructure & IT
9. Service & Support
10. Engineering Specializations
11. Legal & Finance

### Collaborative Workspaces

The demo created multiple collaborative workspaces for 3 robot variants:
- Robo-1: Home Companion
- Robo-2: Construction Assistant
- Robo-3: Rescue Operations
- Manufacturing Excellence
- Supply Chain & Analytics
- Executive Leadership
- Product Strategy
- Customer & Market Success

## Key Features Demonstrated

1. **Multi-Agent Coordination**
   - Agents working across different workspaces
   - Role-based task assignment
   - Cross-workspace dependencies

2. **Task Management**
   - Priority-based scheduling (Critical, High, Medium, Low)
   - Dependency tracking
   - Status monitoring

3. **Communication**
   - Message queue for agent-to-agent communication
   - Task assignment and result reporting
   - Status updates

4. **Organization Management**
   - Agent availability tracking
   - Workspace membership
   - Role categorization

5. **Knowledge Management**
   - Persistent memory with embeddings
   - Organizational learning across tasks
   - Context-aware agent prompts

## Work Products

The organization system produced the following work products during execution:

### Completed Tasks

EOF

# Extract completed tasks from log
echo "Extracting task information..."
grep "Completed" "$OUTPUT_DIR/organization-execution.log" | while read -r line; do
    echo "- $line" >> "$REPORT_FILE"
done

cat >> "$REPORT_FILE" << EOF

## Technical Details

### Architecture

\`\`\`
┌─────────────────────────────────────────────────────────────┐
│                   Organization Daemon                       │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Agent Coordinator                        │  │
│  │  - Message Queue                                      │  │
│  │  - Task Routing                                       │  │
│  │  - Workspace Orchestration                            │  │
│  └───────────────────────────────────────────────────────┘  │
│                          │                                  │
│      ┌───────────────────┼─────────────────┐                │
│      │                   │                 │                │
│  ┌───▼────┐      ┌───────▼───────┐  ┌──────▼──────┐         │
│  │ Agents │      │  Workspaces   │  │    Tasks    │         │
│  │        │◄────►│               │◄─┤             │         │
│  └────────┘      └───────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
\`\`\`

### Implementation

- **Language:** Rust
- **Async Runtime:** Tokio
- **Coordination:** Message-based architecture
- **State Management:** Arc<RwLock<T>> for concurrent access
- **LLM:** gpt-oss:20b-cloud for text generation
- **Embeddings:** nomic-embed-text (local) for memory

## Output Files

### Example-Generated Files (${EXAMPLE_OUTPUT_DIR})
- \`reports/summary.md\` - Organization summary with all agents and workspaces
- \`reports/organization_state.json\` - Full serialized organization state

### Script-Generated Files (${OUTPUT_DIR})
- \`organization-execution.log\` - Full execution trace
- \`demo-report.md\` - This comprehensive demo report
- \`workspace-summary.txt\` - Workspace task breakdown

## Next Steps

1. Review the execution log for detailed agent interactions
2. Examine workspace coordination patterns
3. Analyze task completion metrics
4. Explore cross-workspace collaboration
5. Review the organization_state.json for full system state

---

*Generated by RoboTech Industries Organization System*
EOF

echo "${GREEN}${DOCS} Generated demo report: ${REPORT_FILE}${NC}"
echo ""

# Display the report
echo "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo "${BLUE}${DOCS} Demo Report Preview${NC}"
echo "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Show the example-generated summary if it exists
if [ -f "${EXAMPLE_OUTPUT_DIR}/reports/summary.md" ]; then
    head -50 "${EXAMPLE_OUTPUT_DIR}/reports/summary.md"
else
    head -50 "$REPORT_FILE"
fi

echo ""
echo "${YELLOW}... (see full report in ${REPORT_FILE})${NC}"
echo ""

# Create a workspace summary
WORKSPACE_SUMMARY="$OUTPUT_DIR/workspace-summary.txt"

echo "Workspace Task Distribution" > "$WORKSPACE_SUMMARY"
echo "===========================" >> "$WORKSPACE_SUMMARY"
echo "" >> "$WORKSPACE_SUMMARY"

grep "Project [0-9]:" "$OUTPUT_DIR/organization-execution.log" | while read -r line; do
    echo "$line" >> "$WORKSPACE_SUMMARY"
done

echo ""
echo "${GREEN}${CHECK} Generated workspace summary: ${WORKSPACE_SUMMARY}${NC}"
echo ""

# Final summary
echo "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo "${BLUE}${ROCKET} Demo Complete!${NC}"
echo "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo "${GREEN}Output files created in multiple locations:${NC}"
echo ""
echo "${CYAN}Example Output (${EXAMPLE_OUTPUT_DIR}):${NC}"
echo "  1. ${CYAN}reports/summary.md${NC} - Organization summary report"
echo "  2. ${CYAN}reports/organization_state.json${NC} - Full organization state"
echo ""
echo "${CYAN}Script Output (${OUTPUT_DIR}):${NC}"
echo "  1. ${CYAN}organization-execution.log${NC} - Full execution trace"
echo "  2. ${CYAN}demo-report.md${NC} - Script-generated demo report"
echo "  3. ${CYAN}workspace-summary.txt${NC} - Workspace task breakdown"
echo "  4. ${CYAN}reports/${NC} - Copied from example output"
echo ""
echo "${YELLOW}To view the organization summary:${NC}"
echo "  cat ${EXAMPLE_OUTPUT_DIR}/reports/summary.md"
echo ""
echo "${YELLOW}To view the organization state (JSON):${NC}"
echo "  cat ${EXAMPLE_OUTPUT_DIR}/reports/organization_state.json"
echo ""
echo "${YELLOW}To view execution logs:${NC}"
echo "  less ${OUTPUT_DIR}/organization-execution.log"
echo ""
echo "${GREEN}${CHECK} Demo completed successfully!${NC}"
echo ""
