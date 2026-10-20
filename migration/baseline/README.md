# Baseline measurements before the Stylus SDK 0.10.10 migration

Measured on commit `835b8833` (`main`, 29 commits after the `v0.3.0` tag; the
last release plus ERC-6909 and dependency bumps), with the toolchain pinned by
`rust-toolchain.toml` at that commit: `nightly-2025-08-01`
(`rustc 1.90.0-nightly (adcb3d3b4 2025-07-31)`). Stylus SDK 0.9.0, motsu
0.10.0, cargo-stylus 0.10.10 (the version installed on the machine; CI pinned
0.6.3, which is not installed).

## Unit-test coverage (`coverage.txt`)

Command of the `coverage` job in `.github/workflows/test.yml`:

```bash
cargo llvm-cov --locked --lcov --output-path lcov.info \
  -p openzeppelin-stylus -p openzeppelin-stylus-proc -p openzeppelin-crypto
cargo llvm-cov report
```

| Metric    | Covered / total | Percent |
| --------- | --------------- | ------- |
| Regions   | 24839 / 26079   | 95.25%  |
| Functions | 1492 / 1649     | 90.48%  |
| Lines     | 16056 / 16852   | 95.28%  |

## Unit tests (`unit-tests.txt`)

Command of the `required` job in `test.yml`:

```bash
cargo nextest run --locked --all-targets \
  -p openzeppelin-stylus -p openzeppelin-stylus-proc -p openzeppelin-crypto \
  --features ruint
```

910 tests passed, 15 skipped (ignored). 635 of the tests are `#[motsu::test]`
functions.

## End-to-end tests (`e2e-tests.txt`)

513 `#[e2e::test]` functions in 39 files under `examples/*/tests`, listed as
`file::test_name`. The list is the inventory used to check that every scenario
still exists after the migration.

## WASM sizes (`wasm-sizes.txt`, `wasm-sha256.txt`)

Built with the command of `scripts/check-wasm.sh`:

```bash
cargo build --release --target wasm32-unknown-unknown \
  -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort
```

`wasm_bytes` is the size of the file in `target/wasm32-unknown-unknown/release`.
`compressed_b` is the Brotli-compressed contract size that counts against the
24 KiB (24576 bytes) limit, as printed by `cargo stylus check` against a local
nitro devnet (nitro-node v3.11.1, ArbOS 60). `check` is the exit code of
`cargo stylus check`; every contract passed. `cargo stylus check` 0.10.10
needs a `Stylus.toml` in the Cargo project, so an empty `[workspace]`
`Stylus.toml` was placed at the repository root for the run and removed again.

The SHA-256 of every binary is recorded so that the same binaries can be
deployed next to the migrated ones for a same-node gas comparison.

## Gas (`gas.md`)

Run with `scripts/bench.sh` against the nitro-testnode pinned by
`scripts/nitro-testnode.sh` (commit `947cb198`, nitro-node v3.7.4, ArbOS 40):
L2 gas used per call, for the plain, cached, and `wasm-opt`-ed-and-cached
deployment of each example.

The table covers the 10 benchmarks whose contract has no constructor. The other
4 benchmarks (`access_control`, `erc20`, `ownable`, `erc1155_metadata_uri`)
deploy through `cargo stylus deploy --wasm-file`, and cargo-stylus 0.10.10
ignores `--constructor-signature` and `--constructor-args` on that path (the
constructor is never run), so their calls revert. The before/after comparison
for all 14 benchmarks is done on the migrated harness by deploying the recorded
baseline binaries next to the new ones on the same devnet.
