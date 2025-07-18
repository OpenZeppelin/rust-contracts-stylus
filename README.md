# OpenZeppelin Contracts for Stylus

**A secure, modular smart contract library** for [Arbitrum Stylus](https://docs.arbitrum.io/stylus/gentle-introduction), written in Rust and inspired by [OpenZeppelin Contracts](https://github.com/OpenZeppelin/openzeppelin-contracts).

Stylus enables high-performance smart contracts in Rust, compiled to WebAssembly (Wasm), for deployment on Arbitrum chains.

## ✨ Features

- Security-first contracts, ported from the [`openzeppelin-contracts`] library.
- Full support for `no_std` Rust environments.
- Ready-to-use [unit] and [integration] testing helpers, used in our own tests.
- Familiar, well-documented contract APIs.

[`openzeppelin-contracts`]: https://github.com/OpenZeppelin/openzeppelin-contracts
[unit]: https://github.com/OpenZeppelin/stylus-test-helpers
[integration]: ./lib/e2e/README.md

## 🚀 Usage

Add the crate to your Cargo.toml:

```toml
[dependencies]
# We recommend pinning to a specific version.
openzeppelin-stylus = "=0.2.0"
```

If you want to use the latest features before they are fully stabilized or audited, try the most recent alpha. We release a new alpha version every ~3 weeks.

```toml
[dependencies]
openzeppelin-stylus = "=0.3.0-alpha.1"
```

**Enable ABI export support**:

```toml
[features]
# stylus-sdk/export-abi will be enabled automatically.
export-abi = ["openzeppelin-stylus/export-abi"]
```

## 🧱 `no_std` Projects

This library is designed for `no_std` environments to reduce Wasm size.
Ensure your dependencies also avoid the standard library:

> ```toml
> [dependencies]
> alloy-primitives = { version = "=0.8.20", default-features = false }
> stylus-sdk = "=0.9.0"
> ```

## 🦀 Rust Nightly & WASM Builds

This project requires the **Rust nightly toolchain**, which is already pinned via [`rust-toolchain.toml`](./rust-toolchain.toml).

We also use a [`config.toml`](./.cargo/config.toml) to define platform-specific compiler flags.

To compile contracts for Arbitrum Stylus, run:

```sh
cargo build --release --target wasm32-unknown-unknown \
  -Z build-std=std,panic_abort \
  -Z build-std-features=panic_immediate_abort
```

## 🧪 Example: ERC20 Token

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

For a working demo with deployment and queries, check out the [basic] example.

## 📚 Resources

- **Examples**: Browse our [examples] folder for full project templates.
- **Test helpers**: See [unit] and [integration] testing utilities.
- **Roadmap**: Follow planned features and modules in our [roadmap].

[basic]: ./examples/basic
[examples]: ./examples
[roadmap]: https://github.com/orgs/OpenZeppelin/projects/35/views/9

## 🤝 Contribute

OpenZeppelin Contracts for Stylus exists thanks to its contributors. There are
many ways you can participate and help build high-quality software. Check out
the [contribution guide](CONTRIBUTING.md)!

## 🔐 Security

- **Past audits**: See the [`audits`](./audits) folder.
- Refer to our [Security Policy](SECURITY.md) for guidelines on reporting vulnerabilities.

## ⚖️ License

OpenZeppelin Contracts for Stylus is released under
the [MIT License](./LICENSE).
