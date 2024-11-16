/*!
# OpenZeppelin Contracts for Stylus

A library for secure smart contract development written in Rust for
[Arbitrum Stylus](https://docs.arbitrum.io/stylus/stylus-gentle-introduction).
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

sol_storage! {
    #[entrypoint]
    struct MyContract {
        #[borrow]
        Erc20 erc20;
    }
}

#[public]
#[inherit(Erc20)]
impl MyContract { }
```
*/

#![allow(
    clippy::pub_underscore_fields,
    clippy::module_name_repetitions,
    clippy::used_underscore_items
)]
#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![deny(rustdoc::broken_intra_doc_links)]
extern crate alloc;

pub mod access;
pub mod token;
pub mod utils;
