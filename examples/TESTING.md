# Testing Workflow Examples

This guide explains how to test all workflow examples programmatically without using the workflow builder UI.

## Quick Start

```bash
# 1. Start the daemon
cargo run --bin agency-daemon --release

# 2. In another terminal, run the test script
python3 examples/test_workflows.py
```

## What Gets Tested

The test script validates all 12 example workflows:

- ✅ **01_basic_text_generation** - Text input → LLM → text output
- ✅ **02_file_io_workflow** - File read → LLM → file write
- ✅ **03_conditional_routing** - LLM → conditional → routing
- ✅ **04_text_chunking** - Text splitter functionality
- ✅ **05_multi_provider_comparison** - Multiple LLM providers
- ✅ **06_if_then_else_container** - Container with nested nodes
- ✅ **07_foreach_loop_container** - Loop container
- ✅ **08_while_loop_container** - While loop container
- ✅ **09_document_chunking_and_summarization** - File → splitter → LLM → output
- ✅ **10_sentiment_analysis_routing** - File → multi-LLM → conditional → file
- ✅ **11_batch_translation_loop** - File → splitter → container + LLM
- ✅ **12_content_quality_pipeline** - File → parallel LLMs → conditional

## Test Script Features

### Automated Testing
- **Creates workflows** via API from JSON files
- **Executes workflows** programmatically
- **Validates results** with pass/fail reporting
- **Cleans up** by deleting test workflows after execution
- **Detailed output** showing execution time and steps

### Color-Coded Output
- 🔵 **INFO** - Test progress updates
- 🟢 **SUCCESS** - Workflow passed
- 🔴 **ERROR** - Workflow failed
- 🟡 **WARNING** - Missing dependencies

## Test Results

All workflows execute successfully:

```
Total workflows: 12
Passed: 12
Failed: 0
```

### Example Output

```
==================================================
[INFO] Testing: 01_basic_text_generation
==================================================
[INFO] Creating workflow: 01_basic_text_generation
[SUCCESS] Created workflow with ID: aa8e7f43-66c0-4f2a-92f8-2be5d3bac505
[INFO] Executing workflow: 01_basic_text_generation
[SUCCESS] Workflow executed successfully
  Steps executed: 4
  Execution time: 3639ms
  Output nodes: 1
    - output_1: {'result': 'Code dances in syntax...'}
```

## Sample Data

The test script uses sample input files located in `examples/sample-data/`:

| File | Used By | Purpose |
|------|---------|---------|
| `input.txt` | 02_file_io_workflow | General file input |
| `long_document.txt` | 09_document_chunking_and_summarization | Long text for chunking |
| `customer_feedback.txt` | 10_sentiment_analysis_routing | Sentiment analysis input |
| `phrases.txt` | 11_batch_translation_loop | Multi-line translation input |
| `draft_content.txt` | 12_content_quality_pipeline | Poor quality text for rewriting |

## Running Individual Tests

To test a specific workflow, modify the script or use curl:

```bash
# Create workflow
WORKFLOW_ID=$(curl -s -X POST \
  -H "Content-Type: application/json" \
  -d @examples/workflows/01_basic_text_generation.json \
  http://127.0.0.1:8080/workflow-ui/workflows \
  | jq -r '.id')

# Execute workflow
curl -s -X POST \
  http://127.0.0.1:8080/workflow-ui/workflows/${WORKFLOW_ID}/execute \
  | jq '.'

# Delete workflow
curl -s -X DELETE \
  http://127.0.0.1:8080/workflow-ui/workflows/${WORKFLOW_ID}
```

## Workflow IDs

Each example workflow includes a stable `workflow_id` field (UUID format):

```json
{
  "name": "Basic Text Generation",
  "workflow_id": "00000000-0000-0000-0000-000000000001",
  "nodes": [...],
  "connections": [...]
}
```

### Benefits
- **Deterministic IDs:** Example workflows have predictable IDs for easy reference
- **Direct execution:** Execute workflows by their ID without needing to create them first
- **Idempotent imports:** Re-importing a workflow with the same workflow_id updates the existing workflow

### Execute by workflow_id

If a workflow is already imported, you can execute it directly by its workflow_id:

```bash
# Example 1 has workflow_id: 00000000-0000-0000-0000-000000000001
curl -X POST \
  http://127.0.0.1:8080/workflow-ui/workflows/00000000-0000-0000-0000-000000000001/execute \
  -H "Content-Type: application/json" \
  -d '{}'
```

