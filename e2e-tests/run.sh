#!/bin/bash
set -e

# make sure we will be running script from the project root.
mydir=$(dirname "$0")
cd "$mydir" || exit
cd ..

export ALICE_PRIV_KEY=${ALICE_PRIV_KEY:-5744b91fe94e38f7cde31b0cc83e7fa1f45e31c053d015b9fb8c9ab3298f8a2d}
export BOB_PRIV_KEY=${BOB_PRIV_KEY:-a038232e463efa8ad57de6f88cd3c68ed64d1981daff2dcc015bce7eaf53db9d}
export RPC_URL=${RPC_URL:-http://localhost:8547}
NIGHTLY_TOOLCHAIN=${NIGHTLY_TOOLCHAIN:-nightly}

cargo +"$NIGHTLY_TOOLCHAIN" build --release --target wasm32-unknown-unknown

# TODO: run tests in parallel when concurrency scope will be per test/contract
cargo +stable test --features std,e2e -- --nocapture
