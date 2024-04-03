# Grip - Unit Testing for Stylus

The name `grip` is an analogy to the place where you put your fingers to hold a
stylus pen.

Note that currently, test suites using `grip::test` will run serially because
of global access to storage.

## Usage

Annotate tests with `#[grip::test]` instead of `#[test]` to get access to the
affordances:

Import these shims in your test modules as `grip::prelude::*` to populate the
namespace with the appropriate symbols.

```rust,ignore
#[cfg(test)]
mod tests {
    use contracts::erc20::ERC20;

    #[grip::test]
    fn reads_balance(contract: ERC20) {
        let balance = contract.balance_of(Address::ZERO); // Access storage.
        assert_eq!(balance, U256::ZERO);
    }
}
```

Annotating a test function that accepts no parameters will make `#[grip::test]`
behave the same as `#[test]`.

```rust,ignore
#[cfg(test)]
mod tests {
    #[grip::test] // Equivalent to #[test]
    fn test_fn() {
        ...
    }
}
```

### Notice

We maintain this crate on a best-effort basis. We use it extensively on our own
tests, so we will add here any symbols we may need. However, since we expect
this to be a temporary solution, don't expect us to address all requests.

That being said, please do open an issue to start a discussion, keeping in mind
our [code of conduct] and [contribution guidelines].

[code of conduct]: ../../CODE_OF_CONDUCT.md
[contribution guidelines]: ../../CONTRIBUTING.md

## Security

> [!WARNING]
> This project is still in a very early and experimental phase. It has never
> been audited nor thoroughly reviewed for security vulnerabilities. Do not use
> in production.

Refer to our [Security Policy](../../SECURITY.md) for more details.