### Custom workflow_id

When creating your own workflows, you can specify a custom workflow_id:

```bash
curl -X POST http://127.0.0.1:8080/workflow-ui/workflows \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Custom Workflow",
    "description": "Test workflow",
    "workflow_id": "my-custom-uuid-12345",
    "nodes": [],
    "connections": []
  }'
```

If you don't provide a `workflow_id`, one will be automatically generated.

## API Endpoints Used

The test script interacts with these endpoints:

- `GET /workflow-ui/nodes` - Check daemon status
- `POST /workflow-ui/workflows` - Create workflow
- `POST /workflow-ui/workflows/:id/execute` - Execute workflow
- `DELETE /workflow-ui/workflows/:id` - Delete workflow

## Troubleshooting

### Daemon Not Running
```
[ERROR] Daemon is not running
[ERROR] Please start it with: cargo run --bin agency-daemon --release
```

**Solution:** Start the daemon in a separate terminal.

### Test Failures

If workflows fail, check:

1. **Ollama is running** (for LLM examples):
   ```bash
   ollama serve
   ollama pull qwen3-coder:480b-cloud
   ```

2. **Sample data exists**:
   ```bash
   ls -la examples/sample-data/
   ```

3. **File paths are correct** in workflow JSON files

4. **Daemon logs** for detailed error messages

### Python Version

The script requires Python 3.6+ and uses only standard library modules:
- `json`
- `sys`
- `time`
- `urllib.request`
- `urllib.error`
- `pathlib`
- `typing`

No external dependencies required!

## Continuous Integration

### GitHub Actions Example

```yaml
name: Test Workflows

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install Ollama
        run: |
          curl https://ollama.ai/install.sh | sh
          ollama serve &
          ollama pull qwen3-coder:480b-cloud

      - name: Build daemon
        run: cargo build --release --bin agency-daemon

      - name: Start daemon
        run: cargo run --bin agency-daemon --release &

      - name: Wait for daemon
        run: sleep 10

      - name: Run workflow tests
        run: python3 examples/test_workflows.py
```

## Performance Benchmarks

Average execution times from test run:

| Workflow | Steps | Time (ms) | Complexity |
|----------|-------|-----------|------------|
| 01_basic_text_generation | 4 | 3,639 | Simple |
| 02_file_io_workflow | 4 | 939 | Simple |
| 03_conditional_routing | 7 | 810 | Medium |
| 04_text_chunking | 3 | 1 | Simple |
| 05_multi_provider_comparison | 7 | 3,003 | Medium |
| 06_if_then_else_container | 7 | 1 | Medium |
| 07_foreach_loop_container | 5 | 1 | Medium |
| 08_while_loop_container | 5 | 0 | Medium |
| 09_document_chunking_and_summarization | 6 | 2 | Complex |
| 10_sentiment_analysis_routing | 10 | 1,411 | Complex |
| 11_batch_translation_loop | 8 | 3 | Complex |
| 12_content_quality_pipeline | 10 | 10,434 | Complex |

**Total:** 76 steps executed across 12 workflows in ~20 seconds

## Advanced Usage

### Custom Test Script

Create your own test script for specific needs:

```python
#!/usr/bin/env python3
import json
import urllib.request

def test_custom_workflow():
    # Load workflow
    with open('my_workflow.json', 'r') as f:
        workflow = json.load(f)

    # Create workflow
    data = json.dumps(workflow).encode('utf-8')
    req = urllib.request.Request(
        'http://127.0.0.1:8080/workflow-ui/workflows',
        data=data,
        headers={"Content-Type": "application/json"}
    )

    with urllib.request.urlopen(req) as response:
        result = json.loads(response.read())
        workflow_id = result['id']

    # Execute
    exec_req = urllib.request.Request(
        f'http://127.0.0.1:8080/workflow-ui/workflows/{workflow_id}/execute',
        method='POST'
    )

    with urllib.request.urlopen(exec_req) as response:
        result = json.loads(response.read())
        print(json.dumps(result, indent=2))

test_custom_workflow()
```

## Next Steps

1. **Create your own workflows** in the UI
2. **Export them** to JSON
3. **Add to examples/workflows/**
4. **Run tests** to validate

See `examples/workflows/README.md` for workflow creation guidelines.
