/*!
# OpenZeppelin Contracts for Stylus

A library for secure smart contract development written in Rust for
[Arbitrum Stylus](https://docs.arbitrum.io/stylus/gentle-introduction).
This library offers common smart contract primitives and affordances that take
advantage of the nature of Stylus.

## Usage

To start using it, add `openzeppelin-stylus` to your `Cargo.toml`, or simply run
`cargo add openzeppelin-stylus`.

```toml
[dependencies]
openzeppelin-stylus = "x.x.x"
```

We recommend pinning to a specific version -- expect rapid iteration.

Once defined as a dependency, use one of our pre-defined implementations by
importing them:

```ignore
use stylus_sdk::prelude::*;
use openzeppelin_stylus::token::erc20::Erc20;
use openzeppelin_stylus::token::erc20::utils::SafeErc20;

#[entrypoint]
#[storage]
struct MyContract {
    #[borrow]
    pub erc20: Erc20,
}

#[public]
#[inherit(Erc20)]
#[inherit(SafeErc20)]
impl MyContract { }
```

## SafeERC20

The library includes SafeERC20 functionality that provides wrappers around ERC-20 operations
that throw on failure (when the token contract returns false). Tokens that return no value
(and instead revert or throw on failure) are also supported, non-reverting calls are assumed
to be successful.

### Features

- Safe transfer operations that handle tokens that:
  - Return boolean values
  - Don't return values (revert on failure)
  - Always return false
  - Have non-standard approval behavior (like USDT)

- Safe approval operations that handle:
  - Standard ERC20 approval
  - USDT-style approval
  - Force approval for non-standard tokens

- ERC1363 support with:
  - `transfer_and_call`
  - `transfer_from_and_call`
  - `approve_and_call`

- Try variants for all operations that return a boolean instead of reverting

### Usage

To use SafeERC20, inherit from the `SafeErc20` trait in your contract implementation:

```ignore
#[inherit(SafeErc20)]
impl MyContract {
    // You can now use safe operations like:
    // - safe_transfer
    // - safe_transfer_from
    // - safe_increase_allowance
    // - safe_decrease_allowance
    // - force_approve
    // - transfer_and_call
    // - transfer_from_and_call
    // - approve_and_call
}
```

For more examples, see the `examples/safe-erc20` directory.
*/

#![allow(
    clippy::module_name_repetitions,
    clippy::used_underscore_items,
    deprecated
)]
#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![deny(rustdoc::broken_intra_doc_links)]
extern crate alloc;

pub mod access;
pub mod finance;
pub mod token;
pub mod utils;
