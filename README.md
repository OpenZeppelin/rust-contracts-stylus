# OpenZeppelin Contracts for Stylus

**A library for secure smart contract development** written in Rust for
[Arbitrum Stylus](https://docs.arbitrum.io/stylus/stylus-gentle-introduction).

> [!WARNING]
> This project is still in a very early and experimental phase. It has never
> been audited nor thoroughly reviewed for security vulnerabilities. Do not use
> in production.

## Features

- Security-first smart contracts, ported from the [`openzeppelin-contracts`]
  library.
- First-class `no_std` support.
- Solidity constructors powered by [`koba`].
- [Unit] and [integration] test affordances, used in our own tests.

[`openzeppelin-contracts`]: https://github.com/OpenZeppelin/openzeppelin-contracts
[`koba`]: https://github.com/OpenZeppelin/koba
[Unit]: ./lib/motsu/README.md
[integration]: ./lib/e2e/README.md

## Usage

The library has not been published yet to `crates.io`, and this will be the case
until we reach a stable version. However, one can [specify a git dependency] in
a `Cargo.toml`, like so:

```toml
[dependencies]
openzeppelin-stylus = { git = "https://github.com/OpenZeppelin/rust-contracts-stylus" }
```

We recommend pinning to a specific version -- expect rapid iteration.

Once defined as a dependency, use one of our pre-defined implementations by
importing them:

```rust
use openzeppelin_stylus::token::erc20::Erc20;

sol_storage! {
    #[entrypoint]
    struct Erc20Example {
        #[borrow]
        Erc20 erc20;
    }
}

#[external]
#[inherit(Erc20)]
impl Erc20Example { }
```

For a more complex display of what this library offers, refer to our
[examples](./examples).

For a full example that includes deploying and querying a contract, see the
[basic] example.

For more information on what this library will include in the future, see our
[roadmap].

[specify a git dependency]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories
[examples]: ./examples
[basic]: ./examples/basic
[roadmap]: https://github.com/OpenZeppelin/rust-contracts-stylus/milestone/1

## Contribute

OpenZeppelin Contracts for Stylus exists thanks to its contributors. There are
many ways you can participate and help build high-quality software. Check out
the [contribution guide](CONTRIBUTING.md)!

## Security

> [!WARNING]
> This project is still in a very early and experimental phase. It has never
> been audited nor thoroughly reviewed for security vulnerabilities. Do not use
> in production.

Refer to our [Security Policy](SECURITY.md) for more details.

## License

OpenZeppelin Contracts for Stylus is released under the [MIT License](./LICENSE).
