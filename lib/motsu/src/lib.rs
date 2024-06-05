//! # Motsu - Unit Testing for Stylus
//!
//! This crate enables unit-testing for Stylus contracts. It abstracts away the
//! machinery necessary for writing tests behind a
//! [`#[motsu::test]`][test_attribute] procedural macro.
//!
//! The name `motsu` is an analogy to the place where you put your fingers to
//! hold a stylus pen.
//!
//! ## Usage
//!
//! Annotate tests with [`#[motsu::test]`][test_attribute] instead of `#[test]`
//! to get access to VM affordances.
//!
//! Note that we require contracts to implement [`core::default::Default`]. This
//! implementation should match the way solidity would lay out the contract's
//! state in storage, so that the tests are as close as possible to the real
//! environment.
//!
//! ```rust
//! #[cfg(test)]
//! mod tests {
//!     use contracts::token::erc20::Erc20;
//!
//!     impl Default for Erc20 {
//!         fn default() -> Self {
//!             let root = U256::ZERO;
//!             Erc20 {
//!                 _balances: unsafe { StorageMap::new(root, 0) },
//!                 _allowances: unsafe {
//!                     StorageMap::new(root + U256::from(32), 0)
//!                 },
//!                 _total_supply: unsafe {
//!                     StorageU256::new(root + U256::from(64), 0)
//!                 },
//!             }
//!         }
//!     }
//!
//!     #[motsu::test]
//!     fn reads_balance(contract: Erc20) {
//!         let balance = contract.balance_of(Address::ZERO); // Access storage.
//!         assert_eq!(balance, U256::ZERO);
//!     }
//! }
//! ```
//!
//! Annotating a test function that accepts no parameters will make
//! [`#[motsu::test]`][test_attribute] behave the same as `#[test]`.
//!
//! ```rust,ignore
//! #[cfg(test)]
//! mod tests {
//!     #[motsu::test] // Equivalent to #[test]
//!     fn test_fn() {
//!         ...
//!     }
//! }
//! ```
//!
//! Note that currently, test suites using [`motsu::test`][test_attribute] will
//! run serially because of global access to storage.
//!
//! ### Notice
//!
//! We maintain this crate on a best-effort basis. We use it extensively on our
//! own tests, so we will add here any symbols we may need. However, since we
//! expect this to be a temporary solution, don't expect us to address all
//! requests.
//!
//! [test_attribute]: crate::test
mod context;
pub mod prelude;
mod shims;
mod storage;

pub use motsu_proc::test;
pub use motsu_proc::StylusDefault;
