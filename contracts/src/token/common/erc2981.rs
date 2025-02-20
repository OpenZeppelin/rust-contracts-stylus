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

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    prelude::{public, storage, Erase, TopLevelStorage},
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
        /// Indicates an error for Invalid Default Royalty. Occurs when
        /// `fee_numerator` > `denominator`.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2981InvalidDefaultRoyalty(uint256 numerator, uint256 denominator);

        /// Indicates an error relating to default royalty receiver being invalid.
        /// The zero address is considered invalid.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2981InvalidDefaultRoyaltyReceiver(address receiver);

        /// Indicates an error for Invalid Token Royalty. Occurs when
        /// fee_numerator > denominator
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2981InvalidTokenRoyalty(uint256 tokenId, uint256 numerator, uint256 denominator);

        /// Indicates an error relating to token royalty receiver being invalid.
        /// The zero address is considered invalid.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2981InvalidTokenRoyaltyReceiver(uint256 tokenId, address receiver);

    }
}

/// An [`Erc2981`] error.
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
    royalty_fraction: StorageU96,
}

/// State of an ERC2981 contract.
#[storage]
pub struct Erc2981 {
    /// The default royalty information for all tokens.
    pub(crate) default_royalty_info: RoyaltyInfo,
    /// The royalty information for a particular token.
    /// It is a mapping of `token_id` to `RoyaltyInfo`.
    pub(crate) token_royalty_info: StorageMap<U256, RoyaltyInfo>,
}

/// Interface for the NFT Royalty Standard.

/// A standardized way to retrieve royalty payment information for non-fungible
/// tokens (NFTs) to enable universal support for royalty payments across all
/// NFT marketplaces and ecosystem participants.
#[interface_id]
pub trait IErc2981 {
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
    ///
    /// # Arguments
    ///
    /// * `&self` - Write access to the contract's state
    /// * `token_id` - Token id of the token
    /// * `sale_price` - The sale price of the token
    fn royalty_info(&self, token_id: U256, sale_price: U256)
        -> (Address, U256);
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc2981 {}

#[public]
impl IErc2981 for Erc2981 {
    fn royalty_info(
        &self,
        token_id: U256,
        sale_price: U256,
    ) -> (Address, U256) {
        let royalty_info = self.token_royalty_info.get(token_id);
        let mut royalty_receiver = &royalty_info.receiver;
        let mut royalty_fraction = &royalty_info.royalty_fraction;

        if royalty_receiver.is_zero() {
            royalty_receiver = &self.default_royalty_info.receiver;
            royalty_fraction = &self.default_royalty_info.royalty_fraction;
        }

        let royalty_amount = (sale_price * U256::from(royalty_fraction.get()))
            .wrapping_div(U256::from(self._fee_denominator()));

        (royalty_receiver.get(), royalty_amount)
    }
}

impl IErc165 for Erc2981 {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IErc2981>::INTERFACE_ID == u32::from_be_bytes(*interface_id)
    }
}

impl Erc2981 {
    /// Function to change the denominator with which to interpret the fee set
    /// in _setTokenRoyalty and _setDefaultRoyalty as a fraction of the sale
    /// price  
    /// Defaults to 10000 so fees are expressed in basis points, but
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

        if receiver.is_zero() {
            return Err(Error::InvalidDefaultRoyaltyReceiver(
                ERC2981InvalidDefaultRoyaltyReceiver {
                    receiver: Address::ZERO,
                },
            ));
        }

        self.default_royalty_info.receiver.set(receiver);
        self.default_royalty_info.royalty_fraction.set(fee_numerator);

