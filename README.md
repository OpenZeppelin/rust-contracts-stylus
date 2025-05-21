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
line to your `Cargo.toml` (We recommend pinning to a specific version):

```toml
[dependencies]
openzeppelin-stylus = "=0.1.2"
```

If you want to use some of our newest features before they are fully stable or audited, you can try the latest alpha version of the library. We release a new alpha version every ~3 weeks.

```toml
[dependencies]
openzeppelin-stylus = "=0.2.0-alpha.4"
```

We put great effort in testing the contracts before releasing an alpha, but these are not yet audited and we don't guarantee any backwards compatibility between alpha version.

> [!NOTE]
> This library is designed to be `no_std`, which helps reduce wasm size. If you want your project to be `no_std` as well, ensure that your dependencies are not importing the standard library.
> You can achieve this by setting `default-features = false` for relevant dependencies in your `Cargo.toml`. For example:
>
> ```toml
> [dependencies]
> alloy-primitives = { version = "=0.8.20", default-features = false }
> stylus-sdk = { version = "=0.9.0", default-features = false, features = [
>   "mini-alloc",
> ] }
> ```

Once defined as a dependency, use one of our pre-defined implementations by
importing them:

```rust
use stylus_sdk::prelude::*;
use openzeppelin_stylus::token::erc20::Erc20;

#[entrypoint]
#[storage]
struct Erc20Example {
    #[borrow]
    erc20: Erc20,
}

#[public]
#[inherit(Erc20)]
impl Erc20Example {}
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
