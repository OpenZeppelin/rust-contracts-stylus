#!/bin/bash
set -e

MYDIR=$(realpath "$(dirname "$0")")
cd "$MYDIR"
cd ..

cargo build --release --target wasm32-unknown-unknown

export RPC_URL=http://localhost:8547
# We should use stable here once nitro-testnode is updated and the contracts fit
# the size limit. Work tracked [here](https://github.com/OpenZeppelin/rust-contracts-stylus/issues/87)
cargo test --features std,e2e --test "*"
