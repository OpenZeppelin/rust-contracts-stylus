//! Implementation of the NFT Royalty Standard, a standardized way to retrieve
//! royalty payment information.
//!
//! Royalty information can be specified globally for all token ids via
//! {_setDefaultRoyalty}, and/or individually for specific token ids via
//! {_setTokenRoyalty}. The latter takes precedence over the first.
//!
//! Royalty is specified as a fraction of sale price. {_feeDenominator} is
//! overridable but defaults to 10000, meaning the fee is specified in basis
//! points by default.
//!
//! IMPORTANT: ERC-2981 only specifies a way to signal royalty information and
//! does not enforce its payment.
//!
//! See `<https://eips.ethereum.org/EIPS/eip-2981#optional-royalty-payments>` in the ERC.
//! Marketplaces are expected to voluntarily pay royalties together with sales,
//! but note that this standard is not yet widely supported.

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    prelude::{storage, Erase},
    storage::{StorageAddress, StorageMap},
    stylus_proc::SolidityError,
};

use crate::utils::{
    introspection::erc165::IErc165,
    structs::checkpoints::{Size, S160},
};

type U96 = <S160 as Size>::Key;
type StorageU96 = <S160 as Size>::KeyStorage;

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {

        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2981InvalidDefaultRoyalty(uint256 numerator, uint256 denominator);

        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2981InvalidDefaultRoyaltyReceiver(address receiver);

        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2981InvalidTokenRoyalty(uint256 tokenId, uint256 numerator, uint256 denominator);

        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2981InvalidTokenRoyaltyReceiver(uint256 tokenId, address receiver);

    }
}

/// An [`Erc721Royalty`] extension error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// The default royalty set is invalid (eg. (numerator / denominator) >=
    /// 1).
    InvalidDefaultRoyalty(ERC2981InvalidDefaultRoyalty),

    /// The default royalty receiver is invalid.
    InvalidDefaultRoyaltyReceiver(ERC2981InvalidDefaultRoyaltyReceiver),

    /// The royalty set for an specific `tokenId` is invalid (eg. (numerator /
    /// denominator) >= 1).
    InvalidTokenRoyalty(ERC2981InvalidTokenRoyalty),

    /// The royalty receiver for `tokenId` is invalid.
    InvalidTokenRoyaltyReceiver(ERC2981InvalidTokenRoyaltyReceiver),
}

/// Struct for Royalty Information for tokens.
///
/// # Fields
///
/// * `receiver` - The receiver address for royalty
/// * `royalty_fraction` - Fraction of royalty for receiver
#[storage]
#[derive(Erase)]
pub struct RoyaltyInfo {
    receiver: StorageAddress,
    royalty_fraction: StorageU96, // U96 should be used
}

/// State of a Royalty extension.
#[storage]
pub struct Erc721Royalty {
    /// The default royalty information for all tokens.
    #[allow(clippy::used_underscore_binding)]
    pub _default_royalty_info: RoyaltyInfo,
    /// The royalty information for a particular token.
    #[allow(clippy::used_underscore_binding)]
    pub _token_royalty_info: StorageMap<U256, RoyaltyInfo>,
}

// This is the interface of the optional `Royalty` extension
/// of the ERC-721 standard.
#[interface_id]
pub trait IErc721Royalty {
    /// Returns how much royalty is owed and to whom, based on a sale price that
    /// may be denominated in any unit of exchange.
    ///
    /// The royalty amount is denominated and should be paid in that same unit
    /// of exchange.
    ///
    /// NOTE: ERC-2981 allows setting the royalty to 100% of the price.
    /// In that case all the price would be sent to the royalty receiver and 0
    /// tokens to the seller. Contracts dealing with royalty should consider
    /// empty transfers.
    fn royalty_info(&self, token_id: U256, sale_price: U256)
        -> (Address, U256);
}

