# Basic Token Example

This example showcases an end-to-end user journey of how to write, deploy and
query a smart contract written using our library.

## Running the example

There are two crates in the example: a library crate for our contract
implementation, which is a simple `ERC-20` token extended with `Metadata`, and
a binary crate, which holds the deployment script.

Before running the example, set the `PRIVATE_KEY` const variable and compile
your contract with:

```bash
cargo build --release --target wasm32-unknown-unknown
```

You should now be able to run your contract with:

```bash
$ cargo run -p basic-script-example
wasm data fee: Ξ0.000097
init code size: 17.0 KB
deploying to RPC: https://sepolia-rollup.arbitrum.io/rpc
deployed code: 0x7E57f52Bb61174DCE87deB10c4B683a550b39e8F
deployment tx hash: 0xc627970c65caa65b3b87703ecf2d26d83520c3ac570d76c72f2d0a04b9895d91
activating contract: 0x7E57f52Bb61174DCE87deB10c4B683a550b39e8F
activated with 2651090 gas
activation tx hash: 0x88c992c4c6e36fd2f49f2b30ea12412a2d5436bbfe80df8d10606abdb1f3f39d
```

Note that the script asserts that the deployed contract has the correct name and
symbol.

## Why two crates?

This split is necessary because we need to compile the contract to `wasm`,
however, the script depends on `alloy`, which in turn depends on `getrandom`
which is not compatible with wasm targets.
