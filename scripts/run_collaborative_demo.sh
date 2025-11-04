#!/bin/bash
# Collaborative Robotics Workspace Demo Runner
# This script runs the multi-agent collaborative workspace example

set -e

echo "🚀 Collaborative Robotics Workspace Demo"
echo "========================================="
echo ""
echo "This demo will:"
echo "  ✓ Initialize 3 specialized AI agents"
echo "  ✓ Create a collaborative workspace"
echo "  ✓ Generate Python simulation code"
echo "  ✓ Produce documentation artifacts"
echo "  ✓ Cross-review and verify outputs"
echo ""
echo "Estimated runtime: 25-30 seconds"
echo ""
read -p "Press Enter to start demo..."
echo ""

# Run the example
cargo run --example collaborative_robotics_workspace

echo ""
echo "========================================="
echo "✅ Demo Complete!"
echo ""
echo "Generated artifacts can be found at:"
echo "  examples/robotics_workspace/humanoid_robot_project/"
echo ""
echo "To view the generated code:"
echo "  cat examples/robotics_workspace/humanoid_robot_project/code/*.py"
echo ""
echo "To run the simulation (requires matplotlib and numpy):"
echo "  python3 examples/robotics_workspace/humanoid_robot_project/code/*_implementation.py"
echo ""
echo "For more details, see:"
echo "  examples/robotics_workspace/README.md"
echo ""
