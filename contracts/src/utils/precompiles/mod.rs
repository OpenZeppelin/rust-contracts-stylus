//! `ArbOS` precompiles wrapper enabling easier invocation.

use alloy_primitives::{Address, B256};
use primitives::ecrecover::Error;
use stylus_sdk::prelude::*;

use crate::utils::cryptography::ecdsa::recover;

mod p256_verify;

use p256_verify::p256_verify;
pub use p256_verify::P256_VERIFY_ADDRESS;

/// Precompile primitives.
pub mod primitives {
    /// The `ecRecover` precompile primitives.
    ///
    /// This module provides the cryptographic primitives needed for the
    /// `ecRecover` precompile, which recovers the signer address from an
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
///         self.ec_recover(msg_hash, v, r, s)
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
    /// Recovers the address that signed a hashed message (`hash`) using an
    /// ECDSA signature (v, r, s).
    ///
    /// Wrapper around the `ecRecover` precompile.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `hash` - Hash of the message.
    /// * `v` - `v` value from the signature.
    /// * `r` - `r` value from the signature.
    /// * `s` - `s` value from the signature.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSignatureS`] - If the `s` value is grater than
    ///   [`primitives::ecrecover::SIGNATURE_S_UPPER_BOUND`].
    /// * [`Error::InvalidSignature`] - If the recovered address is
    ///   [`Address::ZERO`].
    ///
    /// # Panics
    ///
    /// * If the `ecRecover` precompile fails to execute.
    fn ec_recover(
        &self,
        hash: B256,
        v: u8,
        r: B256,
        s: B256,
    ) -> Result<Address, Error>;

    /// Performs signature verifications in the `secp256r1` elliptic curve.
    ///
    /// Wrapper around the `P256VERIFY` precompile introduced in [RIP-7212].
    ///
    /// [RIP-7212]: https://github.com/ethereum/RIPs/blob/723155c3d86427412b5bc0f98ad1e4791ea7347f/RIPS/rip-7212.md
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `hash` - Signed data hash.
    /// * `r` - `r` component of the signature.
    /// * `s` - `s` component of the signature.
    /// * `x` - `x` coordinate of the public key.
    /// * `y` - `y` coordinate of the public key.
    fn p256_verify(
        &self,
        hash: B256,
        r: B256,
        s: B256,
        x: B256,
        y: B256,
    ) -> bool;
}

impl<T: TopLevelStorage> Precompiles for T {
    fn ec_recover(
        &self,
        hash: B256,
        v: u8,
        r: B256,
        s: B256,
    ) -> Result<Address, Error> {
        recover(self, hash, v, r, s)
    }

    fn p256_verify(
        &self,
        hash: B256,
        r: B256,
        s: B256,
        x: B256,
        y: B256,
    ) -> bool {
        p256_verify(self, hash, r, s, x, y)
    }
}
