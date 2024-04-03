# Grip - Unit Testing for Stylus

This crate enables unit-testing for Stylus contracts. It abstracts away the
machinery necessary for writing tests behind a `#[grip::test]` procedural
macro.

The name `grip` is an analogy to the place where you put your fingers to hold a
stylus pen.

## Usage

Annotate tests with `#[grip::test]` instead of `#[test]` to get access to VM
affordances.

Note that we require contracts to implement `core::default::Default`. This
implementation should match the way solidity would lay out the contract's state
in storage, so that the tests are as close as possible to the real environment.

```rust
#[cfg(test)]
mod tests {
    use contracts::erc20::ERC20;

    impl Default for ERC20 {
        fn default() -> Self {
            let root = U256::ZERO;
            ERC20 {
                _balances: unsafe { StorageMap::new(root, 0) },
                _allowances: unsafe {
                    StorageMap::new(root + U256::from(32), 0)
                },
                _total_supply: unsafe {
                    StorageU256::new(root + U256::from(64), 0)
                },
            }
        }
    }

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
    #[grip::test]
     fn t() { // If no params, it expands to a `#[test]`.
        // ...
    }
}
```

Note that currently, test suites using `grip::test` will run serially because
of global access to storage.

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
