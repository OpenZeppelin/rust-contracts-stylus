#!/bin/bash
set -e

export RPC_URL=http://localhost:8547
export DEPLOYER_ADDRESS=0x6ac4839Bfe169CadBBFbDE3f29bd8459037Bf64e

# Get the root directory of the git repository
mydir=$(git rev-parse --show-toplevel)
cd "$mydir" || exit

# Function to retrieve all Cargo.toml paths in the ./examples directory
get_example_manifest_paths() {
    find ./examples -maxdepth 2 -type f -name "Cargo.toml" | xargs -n1 dirname
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
    # Process only the specified project
    project_path="./examples/$project_arg"
    
    if [ ! -d "$project_path" ]; then
        echo "Error: Project '$project_arg' not found in examples/"
        exit 1
    fi
    
    if [ ! -f "$project_path/Cargo.toml" ]; then
        echo "Error: No Cargo.toml found in $project_path"
        exit 1
    fi
    
    echo "Processing: $project_path"
    cd "$project_path"
    build_and_test "$@"
    cd "$mydir"
fi

echo "Build and test completed successfully."
