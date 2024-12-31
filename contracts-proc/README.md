# OpenZeppelin Stylus Procedural Macros

Procedural macros for [OpenZeppelin Stylus](../contracts) smart contracts, providing tools to streamline trait definitions, interface ID computation, and Solidity compatibility in Rust-based Stylus contracts.

## Overview

This crate offers procedural macros for OpenZeppelin Stylus contracts, specifically targeting the computation of Solidity `interfaceId` values and enhancing trait implementations with clear and robust syntax.

### Key Features

- **`#[interface_id]` Macro:** Automatically computes Solidity-compatible `INTERFACE_ID` constants for traits.
- **`#[selector]` Attribute:** Override function names to align with Solidity method signatures.

## Usage

### `#[interface_id]`

Annotate a Rust trait with `#[interface_id]` to compute the Solidity-compatible `INTERFACE_ID`:

```rust
use openzeppelin_stylus_proc::interface_id;

#[interface_id]
pub trait IErc721 {
    fn balance_of(&self, owner: Address) -> Result<U256, Vec<u8>>;
    fn owner_of(&self, token_id: U256) -> Result<Address, Vec<u8>>;

    // ...
}
```

This will generate an `INTERFACE_ID` constant based on the XOR of the function selectors.

### `#[selector]`

Override the Solidity function selector explicitly:

```rust
fn safe_transfer_from(
    &mut self,
    from: Address,
    to: Address,
    token_id: U256,
) -> Result<(), Self::Error>;

// Solidity allows function overloading, but Rust does not, so we map
// the Rust function name to the appropriate Solidity function name.
#[selector(name = "safeTransferFrom")]
fn safe_transfer_from_with_data(
    &mut self,
    from: Address,
    to: Address,
    token_id: U256,
    data: Bytes,
) -> Result<(), Vec<u8>>;
```

This ensures compatibility with Solidity's naming conventions.

## Security

Refer to our [Security Policy](../../SECURITY.md) for more details.
