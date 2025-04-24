#!/bin/bash
set -e

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

manifest_path="$1"
test_arg="$2"
    
# Main logic based on number of arguments
case $# in
    1)
        # If one argument is passed, process all examples
        for CRATE_NAME in $(get_example_manifest_paths)
        do
            cd "$CRATE_NAME"
            build_and_test "$1"
            cd "$mydir"
        done
        ;;
    2)
        # If two arguments are passed, process only the specified manifest
        build_and_test "$2" "$1"
        ;;
    *)
        echo "Usage: $0 <test_arg> [<manifest_path>]"
        echo "  One argument: Run with all examples in ./examples directory"
        echo "  Two arguments: Run with specific manifest path"
        exit 1
        ;;
esac

# TODO: FINISH