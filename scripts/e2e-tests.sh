#!/bin/bash
set -e

# Default test file or wildcard
TEST_ARG="*"
CARGO_TEST_FLAGS=""
TEST_BINARY_ARGS=""

# Parse arguments
for arg in "$@"; do
    if [[ "$arg" == "--" ]]; then
        # Everything after `--` goes to the test binary
        shift
        TEST_BINARY_ARGS="$@"
        break
    elif [[ "$arg" == -* ]]; then
        # Capture cargo test flags (e.g., --exclude, --no-run)
        CARGO_TEST_FLAGS="$CARGO_TEST_FLAGS $arg"
    else
        # Set the test file (first positional argument)
        TEST_ARG="$arg"
    fi
done

# Move to project root
cd "$(dirname "$(realpath "$0")")/.."

# Build the project
cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort

# Set RPC_URL environment variable
export RPC_URL=http://localhost:8547

# Run the tests with cargo test flags and test binary arguments
cargo test --features std,e2e --test "$TEST_ARG" $CARGO_TEST_FLAGS -- $TEST_BINARY_ARGS
