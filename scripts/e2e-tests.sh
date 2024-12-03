#!/bin/bash
set -e

MYDIR=$(realpath "$(dirname "$0")")
cd "$MYDIR"
cd ..

cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort

export RPC_URL=http://localhost:8547

cargo test --features std,e2e --test "erc1155"
