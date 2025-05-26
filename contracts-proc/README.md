# OpenZeppelin Stylus Procedural Macros

Procedural macros for [OpenZeppelin Stylus Contracts](../contracts), providing tools to streamline trait definitions, interface ID computation, and Solidity compatibility in Rust-based Stylus Contracts.

## Overview

This crate offers procedural macros for OpenZeppelin Stylus Contracts, specifically targeting the computation of Solidity `interfaceId` value and enhancing trait implementations with clear and robust syntax.

### Key Features

- **`#[interface_id]` Macro:** Adds `interface_id()` function that computes Solidity-compatible interface ID for traits.
- **`#[selector]` Attribute:** Overrides function names to align with Solidity method signatures.

## Usage

### `#[interface_id]`

Annotate a Rust trait with `#[interface_id]` to add the Solidity-compatible interface ID calculation:

```rust,ignore
use openzeppelin_stylus_proc::interface_id;

#[interface_id]
pub trait IErc721 {
    type Error: Into<alloc::vec::Vec<u8>>;

    fn balance_of(&self, owner: Address) -> Result<U256, Self::Error>;
    fn owner_of(&self, token_id: U256) -> Result<Address, Self::Error>;

    // ...
}
```

This will add `interface_id()` function that caluclates interface ID based on the XOR of the function selectors.

### `#[selector]`

Override the Solidity function selector explicitly:

```rust,ignore
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
) -> Result<(), Self::Error>;
```

This ensures compatibility with Solidity's naming conventions.

## Security

Refer to our [Security Policy](../SECURITY.md) for more details.
