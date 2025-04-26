#!/bin/bash
set -e

export RPC_URL=http://localhost:8547
export DEPLOYER_ADDRESS=0x6ac4839Bfe169CadBBFbDE3f29bd8459037Bf64e

# Get the root directory of the git repository
mydir=$(git rev-parse --show-toplevel)
cd "$mydir" || exit

# Function to retrieve all Cargo.toml paths in the ./examples directory
get_example_manifest_paths() {
    find ./examples -maxdepth 2 -type f -name "Cargo.toml" | xargs -n1 dirname | sort
}

# Function to build and test a crate
build_and_test() {    
    cargo build --release --target wasm32-unknown-unknown \
        -Z build-std=std,panic_abort \
        -Z build-std-features=panic_immediate_abort
    
    cargo test --features e2e "$@"
}

# Check for at least one argument
if [ $# -lt 1 ]; then
    echo "Usage: $0 <project_name|*> [test_args...]"
    echo "  project_name: Name of specific project under examples/"
    echo "  *: Run for all examples"
    echo "  test_args: Additional arguments passed to 'cargo test --features e2e'"
    exit 1
fi

# Get the first argument and remove it from the argument list
project_arg="$1"
shift

# Main logic based on first argument
if [ "$project_arg" = "*" ]; then
    # Process all examples
    for CRATE_NAME in $(get_example_manifest_paths)
    do
        echo "Processing: $CRATE_NAME"
        cd "$CRATE_NAME"
        build_and_test "$@"
        cd "$mydir"
    done
else
    # Find matching projects based on pattern
    matching_projects=()
    for dir in ./examples/*; do
        if [ -d "$dir" ] && [ -f "$dir/Cargo.toml" ]; then
            project_name=$(basename "$dir")
            
            # Handle different pattern types
            if [[ "$project_arg" == *"*"* ]]; then
                # Contains wildcard - use pattern matching
                # Replace * with regex pattern for =~ operator
                pattern="${project_arg//\*/.*}"
                [[ "$project_name" =~ ^${pattern}$ ]] && matching_projects+=("$dir")
            else
                # No wildcard - exact match only
                [[ "$project_name" == "$project_arg" ]] && matching_projects+=("$dir")
            fi
        fi
    done
    
    # Check if we found any matches
    if [ ${#matching_projects[@]} -eq 0 ]; then
        echo "Error: No projects found matching '$project_arg' in examples/"
        exit 1
    fi
    
    # Process all matching projects
    echo "Found ${#matching_projects[@]} matching project(s):"
    for project in "${matching_projects[@]}"; do
        echo "  - $(basename "$project")"
    done
    echo ""
    
    for project_path in "${matching_projects[@]}"; do
        echo "Processing: $project_path"
        cd "$project_path"
        build_and_test "$@"
        cd "$mydir"
    done
fi

echo "Build and test completed successfully."
