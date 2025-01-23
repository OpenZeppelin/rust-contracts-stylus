// TODO: Write Documentation

//! Implementation of the NFT Royalty Standard, a standardized way to retrieve royalty payment information.
//!
//! Royalty information can be specified globally for all token ids via {_setDefaultRoyalty}, and/or individually for
//! specific token ids via {_setTokenRoyalty}. The latter takes precedence over the first.
//!
//! Royalty is specified as a fraction of sale price. {_feeDenominator} is overridable but defaults to 10000, meaning the
//! fee is specified in basis points by default.
//!
//! IMPORTANT: ERC-2981 only specifies a way to signal royalty information and does not enforce its payment. See
//! https://eips.ethereum.org/EIPS/eip-2981#optional-royalty-payments[Rationale] in the ERC. Marketplaces are expected to
//! voluntarily pay royalties together with sales, but note that this standard is not yet widely supported.

use alloy_primitives::{FixedBytes, U256, Address};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    prelude::{storage, Erase},
    storage::{StorageAddress, StorageMap},
    stylus_proc::SolidityError,
};  

use crate::utils::{introspection::erc165::IErc165, structs::checkpoints::{Size, S160}};

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
    /// The default royalty set is invalid (eg. (numerator / denominator) >= 1).
    InvalidDefaultRoyalty(ERC2981InvalidDefaultRoyalty),

    /// The default royalty receiver is invalid.
    InvalidDefaultRoyaltyReceiver(ERC2981InvalidDefaultRoyaltyReceiver),
    
    /// The royalty set for an specific `tokenId` is invalid (eg. (numerator / denominator) >= 1).
    InvalidTokenRoyalty(ERC2981InvalidTokenRoyalty),
    
    /// The royalty receiver for `tokenId` is invalid.
    InvalidTokenRoyaltyReceiver(ERC2981InvalidTokenRoyaltyReceiver),
}

#[storage]
#[derive(Erase)]
struct RoyaltyInfo {
    receiver: StorageAddress,
    royalty_fraction: StorageU96, // U96 should be used
}

/// State of a Royalty extension.
#[storage]
pub struct Erc721Royalty {
    
    #[allow(clippy::used_underscore_binding)]
    pub _default_royalty_info: RoyaltyInfo, 

    #[allow(clippy::used_underscore_binding)]
    pub _token_royalty_info: StorageMap<U256, RoyaltyInfo>,
}

#[interface_id]
pub trait IErc721Royalty {
    fn royalty_info(&self, token_id: U256, sale_price: U256) -> (Address, U256);
}

impl IErc721Royalty for Erc721Royalty {
    fn royalty_info(&self, token_id: U256, sale_price: U256) -> (Address, U256) {
        let _royalty_info = self._token_royalty_info.get(token_id);
        let mut royalty_receiver  = &_royalty_info.receiver;
        let mut royalty_fraction = &_royalty_info.royalty_fraction;

        if royalty_receiver.eq(&Address::ZERO) {
            royalty_receiver = &self._default_royalty_info.receiver;
            royalty_fraction = &self._default_royalty_info.royalty_fraction;
        }

        // Check whether dereferencing impacts anything
        // TODO: Check
        let royalty_amount = (sale_price * U256::from(**royalty_fraction)).wrapping_div(U256::from(self._fee_denominator()));

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
    //? &self or &mut self ??

    pub fn _fee_denominator(&self) -> U96 {
        return U96::from(10000);
    }

    pub fn _set_default_royalty(&mut self, receiver: Address, fee_numerator: U96) -> Result<(), Error>{
        let denominator: U256 = U256::from(self._fee_denominator());
        
        if U256::from(fee_numerator) > denominator {
            return Err(Error::InvalidDefaultRoyalty(ERC2981InvalidDefaultRoyalty{
                numerator: U256::from(fee_numerator),
                denominator: denominator,
            }));
        }

        if receiver == Address::ZERO {
            return Err(Error::InvalidDefaultRoyaltyReceiver(ERC2981InvalidDefaultRoyaltyReceiver{
                receiver: Address::ZERO,
            }));
        }

        self._default_royalty_info.receiver.set(receiver);
        self._default_royalty_info.royalty_fraction.set(fee_numerator);

        Ok(())
    }

    pub fn _delete_default_royalty(&mut self) {
        self._default_royalty_info.receiver.set(Address::ZERO);
        self._default_royalty_info.royalty_fraction.set(U96::from(0));
    }

    pub fn _set_token_royalty(&mut self, token_id: U256, receiver: Address, fee_numerator: U96) -> Result<(), Error>{
        let denominator:U256 = U256::from(self._fee_denominator());
        if U256::from(fee_numerator) > denominator {
            return Err(Error::InvalidTokenRoyalty(ERC2981InvalidTokenRoyalty{
                tokenId: token_id,
                numerator: U256::from(fee_numerator),
                denominator: denominator,
            }));
        }

        if receiver == Address::ZERO {
            return Err(Error::InvalidTokenRoyaltyReceiver(ERC2981InvalidTokenRoyaltyReceiver{
                tokenId: token_id,
                receiver: Address::ZERO,       
            }));
        }

        self._token_royalty_info.setter(token_id).receiver.set(receiver);
        self._token_royalty_info.setter(token_id).royalty_fraction.set(fee_numerator);

        Ok(())
    }

    pub fn _reset_token_royalty(&mut self ,token_id: U256) {
        self._token_royalty_info.delete(token_id);
    }
}

mod tests {}