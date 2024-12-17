# OpenZeppelin Contracts for Stylus

**A library for secure smart contract development** written in Rust for
[Arbitrum Stylus](https://docs.arbitrum.io/stylus/gentle-introduction).

## Features

- Security-first smart contracts, ported from the [`openzeppelin-contracts`]
  library.
- First-class `no_std` support.
- Solidity constructors powered by [`koba`].
- [Unit] and [integration] test affordances, used in our own tests.

[`openzeppelin-contracts`]: https://github.com/OpenZeppelin/openzeppelin-contracts

[`koba`]: https://github.com/OpenZeppelin/koba

[Unit]: https://github.com/OpenZeppelin/stylus-test-helpers

[integration]: ./lib/e2e/README.md

## Usage

You can import OpenZeppelin Contracts from crates.io by adding the following
line to your `Cargo.toml` (We recommend pinning to a specific version):

```toml
[dependencies]
openzeppelin-stylus = "0.1.1"
```

Optionally, you can specify a git dependency if you want to have the latest
changes from the `main` branch:

```toml
[dependencies]
openzeppelin-stylus = { git = "https://github.com/OpenZeppelin/rust-contracts-stylus" }
```

> [!NOTE]
> This library is designed to be `no_std`, which helps reduce wasm size. If you want your project to be `no_std` as well, ensure that your dependencies are not importing the standard library.
>You can achieve this by setting `default-features = false` for relevant dependencies in your `Cargo.toml`. For example:
>
> ```toml
> [dependencies]
> alloy-primitives = { version = "=0.7.6", default-features = false }
>
> ```
>
> You will also need to define your own panic handler for `cargo stylus check` to pass.
> Here's an example of a simple panic handler you can use in your `lib.rs` file:
>
> ```rust
> #[cfg(target_arch = "wasm32")]
> #[panic_handler]
> fn panic(_info: &core::panic::PanicInfo) -> ! {
>     loop {}
> }
> ```
>
> The library also works on an `std` environment, without the need to define a panic handler or making extra changes to your project.

Once defined as a dependency, use one of our pre-defined implementations by
importing them:

```rust
use stylus_sdk::prelude::*;
use openzeppelin_stylus::token::erc20::Erc20;

#[entrypoint]
#[storage]
struct Erc20Example {
    #[borrow]
    pub erc20: Erc20,
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
[roadmap]: https://github.com/OpenZeppelin/rust-contracts-stylus/milestone/2

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
