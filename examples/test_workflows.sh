#!/bin/bash

# Workflow Example Tester
# Tests all workflow examples by executing them via the API

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

API_BASE="http://127.0.0.1:8080/workflow-ui"
WORKFLOWS_DIR="examples/workflows"
SAMPLE_DATA_DIR="examples/sample-data"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Function to check if daemon is running
check_daemon() {
    print_status "Checking if daemon is running..."
    if curl -s -f "${API_BASE}/nodes" > /dev/null 2>&1; then
        print_success "Daemon is running"
        return 0
    else
        print_error "Daemon is not running. Please start it with: cargo run --bin agency-daemon --release"
        exit 1
    fi
}

# Function to create workflow from JSON file
create_workflow() {
    local workflow_file=$1
    local workflow_name=$(basename "$workflow_file" .json)

    print_status "Creating workflow: $workflow_name"

    # Read the JSON file
    local workflow_json=$(cat "$workflow_file")

    # Create the workflow via API
    local response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$workflow_json" \
        "${API_BASE}/workflows" 2>&1)

    # Extract workflow ID from response
    local workflow_id=$(echo "$response" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)

    if [ -z "$workflow_id" ]; then
        print_error "Failed to create workflow: $workflow_name"
        echo "Response: $response"
        return 1
    fi

    print_success "Created workflow with ID: $workflow_id"
    echo "$workflow_id"
}

# Function to execute workflow
execute_workflow() {
    local workflow_id=$1
    local workflow_name=$2

    print_status "Executing workflow: $workflow_name"

    local response=$(curl -s -X POST \
        "${API_BASE}/workflows/${workflow_id}/execute" 2>&1)

    # Check if execution was successful
    if echo "$response" | grep -q '"status":"completed"' || echo "$response" | grep -q '"status":"error"'; then
        print_success "Workflow executed"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
        return 0
    else
        print_error "Workflow execution may have failed"
        echo "$response"
        return 1
    fi
}

# Function to delete workflow
delete_workflow() {
    local workflow_id=$1

    curl -s -X DELETE "${API_BASE}/workflows/${workflow_id}" > /dev/null 2>&1
}

# Function to test a single workflow
test_workflow() {
    local workflow_file=$1
    local workflow_name=$(basename "$workflow_file" .json)

    echo ""
    echo "=========================================="
    print_status "Testing: $workflow_name"
    echo "=========================================="

    # Create workflow
    local workflow_id=$(create_workflow "$workflow_file")

    if [ -z "$workflow_id" ]; then
        print_error "Skipping execution for $workflow_name"
        return 1
    fi

    # Small delay to ensure workflow is created
    sleep 0.5

    # Execute workflow
    if execute_workflow "$workflow_id" "$workflow_name"; then
        print_success "Test passed: $workflow_name"

        # Cleanup
        delete_workflow "$workflow_id"
        return 0
    else
        print_error "Test failed: $workflow_name"

        # Cleanup
        delete_workflow "$workflow_id"
        return 1
    fi
}

# Main execution
main() {
    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║   Workflow Examples Tester             ║"
    echo "╔════════════════════════════════════════╗"
    echo ""

    # Check if daemon is running
    check_daemon

    # Check if sample data exists
    if [ ! -d "$SAMPLE_DATA_DIR" ]; then
        print_warning "Sample data directory not found at $SAMPLE_DATA_DIR"
        print_warning "Some workflows may fail without input files"
    fi

    # Track results
    local total=0
    local passed=0
    local failed=0
    local skipped=0

    # Test each workflow
    for workflow_file in "$WORKFLOWS_DIR"/*.json; do
        if [ ! -f "$workflow_file" ]; then
            continue
        fi

        total=$((total + 1))

        # Skip certain examples that require specific setup
        local workflow_name=$(basename "$workflow_file" .json)

        # Check if this workflow needs files that don't exist
        case "$workflow_name" in
            *"file_io"*|*"chunking_and_summarization"*)
                if [ ! -f "$SAMPLE_DATA_DIR/input.txt" ] && [ ! -f "$SAMPLE_DATA_DIR/long_document.txt" ]; then
                    print_warning "Skipping $workflow_name - missing input files"
                    skipped=$((skipped + 1))
                    continue
                fi
                ;;
            *"sentiment"*)
                if [ ! -f "$SAMPLE_DATA_DIR/customer_feedback.txt" ]; then
                    print_warning "Skipping $workflow_name - missing customer_feedback.txt"
                    skipped=$((skipped + 1))
                    continue
                fi
                ;;
            *"translation"*)
                if [ ! -f "$SAMPLE_DATA_DIR/phrases.txt" ]; then
                    print_warning "Skipping $workflow_name - missing phrases.txt"
                    skipped=$((skipped + 1))
                    continue
                fi
                ;;
            *"quality"*)
                if [ ! -f "$SAMPLE_DATA_DIR/draft_content.txt" ]; then
                    print_warning "Skipping $workflow_name - missing draft_content.txt"
                    skipped=$((skipped + 1))
                    continue
                fi
                ;;
        esac

        if test_workflow "$workflow_file"; then
            passed=$((passed + 1))
        else
            failed=$((failed + 1))
        fi

        # Small delay between tests
        sleep 1
    done

    # Print summary
    echo ""
    echo "=========================================="
    echo "Test Summary"
    echo "=========================================="
    echo "Total workflows: $total"
    print_success "Passed: $passed"
    if [ $failed -gt 0 ]; then
        print_error "Failed: $failed"
    fi
    if [ $skipped -gt 0 ]; then
        print_warning "Skipped: $skipped"
    fi
    echo ""

    # Exit with appropriate code
    if [ $failed -gt 0 ]; then
        exit 1
    else
        exit 0
    fi
}

# Run main
main
