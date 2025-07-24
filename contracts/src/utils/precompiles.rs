//! `ArbOS` precompiles wrapper enabling easier invocation.

use alloc::string::ToString;

use alloy_primitives::{
    address,
    aliases::{B1024, B2048},
    hex::FromHex,
    Address, B256,
};
pub use bls_error::*;
use primitives::ecrecover;
use stylus_sdk::{
    call::{self},
    prelude::*,
};

use crate::utils::cryptography::ecdsa::recover;
#[cfg_attr(coverage_nightly, coverage(off))]
mod bls_error {
    use alloy_sol_macro::sol;
    use stylus_sdk::{call::MethodError, prelude::*};

    sol! {
        /// Invalid input to the `BLS12_G1ADD` precompile.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error BLS12G1AddInvalidInput();
        /// The `BLS12_G1ADD` precompile failed to execute.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error BLS12G1AddPrecompileFailed();
        /// Invalid output from the `BLS12_G1ADD` precompile.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error BLS12G1AddInvalidOutput(string output);
    }

    /// An [`Erc20`] error defined as described in [ERC-6093].
    ///
    /// [ERC-6093]: https://eips.ethereum.org/EIPS/eip-6093
    #[derive(SolidityError, Debug)]
    pub enum Error {
        /// Invalid input to the `BLS12_G1ADD` precompile.
        Bls12G1AddInvalidInput(BLS12G1AddInvalidInput),
        /// Invalid output from the `BLS12_G1ADD` precompile.
        Bls12G1AddInvalidOutput(BLS12G1AddInvalidOutput),
        /// The `BLS12_G1ADD` precompile failed to execute.
        Bls12G1AddPrecompileFailed(BLS12G1AddPrecompileFailed),
    }

    impl MethodError for Error {
        fn encode(self) -> alloc::vec::Vec<u8> {
            self.into()
        }
    }
}

/// Address of the `BLS12_G1ADD` precompile.
pub const BLS12_G1ADD_ADDR: Address =
    address!("000000000000000000000000000000000000000b");

/// Precompile primitives.
pub mod primitives {
    /// The `ecrecover` precompile primitives.
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
    ///   [`primitives::ecrecover::SIGNATURE_S_UPPER_BOUND`].
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
    ) -> Result<Address, ecrecover::Error>;

    /// Adds two points on the BLS12-G1 curve.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `a` - First point.
    /// * `b` - Second point.
    ///
    /// # Panics
    ///
    /// * If the input is neither a point on the G1 elliptic curve nor the
    ///   infinity point.
    /// * If the input has invalid coordinate encoding.

    fn bls12_g1_add(
        &self,
        a: B1024,
        b: B1024,
    ) -> Result<B1024, bls_error::Error>;
}

impl<T: TopLevelStorage> Precompiles for T {
    fn ecrecover(
        &mut self,
        hash: B256,
        v: u8,
        r: B256,
        s: B256,
    ) -> Result<Address, ecrecover::Error> {
        Ok(recover(self, hash, v, r, s)?)
    }

    fn bls12_g1_add(
        &self,
        a: B1024,
        b: B1024,
    ) -> Result<B1024, bls_error::Error> {
        let input = B2048::try_from([a, b].concat().as_slice())
            .map_err(|_| BLS12G1AddInvalidInput {})?;

        let output = call::static_call(
            self,
            BLS12_G1ADD_ADDR,
            input.as_slice().as_ref(),
        )
        .map_err(|_| BLS12G1AddPrecompileFailed {})?;

        B1024::try_from(output.as_slice()).map_err(|_| {
            BLS12G1AddInvalidOutput { output: output.len().to_string() }.into()
        })
    }
}
