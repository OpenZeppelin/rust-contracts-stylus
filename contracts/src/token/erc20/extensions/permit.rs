//! Permit Contract.
//!
//! Extension of the ERC-20 standard allowing approvals to be made
//! via signatures, as defined in EIP-2612.
//!
//! Adds the `permit` method, which can be used to change an account’s
//! ERC20 allowance (see [`crate::token::erc20::IErc20::allowance`])
//! by presenting a message signed by the account.
//! By not relying on [`crate::token::erc20::IErc20::approve`],
//! the token holder account doesn’t need to send a transaction,
//! and thus is not required to hold Ether at all.
use alloy_primitives::{b256, keccak256, Address, B256, U256};
use alloy_sol_types::{sol, SolType};
use stylus_sdk::{
    block,
    prelude::StorageType,
    storage::TopLevelStorage,
    stylus_proc::{public, sol_storage, SolidityError},
};

use crate::{
    token::erc20::{self, Erc20, IErc20},
    utils::{
        cryptography::{ecdsa, eip712::IEip712},
        nonces::Nonces,
    },
};

// keccak256("Permit(address owner,address spender,uint256 value,uint256
// nonce,uint256 deadline)")
const PERMIT_TYPEHASH: B256 =
    b256!("6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c9");

type StructHashTuple = sol! {
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

/// A Permit error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the fact that
    /// permit deadline has expired.
    ExpiredSignature(ERC2612ExpiredSignature),
    /// Indicates an error related to the issue about mismatched signature.
    InvalidSigner(ERC2612InvalidSigner),
    /// Error type from [`Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error),
    /// Error type from [`ecdsa`] contract [`ecdsa::Error`].
    ECDSA(ecdsa::Error),
}

sol_storage! {
    /// State of a Permit Contract.
    pub struct Erc20Permit<T: IEip712 + StorageType>{
        /// ERC-20 contract.
        Erc20 erc20;

        /// Nonces contract.
        Nonces nonces;

        /// EIP-712 contract. Must implement [`IEip712`] trait.
        T eip712;
    }
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl<T: IEip712 + StorageType> TopLevelStorage for Erc20Permit<T> {}

#[public]
impl<T: IEip712 + StorageType> Erc20Permit<T> {
    /// Returns the current nonce for `owner`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - The address for which to return the nonce.
    #[must_use]
    pub fn nonces(&self, owner: Address) -> U256 {
        self.nonces.nonces(owner)
    }

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
    ///
    /// # Errors
    ///
    /// If the `deadline` param is from the past, than the error
    /// [`ERC2612ExpiredSignature`] is returned.
    /// If signer is not an `owner`, than the error
    /// [`ERC2612InvalidSigner`] is returned.
    /// * If the `s` value is grater than [`ecdsa::SIGNATURE_S_UPPER_BOUND`],
    /// then the error [`ecdsa::Error::InvalidSignatureS`] is returned.
    /// * If the recovered address is `Address::ZERO`, then the error
    /// [`ecdsa::Error::InvalidSignature`] is returned.
    /// If the `spender` address is `Address::ZERO`, then the error
    /// [`erc20::Error::InvalidSpender`] is returned.
    ///
    /// # Events
    ///
    /// Emits an [`crate::token::erc20::Approval`] event.
    ///
    /// # Requirements
    ///
    /// * `spender` cannot be the ``Address::ZERO``.
    /// * `deadline` must be a timestamp in the future.
    /// * `v`, `r` and `s` must be a valid secp256k1 signature from `owner`
    /// over the EIP712-formatted function arguments.
    /// * the signature must use `owner`'s current nonce.
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
    ) -> Result<(), Error> {
        if U256::from(block::timestamp()) > deadline {
            return Err(ERC2612ExpiredSignature { deadline }.into());
        }

        let struct_hash = keccak256(StructHashTuple::abi_encode(&(
            *PERMIT_TYPEHASH,
            owner,
            spender,
            value,
            self.nonces.use_nonce(owner),
            deadline,
        )));

        let hash: B256 = self.eip712.hash_typed_data_v4(struct_hash);

        let signer: Address = ecdsa::recover(self, hash, v, r, s)?;

        if signer != owner {
            return Err(ERC2612InvalidSigner { signer, owner }.into());
        }

        self.erc20._approve(owner, spender, value, true)?;

        Ok(())
    }

    /// Returns the number of tokens in existence.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn total_supply(&self) -> U256 {
        self.erc20.total_supply()
    }

    /// Returns the number of tokens owned by `account`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - Account to get balance from.
    pub fn balance_of(&self, account: Address) -> U256 {
        self.erc20.balance_of(account)
    }

    /// Moves a `value` amount of tokens from the caller's account to `to`.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    ///
    /// # Errors
    ///
    /// * If the `to` address is `Address::ZERO`, then the error
    /// [`crate::token::erc20::Error::InvalidReceiver`] is returned.
    /// * If the caller doesn't have a balance of at least `value`, then the
    /// error [`crate::token::erc20::Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`crate::token::erc20::Transfer`] event.
    pub fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, crate::token::erc20::Error> {
        self.erc20.transfer(to, value)
    }

    /// Returns the remaining number of tokens that `spender` will be allowed
    /// to spend on behalf of `owner` through `transfer_from`. This is zero by
    /// default.
    ///
    /// This value changes when `approve` or `transfer_from` are called.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }

    /// Sets a `value` number of tokens as the allowance of `spender` over the
    /// caller's tokens.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// WARNING: Beware that changing an allowance with this method brings the
    /// risk that someone may use both the old and the new allowance by
    /// unfortunate transaction ordering. One possible solution to mitigate
    /// this race condition is to first reduce the `spender`'s allowance to 0
    /// and set the desired value afterwards:
    /// <https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729>
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    /// * `value` - The number of tokens being allowed to transfer by `spender`.
    ///
    /// # Errors
    ///
    /// If the `spender` address is `Address::ZERO`, then the error
    /// [`crate::token::erc20::Error::InvalidSpender`] is returned.
    ///
    /// # Events
    ///
    /// Emits an [`crate::token::erc20::Approval`] event.
    pub fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, crate::token::erc20::Error> {
        self.erc20.approve(spender, value)
    }

    /// Moves a `value` number of tokens from `from` to `to` using the
    /// allowance mechanism. `value` is then deducted from the caller's
    /// allowance.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// NOTE: If `value` is the maximum `U256::MAX`, the allowance is not
    /// updated on `transfer_from`. This is semantically equivalent to
    /// an infinite approval.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    ///
    /// # Errors
    ///
    /// * If the `from` address is `Address::ZERO`, then the error
    /// [`crate::token::erc20::Error::InvalidSender`] is returned.
    /// * If the `to` address is `Address::ZERO`, then the error
    /// [`crate::token::erc20::Error::InvalidReceiver`] is returned.
    /// * If not enough allowance is available, then the error
    /// [`crate::token::erc20::Error::InsufficientAllowance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`crate::token::erc20::Transfer`] event.
    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, crate::token::erc20::Error> {
        self.erc20.transfer_from(from, to, value)
    }
}
