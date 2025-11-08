#!/usr/bin/env python3
"""
Add workflow_id to all example workflows
"""

import json
import uuid
from pathlib import Path

# Stable UUIDs for examples (generated once and kept stable)
WORKFLOW_IDS = {
    "01_basic_text_generation.json": "00000000-0000-0000-0000-000000000001",
    "02_file_io_workflow.json": "00000000-0000-0000-0000-000000000002",
    "03_conditional_routing.json": "00000000-0000-0000-0000-000000000003",
    "04_text_chunking.json": "00000000-0000-0000-0000-000000000004",
    "05_multi_provider_comparison.json": "00000000-0000-0000-0000-000000000005",
    "06_if_then_else_container.json": "00000000-0000-0000-0000-000000000006",
    "07_foreach_loop_container.json": "00000000-0000-0000-0000-000000000007",
    "08_while_loop_container.json": "00000000-0000-0000-0000-000000000008",
    "09_document_chunking_and_summarization.json": "00000000-0000-0000-0000-000000000009",
    "10_sentiment_analysis_routing.json": "00000000-0000-0000-0000-000000000010",
    "11_batch_translation_loop.json": "00000000-0000-0000-0000-000000000011",
    "12_content_quality_pipeline.json": "00000000-0000-0000-0000-000000000012",
}

def add_workflow_id_to_file(filepath: Path):
    """Add workflow_id to a workflow JSON file"""
    filename = filepath.name

    # Load workflow
    with open(filepath, 'r') as f:
        workflow = json.load(f)

    # Add workflow_id
    if filename in WORKFLOW_IDS:
        workflow["workflow_id"] = WORKFLOW_IDS[filename]
    else:
        # Generate new UUID for files not in the list
        workflow["workflow_id"] = str(uuid.uuid4())

    # Write back
    with open(filepath, 'w') as f:
        json.dump(workflow, f, indent=2)

    print(f"Added workflow_id to {filename}: {workflow.get('workflow_id')}")

def main():
    workflows_dir = Path("examples/workflows")

    for workflow_file in sorted(workflows_dir.glob("*.json")):
        add_workflow_id_to_file(workflow_file)

    print(f"\nCompleted! Added workflow_id to all workflow files.")

if __name__ == "__main__":
    main()
