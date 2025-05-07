//! Permit Contract.
//!
//! Extension of the ERC-20 standard allowing approvals to be made
//! via signatures, as defined in the [ERC].
//!
//! Adds the `permit` method, which can be used to change an account’s
//! ERC20 allowance (see [`crate::token::erc20::IErc20::allowance`])
//! by presenting a message signed by the account.
//! By not relying on [`erc20::IErc20::approve`],
//! the token holder account doesn’t need to send a transaction,
//! and thus is not required to hold Ether at all.
//!
//! [ERC]: https://eips.ethereum.org/EIPS/eip-2612

use alloc::vec;

use alloy_primitives::{keccak256, Address, B256, U256};
use alloy_sol_types::SolType;
use stylus_sdk::{block, call::MethodError, prelude::*};

use crate::{
    token::erc20::{self, Erc20},
    utils::{
        cryptography::{
            ecdsa::{self, ECDSAInvalidSignature, ECDSAInvalidSignatureS},
            eip712::IEip712,
        },
        nonces::Nonces,
    },
};

const PERMIT_TYPEHASH: [u8; 32] =
    keccak_const::Keccak256::new()
        .update(b"Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)")
        .finalize();

pub use sol::*;
#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    pub(crate) type StructHashTuple = sol! {
        tuple(bytes32, address, address, uint256, uint256, uint256)
    };

    sol! {
        /// Indicates an error related to the fact that
        /// permit deadline has expired.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2612ExpiredSignature(uint256 deadline);

        /// Indicates an error related to the issue about mismatched signature.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2612InvalidSigner(address signer, address owner);
    }
}

/// A Permit error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the fact that
    /// permit deadline has expired.
    ExpiredSignature(ERC2612ExpiredSignature),
    /// Indicates an error related to the issue about mismatched signature.
    InvalidSigner(ERC2612InvalidSigner),
    /// Indicates an error related to the current balance of `sender`. Used in
    /// transfers.
    InsufficientBalance(erc20::ERC20InsufficientBalance),
    /// Indicates a failure with the token `sender`. Used in transfers.
    InvalidSender(erc20::ERC20InvalidSender),
    /// Indicates a failure with the token `receiver`. Used in transfers.
    InvalidReceiver(erc20::ERC20InvalidReceiver),
    /// Indicates a failure with the `spender`’s `allowance`. Used in
    /// transfers.
    InsufficientAllowance(erc20::ERC20InsufficientAllowance),
    /// Indicates a failure with the `spender` to be approved. Used in
    /// approvals.
    InvalidSpender(erc20::ERC20InvalidSpender),
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals. approver Address initiating an approval operation.
    InvalidApprover(erc20::ERC20InvalidApprover),
    /// The signature derives the `Address::ZERO`.
    InvalidSignature(ECDSAInvalidSignature),
    /// The signature has an `S` value that is in the upper half order.
    InvalidSignatureS(ECDSAInvalidSignatureS),
}

impl From<erc20::Error> for Error {
    fn from(value: erc20::Error) -> Self {
        match value {
            erc20::Error::InsufficientBalance(e) => {
                Error::InsufficientBalance(e)
            }
            erc20::Error::InvalidSender(e) => Error::InvalidSender(e),
            erc20::Error::InvalidReceiver(e) => Error::InvalidReceiver(e),
            erc20::Error::InsufficientAllowance(e) => {
                Error::InsufficientAllowance(e)
            }
            erc20::Error::InvalidSpender(e) => Error::InvalidSpender(e),
            erc20::Error::InvalidApprover(e) => Error::InvalidApprover(e),
        }
    }
}

impl From<ecdsa::Error> for Error {
    fn from(value: ecdsa::Error) -> Self {
        match value {
            ecdsa::Error::InvalidSignature(e) => Error::InvalidSignature(e),
            ecdsa::Error::InvalidSignatureS(e) => Error::InvalidSignatureS(e),
        }
    }
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of an [`Erc20Permit`] Contract.
#[storage]
pub struct Erc20Permit<T: IEip712 + StorageType> {
    /// Contract implementing [`IEip712`] trait.
    pub(crate) eip712: T,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl<T: IEip712 + StorageType> TopLevelStorage for Erc20Permit<T> {}

#[public]
impl<T: IEip712 + StorageType> Erc20Permit<T> {
    /// Returns the domain separator used in the encoding of the signature for
    /// [`Self::permit`], as defined by EIP712.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[selector(name = "DOMAIN_SEPARATOR")]
    #[must_use]
    pub fn domain_separator(&self) -> B256 {
        self.eip712.domain_separator_v4()
    }
}

impl<T: IEip712 + StorageType> Erc20Permit<T> {
    /// Sets `value` as the allowance of `spender` over `owner`'s tokens,
    /// given `owner`'s signed approval.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state. given address.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    /// * `value` - The number of tokens being permitted to transfer by
    ///   `spender`.
    /// * `deadline` - Deadline for the permit action.
    /// * `v` - v value from the `owner`'s signature.
    /// * `r` - r value from the `owner`'s signature.
    /// * `s` - s value from the `owner`'s signature.
    /// * `erc20` - Write access to an [`Erc20`] contract.
    /// * `nonces` - Write access to a [`Nonces`] contract.
    ///
    /// # Errors
    ///
    /// * [`ERC2612ExpiredSignature`] - If the `deadline` param is from the
    ///   past.
    /// * [`ERC2612InvalidSigner`] - If signer is not an `owner`.
    /// * [`ecdsa::Error::InvalidSignatureS`] - If the `s` value is grater than
    ///   [`ecdsa::SIGNATURE_S_UPPER_BOUND`].
    /// * [`ecdsa::Error::InvalidSignature`] - If the recovered address is
    ///   `Address::ZERO`.
    /// * [`erc20::Error::InvalidSpender`] - If the `spender` address is
    ///   `Address::ZERO`.
    ///
    /// # Events
    ///
    /// * [`erc20::Approval`]
    #[allow(clippy::too_many_arguments)]
    pub fn permit(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
        deadline: U256,
        v: u8,
        r: B256,
        s: B256,
        erc20: &mut Erc20,
        nonces: &mut Nonces,
    ) -> Result<(), Error> {
        if U256::from(block::timestamp()) > deadline {
            return Err(ERC2612ExpiredSignature { deadline }.into());
        }

        let struct_hash = keccak256(StructHashTuple::abi_encode(&(
            PERMIT_TYPEHASH,
            owner,
            spender,
            value,
            nonces.use_nonce(owner),
            deadline,
        )));

        let hash: B256 = self.eip712.hash_typed_data_v4(struct_hash);

        let signer: Address = ecdsa::recover(self, hash, v, r, s)?;

        if signer != owner {
            return Err(ERC2612InvalidSigner { signer, owner }.into());
        }

        erc20._approve(owner, spender, value, true)?;

        Ok(())
    }
}
