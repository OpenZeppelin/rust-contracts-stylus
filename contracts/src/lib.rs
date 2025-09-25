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
use openzeppelin_stylus::token::erc20::{self, Erc20, IErc20};
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
};

#[entrypoint]
#[storage]
struct MyContract {
    pub erc20: Erc20,
}

#[public]
#[implements(IErc20<Error = erc20::Error>)]
impl MyContract {}

#[public]
impl IErc20 for MyContract {
    type Error = erc20::Error;

    fn total_supply(&self) -> U256 {
        self.erc20.total_supply()
    }

    fn balance_of(&self, account: Address) -> U256 {
        self.erc20.balance_of(account)
    }

    fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.transfer(to, value)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.approve(spender, value)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.transfer_from(from, to, value)
    }
}
```
*/

#![allow(
    clippy::module_name_repetitions,
    clippy::used_underscore_items,
    clippy::unreadable_literal,
    deprecated
)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std, no_main)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![deny(rustdoc::broken_intra_doc_links)]
extern crate alloc;

pub mod access;
pub mod finance;
pub mod proxy;
pub mod token;
pub mod utils;
