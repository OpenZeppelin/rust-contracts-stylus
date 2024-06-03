//! Token standards.
#[cfg(any(feature = "std", feature = "erc20"))]
pub mod erc20;

#[cfg(any(feature = "std", feature = "erc721"))]
pub mod erc721;