        Ok(())
    }

    /// Function to remove default royalty information.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    pub fn _delete_default_royalty(&mut self) {
        self.default_royalty_info.receiver.set(Address::ZERO);
        self.default_royalty_info.royalty_fraction.set(U96::from(0));
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
    /// If `receiver` is the zero address, then the error
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

        if receiver.is_zero() {
            return Err(Error::InvalidTokenRoyaltyReceiver(
                ERC2981InvalidTokenRoyaltyReceiver {
                    tokenId: token_id,
                    receiver: Address::ZERO,
                },
            ));
        }

        self.token_royalty_info.setter(token_id).receiver.set(receiver);
        self.token_royalty_info
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
        self.token_royalty_info.delete(token_id);
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use motsu::prelude::Contract;
    use stylus_sdk::alloy_primitives::{address, uint, Address, U256};

    use super::{Erc2981, Error};
    use crate::{
        token::common::erc2981::IErc2981,
        utils::{
            introspection::erc165::IErc165,
            structs::checkpoints::{Size, S160},
        },
    };

    type U96 = <S160 as Size>::Key;
    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");
    const DAVE: Address = address!("0BB78F7e7132d1651B4Fd884B7624394e92156F1");
    const ZERO_ADDRESS: Address = Address::ZERO;
    const FEE_NUMERATOR: U96 = uint!(9000_U96);
    const TOKEN_ID: U256 = uint!(1_U256);
    const SALE_PRICE: U256 = uint!(1000_U256);

    /// DEFAULT ROYALTY TESTS

    #[motsu::test]
    fn check_whether_update_default_royalty_works(contract: Contract<Erc2981>) {
        let new_fraction = uint!(8000_U96);

        contract
            .sender(BOB)
            ._set_default_royalty(BOB, new_fraction)
            .expect("Error in updating default royalty");

        let (received_address, received_royalty_fraction) =
            contract.sender(BOB).royalty_info(TOKEN_ID, SALE_PRICE);

        assert_eq!(BOB, received_address);
        assert_eq!(
            U256::from(new_fraction) * SALE_PRICE
                / U256::from(contract.sender(BOB)._fee_denominator()),
            received_royalty_fraction
        );
    }

    #[motsu::test]
    fn check_whether_default_royalty_is_same_for_all_tokens(
        contract: Contract<Erc2981>,
    ) {
        let token_id_2 = uint!(2_U256);

        contract
            .sender(BOB)
            ._set_default_royalty(BOB, FEE_NUMERATOR)
            .expect("Error in updating default royalty");

        let (received_address, received_royalty_fraction) =
            contract.sender(BOB).royalty_info(TOKEN_ID, SALE_PRICE);

        let (received_address_2, received_royalty_fraction_2) =
            contract.sender(BOB).royalty_info(token_id_2, SALE_PRICE);

        assert_eq!(received_address, received_address_2);
        assert_eq!(received_royalty_fraction, received_royalty_fraction_2);
    }

    #[motsu::test]
    fn check_whether_delete_default_royalty_works(contract: Contract<Erc2981>) {
        contract
            .sender(BOB)
            ._set_default_royalty(BOB, FEE_NUMERATOR)
            .expect("Error in setting default royalty");

        contract.sender(BOB)._delete_default_royalty();

        let (received_address, received_royalty_fraction) =
            contract.sender(BOB).royalty_info(TOKEN_ID, SALE_PRICE);

        assert_eq!(ZERO_ADDRESS, received_address);
        assert_eq!(uint!(0_U256), received_royalty_fraction);
    }

    #[motsu::test]
    fn check_whether_reverts_if_invalid_parameters(
        contract: Contract<Erc2981>,
    ) {
        let err = contract
            .sender(BOB)
            ._set_default_royalty(Address::ZERO, FEE_NUMERATOR)
            .unwrap_err();

        assert!(matches!(err, Error::InvalidDefaultRoyaltyReceiver(_)));

        let new_fee_numerator = uint!(11000_U96);

        let err = contract
            .sender(BOB)
            ._set_default_royalty(BOB, new_fee_numerator)
            .unwrap_err();

        assert!(matches!(err, Error::InvalidDefaultRoyalty(_)));
    }

    // TOKEN ROYALTY TESTS

    #[motsu::test]
    fn check_whether_update_token_royalty_works(contract: Contract<Erc2981>) {
        let new_fraction = uint!(8000_U96);

        contract
            .sender(BOB)
            ._set_token_royalty(TOKEN_ID, BOB, new_fraction)
            .expect("Error in updating token royalty");

        let (received_address, received_royalty_fraction) =
            contract.sender(BOB).royalty_info(TOKEN_ID, SALE_PRICE);

        assert_eq!(BOB, received_address);
        assert_eq!(
            U256::from(new_fraction) * SALE_PRICE
                / U256::from(contract.sender(BOB)._fee_denominator()),
            received_royalty_fraction
        );
    }

    #[motsu::test]
    fn check_whether_token_royalty_is_different_for_different_tokens(
        contract: Contract<Erc2981>,
    ) {
        let token_id_2 = uint!(2_U256);
        let new_fraction = uint!(8000_U96);

        contract
            .sender(BOB)
            ._set_token_royalty(TOKEN_ID, BOB, FEE_NUMERATOR)
            .expect("Error in updating token royalty");

        contract
            .sender(BOB)
            ._set_token_royalty(token_id_2, DAVE, new_fraction)
            .expect("Error in updating token royalty");

        let (received_address, received_royalty_fraction) =
            contract.sender(BOB).royalty_info(TOKEN_ID, SALE_PRICE);

        let (received_address_2, received_royalty_fraction_2) =
            contract.sender(BOB).royalty_info(token_id_2, SALE_PRICE);

        assert_ne!(received_address, received_address_2);
        assert_ne!(received_royalty_fraction, received_royalty_fraction_2);
    }

    #[motsu::test]
    fn check_whether_reset_token_royalty_works(contract: Contract<Erc2981>) {
        let new_fraction = uint!(8000_U96);

        contract
            .sender(BOB)
            ._set_default_royalty(BOB, FEE_NUMERATOR)
            .expect("Error in setting default royalty");

        contract
            .sender(BOB)
            ._set_token_royalty(TOKEN_ID, DAVE, new_fraction)
            .expect("Error in setting token royalty");

        contract.sender(BOB)._reset_token_royalty(TOKEN_ID);

        let (received_address, received_royalty_fraction) =
            contract.sender(BOB).royalty_info(TOKEN_ID, SALE_PRICE);

        assert_eq!(BOB, received_address);
        assert_eq!(
            U256::from(FEE_NUMERATOR) * SALE_PRICE
                / U256::from(contract.sender(BOB)._fee_denominator()),
            received_royalty_fraction
        );
    }

    #[motsu::test]
    fn check_whether_token_royalty_reverts_if_invalid_parameters(
        contract: Contract<Erc2981>,
    ) {
        let err = contract
            .sender(BOB)
            ._set_token_royalty(TOKEN_ID, Address::ZERO, FEE_NUMERATOR)
            .unwrap_err();

        assert!(matches!(err, Error::InvalidTokenRoyaltyReceiver(_)));

        let new_fee_numerator = uint!(11000_U96);

        let err = contract
            .sender(BOB)
            ._set_token_royalty(TOKEN_ID, BOB, new_fee_numerator)
            .unwrap_err();

        assert!(matches!(err, Error::InvalidTokenRoyalty(_)));
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc2981 as IErc2981>::INTERFACE_ID;
        let expected = 0x2a55_205a;
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface() {
        assert!(<Erc2981 as IErc165>::supports_interface(0x2a55_205a.into()));
    }
}
