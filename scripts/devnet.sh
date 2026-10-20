#!/bin/bash
# Manage the local Stylus devnet used by the end-to-end tests and the benches.
#
# The devnet is a nitro dev node started by the `devnet` binary of the `e2e`
# crate with the Stylus SDK's devnet tooling (`stylus_tools::devnet::Node`).
# It runs in Docker on a random port; the RPC URL is written to
# `target/devnet/rpc`, where the tests and benches pick it up.
#
# Usage:
#   ./scripts/devnet.sh up      start the devnet and wait until it is ready
#   ./scripts/devnet.sh down    stop the devnet
#   ./scripts/devnet.sh status  print whether it is running and its RPC URL
set -euo pipefail

cd "$(dirname "$(realpath "$0")")/.."

TARGET_DIR="${CARGO_TARGET_DIR:-target}"
DEVNET_DIR="$TARGET_DIR/devnet"
RPC_FILE="$DEVNET_DIR/rpc"
LOG_FILE="$DEVNET_DIR/devnet.log"

case "${1:-}" in
  up)
    cargo build -q -p e2e --bin devnet
    mkdir -p "$DEVNET_DIR"
    if "$TARGET_DIR/debug/devnet" status | grep -q '^running'; then
      "$TARGET_DIR/debug/devnet" status
      exit 0
    fi
    rm -f "$RPC_FILE"
    setsid nohup "$TARGET_DIR/debug/devnet" up > "$LOG_FILE" 2>&1 &
    DAEMON_PID=$!
    for _ in $(seq 1 600); do
      if [ -s "$RPC_FILE" ]; then
        echo "devnet ready at $(cat "$RPC_FILE")"
        exit 0
      fi
      if ! kill -0 "$DAEMON_PID" 2>/dev/null; then
        echo "the devnet daemon exited:" >&2
        cat "$LOG_FILE" >&2
        exit 1
      fi
      sleep 1
    done
    echo "timed out waiting for the devnet:" >&2
    cat "$LOG_FILE" >&2
    exit 1
    ;;
  down)
    if [ -x "$TARGET_DIR/debug/devnet" ]; then
      "$TARGET_DIR/debug/devnet" down
    else
      echo "no devnet daemon is running"
    fi
    ;;
  status)
    cargo build -q -p e2e --bin devnet
    "$TARGET_DIR/debug/devnet" status
    ;;
  *)
    echo "usage: $0 up | down | status" >&2
    exit 2
    ;;
esac
