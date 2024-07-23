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

use alloy_primitives::{fixed_bytes, keccak256, Address, B256, U256};
use alloy_sol_types::{sol, SolType};
use stylus_proc::{external, sol_storage, SolidityError};
use stylus_sdk::block;

use crate::{token::erc20::IErc20Internal, utils::nonces::Nonces};

const PERMIT_TYPEHASH: B256 = fixed_bytes!(
    "6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c9"
);

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
    /// .
    ExpiredSignature(ERC2612ExpiredSignature),
    /// .
    InvalidSigner(ERC2612InvalidSigner),
}

sol_storage! {
    /// State of a Permit Contract.
    #[allow(clippy::pub_underscore_fields)]
    pub struct Permit {
        /// Nonces contract.
        Nonces nonces;
    }
}

#[external]
impl Permit {
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
        unimplemented!()
        // Blocked by #184
    }
}

impl Permit {
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
    /// * `erc20` - Write access to a contract providing
    ///   [`crate::token::erc20::IErc20`] interface.
    ///
    /// # Errors
    ///
    /// If the `deadline` param is from the past, than the error
    /// [`ERC2612ExpiredSignature`] is returned.
    /// If signer is not an `owner`, than the error
    /// [`ERC2612InvalidSigner`] is returned.
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
        erc20: &mut impl IErc20Internal,
    ) -> Result<(), Error> {
        if U256::from(block::timestamp()) > deadline {
            return Err(ERC2612ExpiredSignature { deadline }.into());
        }

        let _struct_hash = keccak256(StructHashTuple::encode_params(&(
            *PERMIT_TYPEHASH,
            owner,
            spender,
            value,
            self.nonces.use_nonce(owner),
            deadline,
        )));

        // Blocked by #184.
        let _hash: B256 = todo!("_hashTypedDataV4(structHash)");

        // Blocked by #17.
        let signer: Address = todo!("ECDSA.recover(hash, v, r, s)");

        if signer != owner {
            return Err(ERC2612InvalidSigner { signer, owner }.into());
        }

        let _ = erc20._approve(owner, spender, value);
        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {}
