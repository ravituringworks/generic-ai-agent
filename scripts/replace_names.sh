#!/bin/bash
# Replace all employee names with unique IDs

# Navigate to the examples directory
cd "/Users/ravindraboddipalli/sources/the-agency/examples" || exit 1

# Full name mappings (must be processed first to avoid partial matches)
declare -A FULL_NAMES=(
    ["Alice Chen"]="EMP001"
    ["Bob Martinez"]="EMP002"
    ["Carol Kim"]="EMP003"
    ["David Johnson"]="EMP004"
    ["Emily Zhang"]="EMP005"
    ["Frank Wilson"]="EMP006"
    ["Grace Lee"]="EMP007"
    ["Henry Patel"]="EMP008"
    ["Iris Anderson"]="EMP009"
    ["Jack Thompson"]="EMP010"
    ["Kate Brown"]="EMP011"
    ["Leo Garcia"]="EMP012"
    ["Maya Nguyen"]="EMP013"
    ["Noah Davis"]="EMP014"
    ["Olivia Torres"]="EMP015"
    ["Paul Chen"]="EMP016"
    ["Quinn Rivera"]="EMP017"
    ["Rachel Kim"]="EMP018"
    ["Sam Johnson"]="EMP019"
    ["Tina Martinez"]="EMP020"
    ["Uma Patel"]="EMP021"
    ["Victor Wong"]="EMP022"
    ["Wendy Anderson"]="EMP023"
    ["Xavier Lopez"]="EMP024"
    ["Yara Hassan"]="EMP025"
    ["Zack Thompson"]="EMP026"
)

# First, replace all full names (exact matches)
for name in "${!FULL_NAMES[@]}"; do
    id="${FULL_NAMES[$name]}"
    echo "Replacing '$name' with '$id'..."
    
    # Replace in Rust files
    find . -name "*.rs" -type f -exec sed -i '' "s/$name/$id/g" {} \;
    
    # Replace in Markdown files  
    find . -name "*.md" -type f -exec sed -i '' "s/$name/$id/g" {} \;
    
    # Replace in TOML files
    find . -name "*.toml" -type f -exec sed -i '' "s/$name/$id/g" {} \;
done

echo "✅ All employee names replaced with unique IDs!"