impl IErc721Royalty for Erc721Royalty {
    fn royalty_info(
        &self,
        token_id: U256,
        sale_price: U256,
    ) -> (Address, U256) {
        let _royalty_info = self._token_royalty_info.get(token_id);
        let mut royalty_receiver = &_royalty_info.receiver;
        let mut royalty_fraction = &_royalty_info.royalty_fraction;

        if royalty_receiver.eq(&Address::ZERO) {
            royalty_receiver = &self._default_royalty_info.receiver;
            royalty_fraction = &self._default_royalty_info.royalty_fraction;
        }

        // Check whether dereferencing impacts anything
        // TODO: Check
        let royalty_amount = (sale_price * U256::from(**royalty_fraction))
            .wrapping_div(U256::from(self._fee_denominator()));

        (**royalty_receiver, royalty_amount)
    }
}

impl IErc165 for Erc721Royalty {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IErc721Royalty>::INTERFACE_ID
            == u32::from_be_bytes(*interface_id)
    }
}

impl Erc721Royalty {
    /// Function to change the denominator with which to interpret the fee set
    /// in _setTokenRoyalty and _setDefaultRoyalty as a fraction of the sale
    /// price Defaults to 10000 so fees are expressed in basis points, but
    /// may be customized by an override.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn _fee_denominator(&self) -> U96 {
        U96::from(10000)
    }

    /// Function to set the royalty information that all ids in this contract
    /// will default to.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `receiver` - Address to receive the royalty.
    /// * `fee_numerator` - Fraction of royalty to be given to receiver
    ///
    /// # Errors
    ///
    /// If `fee_numerator` > denominator, then the error
    /// [`Error::InvalidDefaultRoyalty`] is returned.
    ///
    /// If receiver is the zero address, then the error
    /// [`Error::InvalidDefaultRoyaltyReceiver`] is returned.
    pub fn _set_default_royalty(
        &mut self,
        receiver: Address,
        fee_numerator: U96,
    ) -> Result<(), Error> {
        let denominator: U256 = U256::from(self._fee_denominator());

        if U256::from(fee_numerator) > denominator {
            return Err(Error::InvalidDefaultRoyalty(
                ERC2981InvalidDefaultRoyalty {
                    numerator: U256::from(fee_numerator),
                    denominator,
                },
            ));
        }

        if receiver == Address::ZERO {
            return Err(Error::InvalidDefaultRoyaltyReceiver(
                ERC2981InvalidDefaultRoyaltyReceiver {
                    receiver: Address::ZERO,
                },
            ));
        }

        self._default_royalty_info.receiver.set(receiver);
        self._default_royalty_info.royalty_fraction.set(fee_numerator);

        Ok(())
    }

    /// Function to remove default royalty information.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    pub fn _delete_default_royalty(&mut self) {
        self._default_royalty_info.receiver.set(Address::ZERO);
        self._default_royalty_info.royalty_fraction.set(U96::from(0));
    }

    /// Function to set the royalty information for a specific token id,
    /// overriding the global default.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `receiver` - Address to receive the royalty.
    /// * `fee_numerator` - Fraction of royalty to be given to receiver
    ///
    /// # Errors
    ///
    /// If `fee_numerator` > denominator, then the error
    /// [`Error::InvalidTokenRoyalty`] is returned.
    ///
    /// If receiver is the zero address, then the error
    /// [`Error::InvalidTokenRoyaltyReceiver`] is returned.
    pub fn _set_token_royalty(
        &mut self,
        token_id: U256,
        receiver: Address,
        fee_numerator: U96,
    ) -> Result<(), Error> {
        let denominator: U256 = U256::from(self._fee_denominator());
        if U256::from(fee_numerator) > denominator {
            return Err(Error::InvalidTokenRoyalty(
                ERC2981InvalidTokenRoyalty {
                    tokenId: token_id,
                    numerator: U256::from(fee_numerator),
                    denominator,
                },
            ));
        }

        if receiver == Address::ZERO {
            return Err(Error::InvalidTokenRoyaltyReceiver(
                ERC2981InvalidTokenRoyaltyReceiver {
                    tokenId: token_id,
                    receiver: Address::ZERO,
                },
            ));
        }

        self._token_royalty_info.setter(token_id).receiver.set(receiver);
        self._token_royalty_info
            .setter(token_id)
            .royalty_fraction
            .set(fee_numerator);

        Ok(())
    }

    /// Function to reset royalty information for the token id back to the
    /// global default.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    pub fn _reset_token_royalty(&mut self, token_id: U256) {
        self._token_royalty_info.delete(token_id);
    }
}

mod tests {}
