# OpenZeppelin Contracts for Stylus

**A library for secure smart contract development** written in Rust for
[Arbitrum Stylus](https://docs.arbitrum.io/stylus/stylus-gentle-introduction).

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

You can import OpenZeppelin Contracts from crates.io by adding the following
line to your `Cargo.toml` (We recommend pinning to a specific version):

```toml
[dependencies]
openzeppelin-stylus = "0.1.0"
```

Optionally, you can specify a git dependency if you want to have the latest
changes from the `main` branch:

```toml
[dependencies]
openzeppelin-stylus = { git = "https://github.com/OpenZeppelin/rust-contracts-stylus" }
```

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

#[public]
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
[roadmap]: https://github.com/OpenZeppelin/rust-contracts-stylus/milestone/2

## Contribute

OpenZeppelin Contracts for Stylus exists thanks to its contributors. There are
many ways you can participate and help build high-quality software. Check out
the [contribution guide](CONTRIBUTING.md)!

## Security

Past audits can be found in [`audits/`](./audits).

Refer to our [Security Policy](SECURITY.md) for more details.

## License

OpenZeppelin Contracts for Stylus is released under the [MIT License](./LICENSE).
