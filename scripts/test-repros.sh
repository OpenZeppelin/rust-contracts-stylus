#!/bin/bash
set -e

# Retrieve all alphanumeric contract's crate names in `./repros` directory.
# NOTE: optimistically relying on the 'name = ' string at Cargo.toml file
repros=$(find ./repros -maxdepth 2 -type f -name "Cargo.toml" | xargs grep 'name = ' | grep -oE '".*"' | tr -d "'\"")

pkg_args=()
for CRATE_NAME in $repros; do
    pkg_args+="-p $CRATE_NAME "
done

cargo nextest run --locked --all-targets ${pkg_args[*]}
