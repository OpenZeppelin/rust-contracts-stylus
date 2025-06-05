//! `ArbOS` precompile wrapper enabling easier precompile invocation.
use alloy_primitives::{Address, B256};
use primitives::ecrecover::Error;
use stylus_sdk::prelude::*;

use crate::utils::cryptography::ecdsa::recover;

/// Precompile primitives.
pub mod primitives {
    /// The ecrecover precompile primitives.
    ///
    /// This module provides the cryptographic primitives needed for the
    /// `ecrecover` precompile, which recovers the signer address from an
    /// ECDSA signature and message hash.
    ///
    /// Re-exports selected ECDSA types and constants specifically relevant
    /// to the ecrecover operation.
    pub mod ecrecover {
        pub use crate::utils::cryptography::ecdsa::{
            ECDSAInvalidSignature, ECDSAInvalidSignatureS, EcRecoverData,
            Error, ECRECOVER_ADDR, SIGNATURE_S_UPPER_BOUND,
        };
    }
}

/// Trait providing access to Arbitrum precompiles for Stylus contracts.
///
/// This trait wraps complex precompile invocations to provide a clean,
/// ergonomic interface for calling Arbitrum's built-in cryptographic functions
/// and utilities from within Stylus smart contracts.
///
/// Precompiles are pre-deployed contracts at fixed addresses that implement
/// commonly used cryptographic operations and other utilities. They execute
/// natively in the Arbitrum runtime for better performance compared to
/// implementing these operations in contract code.
///
/// See: <https://docs.arbitrum.io/build-decentralized-apps/precompiles/overview>
///
/// # Usage
///
/// Implement this trait for your contract storage type to gain access to
/// precompile functionality:
///
/// ```rust,ignore
/// use openzeppelin_stylus::utils::cryptography::Precompiles;
///
/// #[storage]
/// #[entrypoint]
/// struct MyContract {
///     // your fields...
/// }
///
/// // The `Precompiles` trait is automatically implemented for all
/// // contracts annotated with `#[entrypoint]` or that implement
/// // `stylus_sdk::prelude::TopLevelStorage`.
///
/// #[public]
/// impl MyContract {
///     fn verify_signature(&mut self, msg_hash: B256, sig: (u8, B256, B256)) -> Result<Address, Error> {
///         let (v, r, s) = sig;
///         self.ecrecover(msg_hash, v, r, s)
///     }
/// }
/// ```
///
/// # Error Handling
///
/// Precompile methods return `Result` types to handle both invalid inputs and
/// precompile execution failures. Always handle these errors appropriately
/// in your contract logic.
pub trait Precompiles: TopLevelStorage {
    /// Returns the address that signed a hashed message (`hash`).
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state. given address.
    /// * `hash` - Hash of the message.
    /// * `v` - `v` value from the signature.
    /// * `r` - `r` value from the signature.
    /// * `s` - `s` value from the signature.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSignatureS`] - If the `s` value is grater than
    ///   [`ecrecover::SIGNATURE_S_UPPER_BOUND`].
    /// * [`Error::InvalidSignature`] - If the recovered address is
    ///   [`Address::ZERO`].
    ///
    /// # Panics
    ///
    /// * If the `ecrecover` precompile fails to execute.
    fn ecrecover(
        &mut self,
        hash: B256,
        v: u8,
        r: B256,
        s: B256,
    ) -> Result<Address, Error>;
}

impl<T: TopLevelStorage> Precompiles for T {
    fn ecrecover(
        &mut self,
        hash: B256,
        v: u8,
        r: B256,
        s: B256,
    ) -> Result<Address, Error> {
        recover(self, hash, v, r, s)
    }
}
