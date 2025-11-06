//! Elliptic Curve Digital Signature Algorithm (ECDSA) operations.
//!
//! These functions can be used to verify that a message was signed
//! by the holder of the private keys of a given address.
use alloc::vec::Vec;

use alloy_primitives::{address, uint, Address, B256, U256};
use alloy_sol_types::SolType;
use stylus_sdk::{
    call::{self, MethodError, StaticCallContext},
    prelude::*,
};

/// Address of the `ecRecover` EVM precompile.
pub const ECRECOVER_ADDR: Address =
    address!("0000000000000000000000000000000000000001");

/// Upper range for `s` value from the signature.
pub const SIGNATURE_S_UPPER_BOUND: U256 = uint!(
    0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0_U256
);

pub use sol::*;
#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// The signature derives the [`Address::ZERO`].
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ECDSAInvalidSignature();

        /// The signature has an `S` value that is in the upper half order.
        ///
        /// * `s` - Invalid `S` value.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ECDSAInvalidSignatureS(bytes32 s);
    }

    sol! {
        /// Struct with callable data to the `ecRecover` precompile.
        #[allow(missing_docs)]
        struct EcRecoverData {
            /// EIP-191 Hash of the message.
            bytes32 hash;
            /// `v` value from the signature.
            uint8 v;
            /// `r` value from the signature.
            bytes32 r;
            /// `s` value from the signature.
            bytes32 s;
        }
    }
}

/// An error that occurred in the implementation of an `ECDSA` library.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// The signature derives the [`Address::ZERO`].
    InvalidSignature(ECDSAInvalidSignature),
    /// The signature has an `S` value that is in the upper half order.
    InvalidSignatureS(ECDSAInvalidSignatureS),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// Returns the address that signed a hashed message (`hash`).
///
/// # Arguments
///
/// * `context` - Execution context for making static calls.
/// * `hash` - Hash of the message.
/// * `v` - `v` value from the signature.
/// * `r` - `r` value from the signature.
/// * `s` - `s` value from the signature.
///
/// # Errors
///
/// * [`Error::InvalidSignatureS`] - If the `s` value is grater than
///   [`SIGNATURE_S_UPPER_BOUND`].
/// * [`Error::InvalidSignature`] - If the recovered address is
///   [`Address::ZERO`].
///
/// # Panics
///
/// * If the `ecRecover` precompile fails to execute.
pub fn recover(
    context: impl StaticCallContext,
    hash: B256,
    v: u8,
    r: B256,
    s: B256,
) -> Result<Address, Error> {
    check_if_malleable(&s)?;
    // If the signature is valid (and not malleable), return the signer address.
    _recover(context, hash, v, r, s)
}

/// Calls `ecRecover` EVM precompile.
///
/// The `ecRecover` EVM precompile allows for malleable (non-unique) signatures:
/// this function rejects them by requiring the `s` value to be in the lower
/// half order, and the `v` value to be either 27 or 28.
///
/// # Arguments
///
/// * `context` - Execution context for making static calls.
/// * `hash` - Hash of the message.
/// * `v` - `v` value from the signature.
/// * `r` - `r` value from the signature.
/// * `s` - `s` value from the signature.
///
/// # Errors
///
/// * [`Error::InvalidSignature`] - If the recovered address is
///   [`Address::ZERO`].
///
/// # Panics
///
/// * If the `ecRecover` precompile fails to execute.
fn _recover(
    context: impl StaticCallContext,
    hash: B256,
    v: u8,
    r: B256,
    s: B256,
) -> Result<Address, Error> {
    let calldata = encode_calldata(hash, v, r, s);

    if v == 0 || v == 1 {
        // `ecRecover` panics for these values but following the Solidity tests
        // https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/test/utils/cryptography/ECDSA.test.js
        // it should return `ECDSAInvalidSignature` error.
        return Err(ECDSAInvalidSignature {}.into());
    }

    let recovered = call::static_call(context, ECRECOVER_ADDR, &calldata)
        .expect("should call `ecRecover` precompile");

    let recovered = Address::from_slice(&recovered[12..]);

    if recovered.is_zero() {
        return Err(ECDSAInvalidSignature {}.into());
    }
    Ok(recovered)
}

/// Encodes call data for `ecRecover` EVM precompile.
///
/// # Arguments
///
/// * `hash` - Hash of the message.
/// * `v` - `v` value from the signature.
/// * `r` - `r` value from the signature.
/// * `s` - `s` value from the signature.
fn encode_calldata(hash: B256, v: u8, r: B256, s: B256) -> Vec<u8> {
    let calldata = EcRecoverData { hash, v, r, s };
    EcRecoverData::abi_encode(&calldata)
}

