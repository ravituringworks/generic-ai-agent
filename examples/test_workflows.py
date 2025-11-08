#!/usr/bin/env python3
"""
Workflow Example Tester
Tests all workflow examples by executing them via the API
"""

import json
import sys
import time
import urllib.request
import urllib.error
from pathlib import Path
from typing import Dict, Any, Tuple

# Configuration
API_BASE = "http://127.0.0.1:8080/workflow-ui"
WORKFLOWS_DIR = Path("examples/workflows")
SAMPLE_DATA_DIR = Path("examples/sample-data")

# Colors for output
class Colors:
    GREEN = '\033[0;32m'
    RED = '\033[0;31m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    BOLD = '\033[1m'
    NC = '\033[0m'  # No Color

def print_status(msg: str):
    print(f"{Colors.BLUE}[INFO]{Colors.NC} {msg}")

def print_success(msg: str):
    print(f"{Colors.GREEN}[SUCCESS]{Colors.NC} {msg}")

def print_error(msg: str):
    print(f"{Colors.RED}[ERROR]{Colors.NC} {msg}")

def print_warning(msg: str):
    print(f"{Colors.YELLOW}[WARNING]{Colors.NC} {msg}")

def check_daemon() -> bool:
    """Check if daemon is running"""
    print_status("Checking if daemon is running...")
    try:
        req = urllib.request.Request(f"{API_BASE}/nodes")
        with urllib.request.urlopen(req, timeout=5) as response:
            if response.status == 200:
                print_success("Daemon is running")
                return True
    except (urllib.error.URLError, urllib.error.HTTPError):
        pass

    print_error("Daemon is not running")
    print_error("Please start it with: cargo run --bin agency-daemon --release")
    return False

def load_workflow(filepath: Path) -> Dict[str, Any]:
    """Load workflow from JSON file"""
    with open(filepath, 'r') as f:
        return json.load(f)

def create_workflow(workflow_data: Dict[str, Any]) -> Tuple[bool, str]:
    """Create workflow via API"""
    try:
        # Create workflow with name, description, nodes, and connections
        create_data = {
            "name": workflow_data.get("name", "Test Workflow"),
            "description": workflow_data.get("description", ""),
            "nodes": workflow_data.get("nodes", []),
            "connections": workflow_data.get("connections", [])
        }

        data = json.dumps(create_data).encode('utf-8')
        req = urllib.request.Request(
            f"{API_BASE}/workflows",
            data=data,
            headers={"Content-Type": "application/json"},
            method='POST'
        )

        with urllib.request.urlopen(req, timeout=10) as response:
            if response.status in [200, 201]:
                result = json.loads(response.read().decode('utf-8'))
                workflow_id = result.get("id")
                if workflow_id:
                    return True, workflow_id
                else:
                    print_error(f"No workflow ID in response")
                    return False, ""
            else:
                print_error(f"Failed to create workflow: {response.status}")
                return False, ""

    except Exception as e:
        print_error(f"Exception creating workflow: {e}")
        return False, ""

def execute_workflow(workflow_id: str) -> Tuple[bool, Dict[str, Any]]:
    """Execute workflow via API"""
    try:
        req = urllib.request.Request(
            f"{API_BASE}/workflows/{workflow_id}/execute",
            method='POST'
        )

        with urllib.request.urlopen(req, timeout=60) as response:
            if response.status == 200:
                result = json.loads(response.read().decode('utf-8'))
                return True, result
            else:
                print_error(f"Execution failed: {response.status}")
                return False, {}

    except Exception as e:
        print_error(f"Exception executing workflow: {e}")
        return False, {}

def delete_workflow(workflow_id: str):
    """Delete workflow via API"""
    try:
        req = urllib.request.Request(
            f"{API_BASE}/workflows/{workflow_id}",
            method='DELETE'
        )
        with urllib.request.urlopen(req, timeout=5) as response:
            pass
    except:
        pass

def test_workflow(filepath: Path) -> bool:
    """Test a single workflow"""
    workflow_name = filepath.stem

    print("\n" + "=" * 50)
    print_status(f"Testing: {workflow_name}")
    print("=" * 50)

    try:
        # Load workflow
        workflow_data = load_workflow(filepath)

        # Create workflow
        print_status(f"Creating workflow: {workflow_name}")
        success, workflow_id = create_workflow(workflow_data)

        if not success:
            print_error(f"Failed to create workflow: {workflow_name}")
            return False

        print_success(f"Created workflow with ID: {workflow_id}")

        # Small delay
        time.sleep(0.5)

        # Execute workflow
        print_status(f"Executing workflow: {workflow_name}")
        success, result = execute_workflow(workflow_id)

        if success:
            status = result.get("status", "unknown")
            if status == "completed":
                print_success(f"Workflow executed successfully")
                print(f"  Steps executed: {result.get('steps_executed', 0)}")
                print(f"  Execution time: {result.get('execution_time_ms', 0)}ms")

                # Print output summary
                output = result.get("output", {})
                if output:
                    print(f"  Output nodes: {len(output)}")
                    for idx, key in enumerate(list(output.keys())[:3]):  # Show first 3 outputs
                        val_str = str(output[key])
                        if len(val_str) > 100:
                            val_str = val_str[:100] + "..."
                        print(f"    - {key}: {val_str}")

                # Cleanup
                delete_workflow(workflow_id)
                return True
            else:
                print_error(f"Workflow completed with status: {status}")
                print(f"  Result: {json.dumps(result, indent=2)[:500]}")
                delete_workflow(workflow_id)
                return False
        else:
            print_error(f"Workflow execution failed")
            delete_workflow(workflow_id)
            return False

    except Exception as e:
        print_error(f"Test failed with exception: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    print("\n" + "╔" + "=" * 48 + "╗")
    print("║" + " " * 15 + "Workflow Examples Tester" + " " * 9 + "║")
    print("╚" + "=" * 48 + "╝\n")

    # Check daemon
    if not check_daemon():
        sys.exit(1)

    # Check sample data
    if not SAMPLE_DATA_DIR.exists():
        print_warning(f"Sample data directory not found: {SAMPLE_DATA_DIR}")
        print_warning("Some workflows may fail without input files")

    # Track results
    total = 0
    passed = 0
    failed = 0

    # Get all workflow files
    workflow_files = sorted(WORKFLOWS_DIR.glob("*.json"))

    if not workflow_files:
        print_error(f"No workflow files found in {WORKFLOWS_DIR}")
        sys.exit(1)

    # Test each workflow
    for workflow_file in workflow_files:
        total += 1

        if test_workflow(workflow_file):
            passed += 1
        else:
            failed += 1

        # Delay between tests
        time.sleep(1)

    # Print summary
    print("\n" + "=" * 50)
    print(f"{Colors.BOLD}Test Summary{Colors.NC}")
    print("=" * 50)
    print(f"Total workflows: {total}")
    print_success(f"Passed: {passed}")
    if failed > 0:
        print_error(f"Failed: {failed}")
    print()

    # Exit with appropriate code
    sys.exit(0 if failed == 0 else 1)

if __name__ == "__main__":
    main()
