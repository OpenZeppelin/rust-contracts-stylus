# OpenZeppelin Contracts for Stylus

**A library for secure smart contract development** written in Rust for
[Arbitrum Stylus](https://docs.arbitrum.io/stylus/gentle-introduction).

## Features

- Security-first smart contracts, ported from the [`openzeppelin-contracts`]
  library.
- First-class `no_std` support.
- [Unit] and [integration] test affordances, used in our own tests.

[`openzeppelin-contracts`]: https://github.com/OpenZeppelin/openzeppelin-contracts
[Unit]: https://github.com/OpenZeppelin/stylus-test-helpers
[integration]: ./lib/e2e/README.md

## Usage

You can import OpenZeppelin Contracts from crates.io by adding the following
dependency declarations to your `Cargo.toml` (we recommend pinning to a specific version):

```toml
[dependencies]
openzeppelin-stylus = "=0.2.0"

[dev-dependencies]
openzeppelin-stylus = { version = "=0.2.0", features = ["stylus-test"] }
```

Note that you need to enable the `stylus-test` feature flag in development and test environments.

You should also enable `openzeppelin-stylus/export-abi` in the `export-abi` feature declaration to enable export ABI functionality:

```toml
[features]
# we can omit `stylus-sdk/export-abi` as it will be activated
# by the `openzeppelin-stylus/export-abi` feature.
export-abi = ["openzeppelin-stylus/export-abi"]
```

We put great effort in testing the contracts before releasing an alpha, but these are not yet audited and we don't guarantee any backwards compatibility between alpha version.

> [!NOTE]
> This library is designed to be `no_std`, which helps reduce wasm size. If you want your project to be `no_std` as well, ensure that your dependencies are not importing the standard library.
> You can achieve this by setting `default-features = false` for relevant dependencies in your `Cargo.toml`. For example:
>
> ```toml
> [dependencies]
> alloy-primitives = { version = "=0.8.20", default-features = false }
> stylus-sdk = "=0.9.0"
> ```

Once defined as a dependency, use one of our pre-defined implementations by
importing them:

```rust
use openzeppelin_stylus::token::erc20::{self, Erc20, IErc20};
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
};

#[entrypoint]
#[storage]
struct Erc20Example {
    erc20: Erc20,
}

#[public]
#[implements(IErc20<Error = erc20::Error>)]
impl Erc20Example {}

#[public]
impl IErc20 for Erc20Example {
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

For a more complex display of what this library offers, refer to our
[examples](./examples).

For a full example that includes deploying and querying a contract, see the
[basic] example.

For more information on what this library will include in the future, see our
[roadmap].

[basic]: ./examples/basic
[roadmap]: https://github.com/orgs/OpenZeppelin/projects/35/views/8

## Contribute

OpenZeppelin Contracts for Stylus exists thanks to its contributors. There are
many ways you can participate and help build high-quality software. Check out
the [contribution guide](CONTRIBUTING.md)!

## Security

Past audits can be found in [`audits/`](./audits).

Refer to our [Security Policy](SECURITY.md) for more details.

## License

OpenZeppelin Contracts for Stylus is released under
the [MIT License](./LICENSE).
