#!/bin/bash
set -e

MYDIR=$(realpath "$(dirname "$0")")
cd "$MYDIR"
cd ..

NIGHTLY_TOOLCHAIN=${NIGHTLY_TOOLCHAIN:-nightly-2024-01-01}
cargo +"$NIGHTLY_TOOLCHAIN" build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort

export RPC_URL=http://localhost:8547

# No need to compile benchmarks with `--release`
# since gas measurement happens on the contract side
cargo run -p benches

echo "Finished running benches!"