/// Validates the `s` value of a signature.
///
/// EIP-2 still allows signature malleability for `ecRecover` precompile.
///
/// Remove this possibility and make the signature unique.
///
/// Appendix F in the [Ethereum Yellow paper], defines the valid range for s in
/// (301): 0 < s < secp256k1n ÷ 2 + 1, and for v in (302): v ∈ {27, 28}.
///
/// Most signatures from current libraries generate a unique signature
/// with an s-value in the lower half order.
///
/// If your library generates malleable signatures,
/// such as s-values in the upper range, calculate a new s-value
/// with 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141 -
/// s1, and flip v from 27 to 28 or vice versa.
///
/// If your library also generates signatures with 0/1 for v instead 27/28,
/// add 27 to v to accept these malleable signatures as well.
///
/// # Arguments
///
/// * `s` - `s` value from the signature.
///
/// # Errors
///
/// * [`Error::InvalidSignatureS`] - If the `s` value is greater than
///   [`SIGNATURE_S_UPPER_BOUND`].
///
/// [Ethereum Yellow paper]: https://ethereum.github.io/yellowpaper/paper.pdf
fn check_if_malleable(s: &B256) -> Result<(), Error> {
    let s_u256 = U256::from_be_slice(s.as_slice());
    if s_u256 > SIGNATURE_S_UPPER_BOUND {
        return Err(ECDSAInvalidSignatureS { s: *s }.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{b256, B256};
    use motsu::prelude::*;

    use super::*;

    const MSG_HASH: B256 = b256!(
        "a1de988600a42c4b4ab089b619297c17d53cffae5d5120d82d8a92d0bb3b78f2"
    );
    const V: u8 = 28;
    const R: B256 = b256!(
        "65e72b1cf8e189569963750e10ccb88fe89389daeeb8b735277d59cd6885ee82"
    );
    const S: B256 = b256!(
        "3eb5a6982b540f185703492dab77b863a88ce01f27e21ade8b2879c10fc9e653"
    );

    #[test]
    fn prepares_calldata() {
        let expected = alloy_primitives::bytes!("a1de988600a42c4b4ab089b619297c17d53cffae5d5120d82d8a92d0bb3b78f2000000000000000000000000000000000000000000000000000000000000001c65e72b1cf8e189569963750e10ccb88fe89389daeeb8b735277d59cd6885ee823eb5a6982b540f185703492dab77b863a88ce01f27e21ade8b2879c10fc9e653");
        let calldata = encode_calldata(MSG_HASH, V, R, S);
        assert_eq!(expected, calldata);
    }

    #[test]
    fn rejects_invalid_s() {
        let invalid_s = SIGNATURE_S_UPPER_BOUND + U256::ONE;
        let invalid_s = B256::from_slice(&invalid_s.to_be_bytes_vec());
        let err = check_if_malleable(&invalid_s)
            .expect_err("should return ECDSAInvalidSignatureS");

        assert!(matches!(err,
                Error::InvalidSignatureS(ECDSAInvalidSignatureS {
                    s
                }) if s == invalid_s
        ));
    }

    #[test]
    fn validates_s() {
        let valid_s = SIGNATURE_S_UPPER_BOUND - U256::ONE;
        let invalid_s = B256::from_slice(&valid_s.to_be_bytes_vec());
        let result = check_if_malleable(&invalid_s);
        assert!(result.is_ok());
    }

    #[storage]
    struct ContractStorage;

    unsafe impl TopLevelStorage for ContractStorage {}

    #[public]
    impl ContractStorage {}

    #[motsu::test]
    fn _recover_fails_when_v_is_0_or_1(
        contract: Contract<ContractStorage>,
        alice: Address,
    ) {
        let err = _recover(&mut *contract.sender(alice), MSG_HASH, 0, R, S)
            .motsu_expect_err("should return ECDSAInvalidSignature");

        assert!(matches!(
            err,
            Error::InvalidSignature(ECDSAInvalidSignature {})
        ));
    }

    #[motsu::test]
    fn _recover_fails_when_recovered_address_is_zero(
        contract: Contract<ContractStorage>,
        alice: Address,
    ) {
        let invalid_v = 30;

        let err = _recover(&*contract.sender(alice), MSG_HASH, invalid_v, R, S)
            .motsu_expect_err("should return ECDSAInvalidSignature");

        assert!(matches!(
            err,
            Error::InvalidSignature(ECDSAInvalidSignature {})
        ));
    }
}
