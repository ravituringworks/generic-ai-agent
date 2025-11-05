#!/usr/bin/env python3
"""Replace all employee names in examples with unique random IDs."""

import os
import re
from pathlib import Path

# Mapping of full names to unique IDs
# Must be processed in order (longest first) to avoid partial matches
NAME_TO_ID = {
    "Alice Chen": "EMP001",
    "Bob Martinez": "EMP002",
    "Carol Kim": "EMP003", 
    "David Johnson": "EMP004",
    "Emily Zhang": "EMP005",
    "Frank Wilson": "EMP006",
    "Grace Lee": "EMP007",
    "Henry Patel": "EMP008",
    "Iris Anderson": "EMP009",
    "Jack Thompson": "EMP010",
    "Kate Brown": "EMP011",
    "Leo Garcia": "EMP012",
    "Maya Nguyen": "EMP013",
    "Noah Davis": "EMP014",
    "Olivia Torres": "EMP015",
    "Paul Chen": "EMP016",
    "Quinn Rivera": "EMP017",
    "Rachel Kim": "EMP018",
    "Sam Johnson": "EMP019",
    "Tina Martinez": "EMP020",
    "Uma Patel": "EMP021",
    "Victor Wong": "EMP022",
    "Wendy Anderson": "EMP023",
    "Xavier Lopez": "EMP024",
    "Yara Hassan": "EMP025",
    "Zack Thompson": "EMP026",
    "Dr. Sarah Chen": "EMP027",
    "Dr. James Park": "EMP028",
    "Dr. Lisa Wang": "EMP029",
    "Marcus Johnson": "EMP030",
    "John Doe": "EMP031",  # Generic example name
}

# Additional patterns for context-specific replacements
PATTERN_REPLACEMENTS = [
    (r"SimulationEngineer_Alice", "SimulationEngineer_EMP001"),
    (r"ScalingEngineer_Bob", "ScalingEngineer_EMP002"), 
    (r"Coordinator_Charlie", "Coordinator_EMP003"),
    # Standalone names - be careful about context
    (r"\bAlice\b(?! CPU)(?! Grace)", "EMP001"),  # Don't replace "Grace CPU"
    (r"\bBob\b(?! CPU)(?! Grace)", "EMP002"),
    (r"\bCharlie\b(?! CPU)(?! Grace)", "EMP003"),
    (r"\bCarol\b(?! CPU)(?! Grace)", "EMP003"),
    (r"\bDavid\b(?! CPU)(?! Grace)", "EMP004"),
    (r"\bEmily\b(?! CPU)(?! Grace)", "EMP005"),
    (r"\bFrank\b(?! CPU)(?! Grace)", "EMP006"),
    # Only replace Grace when it's clearly a person name
    (r"\bGrace(?!\s+(CPU|chip|processor))\b", "EMP007"),
    (r"\bHenry\b(?! CPU)(?! Grace)", "EMP008"),
    (r"\bIris\b(?! CPU)(?! Grace)", "EMP009"),
    (r"\bJack\b(?! CPU)(?! Grace)", "EMP010"),
    (r"\bKate\b(?! CPU)(?! Grace)", "EMP011"),
    (r"\bLeo\b(?! CPU)(?! Grace)", "EMP012"),
    (r"\bMaya\b(?! CPU)(?! Grace)", "EMP013"),
    (r"\bNoah\b(?! CPU)(?! Grace)", "EMP014"),
    (r"\bOlivia\b(?! CPU)(?! Grace)", "EMP015"),
    (r"\bPaul\b(?! CPU)(?! Grace)", "EMP016"),
    (r"\bQuinn\b(?! CPU)(?! Grace)", "EMP017"),
    (r"\bRachel\b(?! CPU)(?! Grace)", "EMP018"),
    (r"\bSam\b(?! CPU)(?! Grace)", "EMP019"),
    (r"\bTina\b(?! CPU)(?! Grace)", "EMP020"),
    (r"\bUma\b(?! CPU)(?! Grace)", "EMP021"),
    (r"\bVictor\b(?! CPU)(?! Grace)", "EMP022"),
    (r"\bWendy\b(?! CPU)(?! Grace)", "EMP023"),
    (r"\bXavier\b(?! CPU)(?! Grace)", "EMP024"),
    (r"\bYara\b(?! CPU)(?! Grace)", "EMP025"),
    (r"\bZack\b(?! CPU)(?! Grace)", "EMP026"),
]

def replace_names_in_file(filepath):
    """Replace all employee names in a file with their IDs."""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # First replace full names (order matters - longest first)
        for name, emp_id in NAME_TO_ID.items():
            content = content.replace(name, emp_id)
        
        # Then apply pattern-based replacements
        for pattern, replacement in PATTERN_REPLACEMENTS:
            content = re.sub(pattern, replacement, content)
        
        # Only write if changes were made
        if content != original_content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✓ Updated: {filepath}")
            return True
        return False
    except Exception as e:
        print(f"✗ Error processing {filepath}: {e}")
        return False

def main():
    """Main function to process all files."""
    examples_dir = Path("/Users/ravindraboddipalli/sources/the-agency/examples")
    
    if not examples_dir.exists():
        print(f"Error: Directory not found: {examples_dir}")
        return
    
    # File patterns to process
    patterns = ["*.rs", "*.md", "*.toml"]
    
    files_updated = 0
    files_processed = 0
    
    print("🔄 Replacing employee names with unique IDs...\n")
    
    for pattern in patterns:
        for filepath in examples_dir.rglob(pattern):
            files_processed += 1
            if replace_names_in_file(filepath):
                files_updated += 1
    
    print(f"\n✅ Complete!")
    print(f"   Files processed: {files_processed}")
    print(f"   Files updated: {files_updated}")

if __name__ == "__main__":
    main()
