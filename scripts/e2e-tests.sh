#!/bin/bash
set -e

# Invoke the script by passing any non-zero number of arguments.
# The first argument is always the test project(s) to run and all the other arguments are valid `cargo test` arguments.
#
# The following are all valid ways to invoke this script:
# - ./scripts/e2e-tests.sh "*" <- invoke all tests
# - ./scripts/e2e-tests.sh erc20 <- invoke tests only for erc20
# - ./scripts/e2e-tests.sh erc20* <- invoke tests for all tests beginning with "erc20..."
# - ./scripts/e2e-tests.sh *erc20 <- invoke tests for all tests ending with "...erc20"
# - ./scripts/e2e-tests.sh "*" -- constructs <- invoke the "constructs" test in all test projects

export RPC_URL=http://localhost:8547
export DEPLOYER_ADDRESS=0x6ac4839Bfe169CadBBFbDE3f29bd8459037Bf64e

# Get the root directory of the git repository
ROOT_DIR=$(git rev-parse --show-toplevel)
cd "$ROOT_DIR" || exit

# Function to retrieve all Cargo.toml paths in the ./examples directory
get_example_dirs() {
    find ./examples -maxdepth 2 -type f -name "Cargo.toml" | xargs -n1 dirname | grep -v "erc721\|erc1155" | sort
}

run_test() {
    local project_path=$1
    shift
    local test_args=$@

    echo "Processing: $project_path"
    cd "$project_path"
    cargo test --features e2e $test_args
    cd "$ROOT_DIR"
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
    for CRATE_NAME in $(get_example_dirs); do
        run_test "$CRATE_NAME" "$@"
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
        run_test "$project_path" "$@"
    done
fi

echo "Build and test completed successfully."
