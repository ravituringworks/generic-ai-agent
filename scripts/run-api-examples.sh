#!/usr/bin/env bash
#
# Run the Agency daemon and demonstrate API usage with curl, Python, and JavaScript
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
API_URL="http://127.0.0.1:8080"
DAEMON_BIN="${DAEMON_BIN:-./target/debug/agency-daemon}"
DAEMON_PID_FILE="/tmp/agency-daemon-examples.pid"
DAEMON_LOG_FILE="/tmp/agency-daemon-examples.log"

# Function to print colored output
print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# Function to check if daemon is running
is_daemon_running() {
    if [ -f "$DAEMON_PID_FILE" ]; then
        PID=$(cat "$DAEMON_PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            return 0
        fi
    fi
    return 1
}

# Function to start the daemon
start_daemon() {
    print_header "Starting Agency Daemon"
    
    if is_daemon_running; then
        print_info "Daemon is already running (PID: $(cat $DAEMON_PID_FILE))"
        return 0
    fi
    
    if [ ! -f "$DAEMON_BIN" ]; then
        print_error "Daemon binary not found at: $DAEMON_BIN"
        print_info "Build it with: cargo build --bin agency-daemon"
        exit 1
    fi
    
    # Start daemon in background
    "$DAEMON_BIN" > "$DAEMON_LOG_FILE" 2>&1 &
    DAEMON_PID=$!
    echo "$DAEMON_PID" > "$DAEMON_PID_FILE"
    
    # Wait for daemon to be ready
    print_info "Waiting for daemon to start..."
    for i in {1..30}; do
        if curl -s "$API_URL/health" > /dev/null 2>&1; then
            print_success "Daemon started successfully (PID: $DAEMON_PID)"
            return 0
        fi
        sleep 1
    done
    
    print_error "Daemon failed to start within 30 seconds"
    print_info "Check logs at: $DAEMON_LOG_FILE"
    exit 1
}

# Function to stop the daemon
stop_daemon() {
    print_header "Stopping Agency Daemon"
    
    if [ -f "$DAEMON_PID_FILE" ]; then
        PID=$(cat "$DAEMON_PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            kill "$PID"
            rm -f "$DAEMON_PID_FILE"
            print_success "Daemon stopped"
        else
            print_info "Daemon was not running"
            rm -f "$DAEMON_PID_FILE"
        fi
    else
        print_info "No PID file found"
    fi
}

# Cleanup on exit
cleanup() {
    echo ""
    if [ "$KEEP_RUNNING" != "true" ]; then
        stop_daemon
    else
        print_info "Daemon left running (use --stop to stop it)"
    fi
}

trap cleanup EXIT INT TERM

# Function to run curl examples
run_curl_examples() {
    print_header "cURL Examples"
    
    echo -e "\n${GREEN}1. Health Check${NC}"
    echo "Command: curl -s \"$API_URL/health\" | jq"
    curl -s "$API_URL/health" | jq
    
    echo -e "\n${GREEN}2. Process a Simple Message${NC}"
    echo "Command: curl -X POST \"$API_URL/api/v1/agent/process\" \\"
    echo "  -H 'Content-Type: application/json' \\"
    echo "  -d '{\"message\": \"What is Rust?\", \"max_steps\": 5}' | jq"
    curl -X POST "$API_URL/api/v1/agent/process" \
        -H "Content-Type: application/json" \
        -d '{"message": "What is Rust?", "max_steps": 5}' | jq
    
    echo -e "\n${GREEN}3. Create a Workflow${NC}"
    WORKFLOW_ID="workflow-$(date +%s)"
    echo "Command: curl -X POST \"$API_URL/api/v1/workflows\" \\"
    echo "  -H 'Content-Type: application/json' \\"
    echo "  -d '{\"workflow_id\": \"$WORKFLOW_ID\", \"initial_message\": \"Process this task\", \"max_steps\": 20}' | jq"
    curl -X POST "$API_URL/api/v1/workflows" \
        -H "Content-Type: application/json" \
        -d "{\"workflow_id\": \"$WORKFLOW_ID\", \"initial_message\": \"Process this task\", \"max_steps\": 20}" | jq
    
    echo -e "\n${GREEN}4. List Workflow Snapshots${NC}"
    echo "Command: curl -s \"$API_URL/api/v1/workflows/snapshots\" | jq"
    curl -s "$API_URL/api/v1/workflows/snapshots" | jq
    
    echo -e "\n${GREEN}5. Get OpenAPI Spec${NC}"
    echo "Command: curl -s \"$API_URL/api-docs/openapi.json\" | jq '.info'"
    curl -s "$API_URL/api-docs/openapi.json" | jq '.info'
}

# Function to create Python example
create_python_example() {
    cat > /tmp/agency_api_example.py << 'EOF'
#!/usr/bin/env python3
"""
The Agency API - Python Client Example

Demonstrates using the Agency REST API from Python.
"""

import requests
import json
import sys
from typing import Dict, Any, Optional

API_URL = "http://127.0.0.1:8080"

class AgencyClient:
    """Client for The Agency REST API"""
    
    def __init__(self, base_url: str = API_URL):
        self.base_url = base_url
        self.session = requests.Session()
        self.session.headers.update({"Content-Type": "application/json"})
    
    def health(self) -> Dict[str, Any]:
        """Check API health"""
        response = self.session.get(f"{self.base_url}/health")
        response.raise_for_status()
        return response.json()
    
    def process(self, message: str, max_steps: Optional[int] = None) -> Dict[str, Any]:
        """Process a message through the agent"""
        data = {"message": message}
        if max_steps:
            data["max_steps"] = max_steps
        
        response = self.session.post(
            f"{self.base_url}/api/v1/agent/process",
            json=data
        )
        response.raise_for_status()
        return response.json()
    
    def create_workflow(self, workflow_id: str, initial_message: str, max_steps: int = 20) -> Dict[str, Any]:
        """Create a new workflow"""
        data = {
            "workflow_id": workflow_id,
            "initial_message": initial_message,
            "max_steps": max_steps
        }
        response = self.session.post(
            f"{self.base_url}/api/v1/workflows",
            json=data
        )
        response.raise_for_status()
        return response.json()
    
    def list_snapshots(self) -> list:
        """List all workflow snapshots"""
        response = self.session.get(f"{self.base_url}/api/v1/workflows/snapshots")
        response.raise_for_status()
        return response.json()
    
    def resume_workflow(self, snapshot_id: str) -> Dict[str, Any]:
        """Resume a workflow from a snapshot"""
        data = {"snapshot_id": snapshot_id}
        response = self.session.post(
            f"{self.base_url}/api/v1/workflows/resume",
            json=data
        )
        response.raise_for_status()
        return response.json()

def main():
    print("🐍 The Agency API - Python Client Example\n")
    
    client = AgencyClient()
    
    # 1. Health check
    print("1️⃣  Health Check")
    health = client.health()
    print(f"   Status: {health['status']}, Version: {health['version']}\n")
    
    # 2. Process a message
    print("2️⃣  Processing Message")
    result = client.process("Tell me about Python programming", max_steps=5)
    print(f"   Response: {result['response'][:100]}...")
    print(f"   Steps: {result['steps_executed']}, Completed: {result['completed']}\n")
    
    # 3. Create a workflow
    print("3️⃣  Creating Workflow")
    import time
    workflow_id = f"python-workflow-{int(time.time())}"
    workflow = client.create_workflow(workflow_id, "Analyze this data", max_steps=20)
    print(f"   Workflow ID: {workflow['workflow_id']}")
    print(f"   Status: {workflow['status']}\n")
    
    # 4. List snapshots
    print("4️⃣  Listing Snapshots")
    snapshots = client.list_snapshots()
    print(f"   Total snapshots: {len(snapshots)}\n")
    
    print("✅ Python examples completed!")

if __name__ == "__main__":
    try:
        main()
    except requests.exceptions.ConnectionError:
        print("❌ Error: Could not connect to the Agency daemon")
        print("   Make sure it's running on http://127.0.0.1:8080")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Error: {e}")
        sys.exit(1)
EOF
    chmod +x /tmp/agency_api_example.py
}

# Function to run Python examples
run_python_examples() {
    print_header "Python Examples"
    
    if ! command -v python3 &> /dev/null; then
        print_error "Python 3 is not installed"
        return 1
    fi
    
    # Check if requests library is available
    if ! python3 -c "import requests" 2>/dev/null; then
        print_info "Installing requests library..."
        pip3 install requests --quiet || print_error "Failed to install requests"
    fi
    
    create_python_example
    print_info "Running Python example..."
    python3 /tmp/agency_api_example.py
}

# Function to create JavaScript/Node.js example
create_javascript_example() {
    cat > /tmp/agency_api_example.js << 'EOF'
#!/usr/bin/env node
/**
 * The Agency API - JavaScript/Node.js Client Example
 * 
 * Demonstrates using the Agency REST API from Node.js.
 * Run with: node agency_api_example.js
 */

const API_URL = 'http://127.0.0.1:8080';

class AgencyClient {
    constructor(baseUrl = API_URL) {
        this.baseUrl = baseUrl;
    }

    async request(endpoint, options = {}) {
        const url = `${this.baseUrl}${endpoint}`;
        const response = await fetch(url, {
            ...options,
            headers: {
                'Content-Type': 'application/json',
                ...options.headers
            }
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        return response.json();
    }

    async health() {
        return this.request('/health');
    }

    async process(message, maxSteps = null) {
        const body = { message };
        if (maxSteps) body.max_steps = maxSteps;
        
        return this.request('/api/v1/agent/process', {
            method: 'POST',
            body: JSON.stringify(body)
        });
    }

    async createWorkflow(workflowId, initialMessage, maxSteps = 20) {
        return this.request('/api/v1/workflows', {
            method: 'POST',
            body: JSON.stringify({
                workflow_id: workflowId,
                initial_message: initialMessage,
                max_steps: maxSteps
            })
        });
    }

    async listSnapshots() {
        return this.request('/api/v1/workflows/snapshots');
    }

    async resumeWorkflow(snapshotId) {
        return this.request('/api/v1/workflows/resume', {
            method: 'POST',
            body: JSON.stringify({ snapshot_id: snapshotId })
        });
    }
}

async function main() {
    console.log('🟨 The Agency API - JavaScript/Node.js Client Example\n');

    const client = new AgencyClient();

    try {
        // 1. Health check
        console.log('1️⃣  Health Check');
        const health = await client.health();
        console.log(`   Status: ${health.status}, Version: ${health.version}\n`);

        // 2. Process a message
        console.log('2️⃣  Processing Message');
        const result = await client.process('Tell me about JavaScript', 5);
        console.log(`   Response: ${result.response.substring(0, 100)}...`);
        console.log(`   Steps: ${result.steps_executed}, Completed: ${result.completed}\n`);

        // 3. Create a workflow
        console.log('3️⃣  Creating Workflow');
        const workflowId = `js-workflow-${Date.now()}`;
        const workflow = await client.createWorkflow(workflowId, 'Process this task', 20);
        console.log(`   Workflow ID: ${workflow.workflow_id}`);
        console.log(`   Status: ${workflow.status}\n`);

        // 4. List snapshots
        console.log('4️⃣  Listing Snapshots');
        const snapshots = await client.listSnapshots();
        console.log(`   Total snapshots: ${snapshots.length}\n`);

        console.log('✅ JavaScript examples completed!');
    } catch (error) {
        if (error.cause?.code === 'ECONNREFUSED') {
            console.error('❌ Error: Could not connect to the Agency daemon');
            console.error('   Make sure it\'s running on http://127.0.0.1:8080');
        } else {
            console.error(`❌ Error: ${error.message}`);
        }
        process.exit(1);
    }
}

main();
EOF
    chmod +x /tmp/agency_api_example.js
}

# Function to run JavaScript examples
run_javascript_examples() {
    print_header "JavaScript/Node.js Examples"
    
    if ! command -v node &> /dev/null; then
        print_error "Node.js is not installed"
        return 1
    fi
    
    create_javascript_example
    print_info "Running Node.js example..."
    node /tmp/agency_api_example.js
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Run the Agency daemon and demonstrate API usage with different languages.

OPTIONS:
    --curl              Run only curl examples
    --python            Run only Python examples
    --javascript        Run only JavaScript/Node.js examples
    --keep-running      Keep daemon running after examples
    --stop              Stop the daemon if running
    --help              Show this help message

EXAMPLES:
    # Run all examples
    $0

    # Run only curl examples
    $0 --curl

    # Run examples and keep daemon running
    $0 --keep-running

    # Stop the daemon
    $0 --stop
EOF
}

# Main execution
main() {
    local run_curl=false
    local run_python=false
    local run_javascript=false
    local run_all=true
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --curl)
                run_curl=true
                run_all=false
                shift
                ;;
            --python)
                run_python=true
                run_all=false
                shift
                ;;
            --javascript)
                run_javascript=true
                run_all=false
                shift
                ;;
            --keep-running)
                KEEP_RUNNING=true
                shift
                ;;
            --stop)
                stop_daemon
                exit 0
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Start daemon
    start_daemon
    
    echo ""
    
    # Run examples
    if [ "$run_all" = true ] || [ "$run_curl" = true ]; then
        run_curl_examples
        echo ""
    fi
    
    if [ "$run_all" = true ] || [ "$run_python" = true ]; then
        run_python_examples
        echo ""
    fi
    
    if [ "$run_all" = true ] || [ "$run_javascript" = true ]; then
        run_javascript_examples
        echo ""
    fi
    
    print_success "All examples completed!"
    print_info "API Documentation: http://127.0.0.1:8080/api-docs/openapi.json"
    print_info "Swagger UI: Open docs/swagger-ui.html in your browser"
}

# Run main function
main "$@"
