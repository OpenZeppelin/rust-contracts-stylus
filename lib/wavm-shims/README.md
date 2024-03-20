# WAVM Shims

Shims crate that mocks common host imports in Stylus `wasm` programs.

## Motivation

Without this crate we can't currently run unit tests for stylus contracts,
since the symbols the compiled binaries expect to find are not there.

If you run `cargo test` on a fresh Stylus project, it will error with:

    dyld[97792]: missing symbol called

This crate is a temporary solution until the Stylus team provides us with a
different and more stable mechanism for unit-testing our contracts.

## Usage

Import this crate in your test modules as `wavm_shims::*` to populate the
namespace with the appropriate symbols.

```rust
#[cfg(test)]
mod tests {
    use wavm_shims::*;

    #[test]
    fn reads_balance() {
        let token = init_token(); // Init an ERC-20, for example.
        let balance = token.balance_of(Address::ZERO); // Access storage.
        assert_eq!(balance, U256::ZERO);
    }
}
```

---

We maintain this crate on a best-effort basis. We use it extensively on our own
tests, so we will add here any symbols we may need. However, since we expect
this to be a temporary solution, don't expect us to address all requests.

That being said, please do open an issue to start a discussion, keeping in mind
our [code of conduct] and [contribution guidelines].

[code of conduct]: ../../CODE_OF_CONDUCT.md
[contribution guidelines]: ../../CONTRIBUTING.md
