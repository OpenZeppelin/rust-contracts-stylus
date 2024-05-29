#!/bin/bash
set -e

MYDIR=$(realpath "$(dirname "$0")")
cd "$MYDIR"
cd ..

NIGHTLY_TOOLCHAIN=${NIGHTLY_TOOLCHAIN:-nightly}
cargo +"$NIGHTLY_TOOLCHAIN" build --release --target wasm32-unknown-unknown

export RPC_URL=http://localhost:8547
cargo +stable test --features std,e2e -- --nocapture
