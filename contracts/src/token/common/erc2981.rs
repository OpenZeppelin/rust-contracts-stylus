//! Implementation of the NFT Royalty Standard, a standardized way to retrieve
//! royalty payment information.
//!
//! Royalty information can be specified globally for all token ids via
//! [`Erc2981::_set_default_royalty`], and/or individually for specific token
//! ids via [`Erc2981::_set_token_royalty`]. The latter takes precedence over
//! the first.
//!
//! Royalty is specified as a fraction of sale price.
//! [`Erc2981::fee_denominator`] is overridable but defaults to 10000, meaning
//! the fee is specified in basis points by default.
//!
//! IMPORTANT: ERC-2981 only specifies a way to signal royalty information and
//! does not enforce its payment.
//!
//! See <https://eips.ethereum.org/EIPS/eip-2981#optional-royalty-payments> in the ERC.
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
    introspection::erc165::{Erc165, IErc165},
    structs::checkpoints::{Size, S160},
};

type U96 = <S160 as Size>::Key;
type StorageU96 = <S160 as Size>::KeyStorage;

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Indicates an error for Invalid Default Royalty. Occurs when
        /// `numerator` > `denominator`.
        ///
        /// * `numerator` - Numerator in fraction of royalty.
        /// * `denomimator` - Denominator in fraction of royalty.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2981InvalidDefaultRoyalty(uint256 numerator, uint256 denominator);

        /// Indicates an error relating to default royalty receiver being invalid.
        /// The zero address is considered invalid.
        ///
        /// * `receiver` - Address to which royalty is sent.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2981InvalidDefaultRoyaltyReceiver(address receiver);

        /// Indicates an error for Invalid Token Royalty. Occurs when
        /// `numerator` > `denominator`.
        ///
        /// * `token_id` - Id of a token.
        /// * `numerator` - Numerator in fraction of royalty.
        /// * `denomimator` - Denominator in fraction of royalty.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2981InvalidTokenRoyalty(uint256 token_id, uint256 numerator, uint256 denominator);

        /// Indicates an error relating to token royalty receiver being invalid.
        /// The zero address is considered invalid.
        ///
        /// * `token_id` - Id of a token.
        /// * `receiver` - Address to which royalty is sent.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2981InvalidTokenRoyaltyReceiver(uint256 token_id, address receiver);

    }
}

/// An [`Erc2981`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error for Invalid Default Royalty. Occurs when
    /// `numerator` > `denominator`.
    InvalidDefaultRoyalty(ERC2981InvalidDefaultRoyalty),

    /// Indicates an error relating to default royalty receiver being invalid.
    /// The zero address is considered invalid.
    InvalidDefaultRoyaltyReceiver(ERC2981InvalidDefaultRoyaltyReceiver),

    /// Indicates an error for Invalid Token Royalty. Occurs when
    /// `numerator` > `denominator`.
    InvalidTokenRoyalty(ERC2981InvalidTokenRoyalty),

    /// Indicates an error relating to token royalty receiver being invalid.
    /// The zero address is considered invalid.
    InvalidTokenRoyaltyReceiver(ERC2981InvalidTokenRoyaltyReceiver),
}

/// Struct for Royalty Information of tokens.
#[storage]
#[derive(Erase)]
pub struct RoyaltyInfo {
    /// The receiver address for royalty.
    receiver: StorageAddress,
    /// Fraction of royalty for receiver.
    royalty_fraction: StorageU96,
}

/// State of an [`Erc2981`] contract.
#[storage]
pub struct Erc2981 {
    /// The default royalty information for all tokens.
    pub(crate) default_royalty_info: RoyaltyInfo,
    /// The royalty information for a particular token.
    pub(crate) token_royalty_info: StorageMap<U256, RoyaltyInfo>,
    /// The fee denominator for royalty fraction.
    /// Should be set in the constructor.
    pub fee_denominator: StorageU96,
}

/// Interface for the NFT Royalty Standard.
///
/// A standardized way to retrieve royalty payment information for non-fungible
/// tokens (NFTs) to enable universal support for royalty payments across all
/// NFT marketplaces and ecosystem participants.
#[interface_id]
pub trait IErc2981: IErc165 {
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
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Id of a token.
    /// * `sale_price` - The sale price of the token.
    ///
    /// # Panics
    ///
    /// * If `sale_price` * `royalty_fraction` overflows.
    /// * If [`Erc2981::_fee_denominator()`] is zero.
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
        let (royalty_receiver, royalty_fraction) =
            if royalty_info.receiver.is_zero() {
                (
                    &self.default_royalty_info.receiver,
                    &self.default_royalty_info.royalty_fraction,
                )
            } else {
                (&royalty_info.receiver, &royalty_info.royalty_fraction)
            };

        let royalty_amount = sale_price
            .checked_mul(U256::from(royalty_fraction.get()))
            .expect(
                "multiplication overflowed in `royalty_amount` calculation.",
            )
            .checked_div(U256::from(self._fee_denominator()))
            .expect("division by zero in `royalty_amount` calculation.");

        (royalty_receiver.get(), royalty_amount)
    }
}

impl IErc165 for Erc2981 {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IErc2981>::INTERFACE_ID == u32::from_be_bytes(*interface_id)
            || Erc165::supports_interface(interface_id)
    }
}

impl Erc2981 {
    /// Fetches the denominator with which to interpret the fee set
    /// in [`Self::_set_token_royalty`] and [`Self::_set_default_royalty`] as a
    /// fraction of the sale price.
    ///
    /// Defaults to 10000 so fees are expressed in basis points, but
    /// may be customized in the constructor.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn _fee_denominator(&self) -> U96 {
        self.fee_denominator.get()
    }

    /// Sets the royalty information that all ids in this contract
    /// will default to.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `receiver` - Address to receive the royalty.
    /// * `fee_numerator` - Fraction of royalty to be given to receiver.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidDefaultRoyalty`] - If `fee_numerator` > denominator.
    /// * [`Error::InvalidDefaultRoyaltyReceiver`] - If `receiver` is
    ///   `Address::ZERO`.
    pub fn _set_default_royalty(
        &mut self,
        receiver: Address,
        fee_numerator: U96,
    ) -> Result<(), Error> {
        let denominator = self._fee_denominator();

        if fee_numerator > denominator {
            return Err(Error::InvalidDefaultRoyalty(
                ERC2981InvalidDefaultRoyalty {
                    numerator: U256::from(fee_numerator),
                    denominator: U256::from(denominator),
                },
            ));
        }

        if receiver.is_zero() {
            return Err(Error::InvalidDefaultRoyaltyReceiver(
                ERC2981InvalidDefaultRoyaltyReceiver { receiver },
            ));
        }

        self.default_royalty_info.receiver.set(receiver);
        self.default_royalty_info.royalty_fraction.set(fee_numerator);

        Ok(())
    }

    /// Removes default royalty information.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    pub fn _delete_default_royalty(&mut self) {
        self.default_royalty_info.erase();
    }

    /// Sets the royalty information for a specific `token_id`,
    /// overriding the global default.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token_id` - Id of a token.
    /// * `receiver` - Address to receive the royalty.
    /// * `fee_numerator` - Fraction of royalty to be given to receiver.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidTokenRoyalty`] - If `fee_numerator` >
    ///   [`Self::_fee_denominator()`].
    /// * [`Error::InvalidTokenRoyaltyReceiver`] - If `receiver` is
    ///   `Address::ZERO`.
    pub fn _set_token_royalty(
        &mut self,
        token_id: U256,
        receiver: Address,
        fee_numerator: U96,
    ) -> Result<(), Error> {
        let denominator = self._fee_denominator();
        if fee_numerator > denominator {
            return Err(Error::InvalidTokenRoyalty(
                ERC2981InvalidTokenRoyalty {
                    token_id,
                    numerator: U256::from(fee_numerator),
                    denominator: U256::from(denominator),
                },
            ));
        }

        if receiver.is_zero() {
            return Err(Error::InvalidTokenRoyaltyReceiver(
                ERC2981InvalidTokenRoyaltyReceiver { token_id, receiver },
            ));
        }

        let mut token_royalty_info = self.token_royalty_info.setter(token_id);
        token_royalty_info.receiver.set(receiver);
        token_royalty_info.royalty_fraction.set(fee_numerator);

        Ok(())
    }

    /// Resets royalty information for the token id back to the
    /// global default.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token_id` - Id of the token.
    pub fn _reset_token_royalty(&mut self, token_id: U256) {
        self.token_royalty_info.delete(token_id);
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use motsu::prelude::Contract;
    use stylus_sdk::alloy_primitives::{uint, Address, U256};

    use super::*;
    use crate::{
        token::common::erc2981::IErc2981, utils::introspection::erc165::IErc165,
    };

    const FEE_NUMERATOR: U96 = uint!(9000_U96);
    const TOKEN_ID: U256 = uint!(1_U256);
    const SALE_PRICE: U256 = uint!(1000_U256);
    const DEFAULT_FEE_DENOMINATOR: U96 = uint!(10000_U96);

    fn calculate_royalty_fraction(
        royalty_fraction: U96,
        sale_price: U256,
        fee_denominator: U96,
    ) -> U256 {
        (U256::from(royalty_fraction) * sale_price)
            / U256::from(fee_denominator)
    }

    // DEFAULT ROYALTY TESTS
    #[motsu::test]
    fn fee_denominator_is_set_in_constructor(
        contract: Contract<Erc2981>,
        bob: Address,
    ) {
        let fee_denominator = uint!(6000_U96);
        contract.init(bob, |contract| {
            contract.fee_denominator.set(fee_denominator);
        });

        assert_eq!(contract.sender(bob)._fee_denominator(), fee_denominator);
    }

    #[motsu::test]
    fn set_default_royalty(contract: Contract<Erc2981>, bob: Address) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(DEFAULT_FEE_DENOMINATOR);
        });

        let new_fraction = uint!(8000_U96);

        contract
            .sender(bob)
            ._set_default_royalty(bob, new_fraction)
            .expect("should update default royalty");

        let (received_address, received_royalty_fraction) =
            contract.sender(bob).royalty_info(TOKEN_ID, SALE_PRICE);

        assert_eq!(bob, received_address);
        assert_eq!(
            calculate_royalty_fraction(
                new_fraction,
                SALE_PRICE,
                DEFAULT_FEE_DENOMINATOR
            ),
            received_royalty_fraction
        );
    }

    #[motsu::test]
    fn set_default_royalty_reverts_if_invalid_receiver(
        contract: Contract<Erc2981>,
        bob: Address,
    ) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(DEFAULT_FEE_DENOMINATOR);
        });

        let err = contract
            .sender(bob)
            ._set_default_royalty(Address::ZERO, FEE_NUMERATOR)
            .expect_err("should return `Error::InvalidDefaultRoyaltyReceiver`");

        assert!(
            matches!(err, Error::InvalidDefaultRoyaltyReceiver(ERC2981InvalidDefaultRoyaltyReceiver{
            receiver
        }) if receiver.is_zero())
        );
    }

    #[motsu::test]
    fn set_default_royalty_reverts_if_invalid_fee_numerator(
        contract: Contract<Erc2981>,
        bob: Address,
    ) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(DEFAULT_FEE_DENOMINATOR);
        });

        let new_fee_numerator = uint!(11000_U96);

        let err = contract
            .sender(bob)
            ._set_default_royalty(bob, new_fee_numerator)
            .expect_err("should return `Error::InvalidDefaultRoyalty`");

        assert!(
            matches!(err, Error::InvalidDefaultRoyalty(ERC2981InvalidDefaultRoyalty{
            numerator,
            denominator
        }) if numerator == U256::from(new_fee_numerator) &&
            denominator == U256::from(DEFAULT_FEE_DENOMINATOR))
        );
    }

    #[motsu::test]
    fn default_royalty_is_same_for_all_tokens(
        contract: Contract<Erc2981>,
        bob: Address,
    ) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(DEFAULT_FEE_DENOMINATOR);
        });

        let token_id_2 = uint!(2_U256);

        contract
            .sender(bob)
            ._set_default_royalty(bob, FEE_NUMERATOR)
            .expect("should update default royalty");

        let (received_address, received_royalty_fraction) =
            contract.sender(bob).royalty_info(TOKEN_ID, SALE_PRICE);

        let (received_address_2, received_royalty_fraction_2) =
            contract.sender(bob).royalty_info(token_id_2, SALE_PRICE);

        assert_eq!(received_address, received_address_2);
        assert_eq!(received_royalty_fraction, received_royalty_fraction_2);
    }

    #[motsu::test]
    fn delete_default_royalty(contract: Contract<Erc2981>, bob: Address) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(DEFAULT_FEE_DENOMINATOR);
        });

        contract
            .sender(bob)
            ._set_default_royalty(bob, FEE_NUMERATOR)
            .expect("should set default royalty");

        contract.sender(bob)._delete_default_royalty();

        let (received_address, received_royalty_fraction) =
            contract.sender(bob).royalty_info(TOKEN_ID, SALE_PRICE);

        assert!(received_address.is_zero());
        assert!(received_royalty_fraction.is_zero());
    }

    // TOKEN ROYALTY TESTS

    #[motsu::test]
    fn set_token_royalty(contract: Contract<Erc2981>, bob: Address) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(DEFAULT_FEE_DENOMINATOR);
        });

        let new_fraction = uint!(8000_U96);

        contract
            .sender(bob)
            ._set_token_royalty(TOKEN_ID, bob, new_fraction)
            .expect("should update token royalty");

        let (received_address, received_royalty_fraction) =
            contract.sender(bob).royalty_info(TOKEN_ID, SALE_PRICE);

        assert_eq!(bob, received_address);
        assert_eq!(
            U256::from(new_fraction) * SALE_PRICE
                / U256::from(contract.sender(bob)._fee_denominator()),
            received_royalty_fraction
        );
    }

    #[motsu::test]
    fn token_royalty_is_different_for_different_tokens(
        contract: Contract<Erc2981>,
        bob: Address,
        dave: Address,
    ) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(DEFAULT_FEE_DENOMINATOR);
        });

        let token_id_2 = uint!(2_U256);
        let new_fraction = uint!(8000_U96);

        contract
            .sender(bob)
            ._set_token_royalty(TOKEN_ID, bob, FEE_NUMERATOR)
            .expect("should update token royalty");

        contract
            .sender(bob)
            ._set_token_royalty(token_id_2, dave, new_fraction)
            .expect("should update token royalty");

        let (received_address, received_royalty_fraction) =
            contract.sender(bob).royalty_info(TOKEN_ID, SALE_PRICE);

        assert_eq!(bob, received_address);
        assert_eq!(
            calculate_royalty_fraction(
                FEE_NUMERATOR,
                SALE_PRICE,
                DEFAULT_FEE_DENOMINATOR
            ),
            received_royalty_fraction
        );

        let (received_address_2, received_royalty_fraction_2) =
            contract.sender(bob).royalty_info(token_id_2, SALE_PRICE);

        assert_eq!(dave, received_address_2);
        assert_eq!(
            calculate_royalty_fraction(
                new_fraction,
                SALE_PRICE,
                DEFAULT_FEE_DENOMINATOR
            ),
            received_royalty_fraction_2
        );
    }

    #[motsu::test]
    fn set_token_royalty_reverts_if_invalid_receiver(
        contract: Contract<Erc2981>,
        bob: Address,
    ) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(DEFAULT_FEE_DENOMINATOR);
        });

        let err = contract
            .sender(bob)
            ._set_token_royalty(TOKEN_ID, Address::ZERO, FEE_NUMERATOR)
            .expect_err("should return `Error::InvalidTokenRoyaltyReceiver`");

        assert!(
            matches!(err, Error::InvalidTokenRoyaltyReceiver(ERC2981InvalidTokenRoyaltyReceiver{
            token_id,
            receiver
        }) if token_id == TOKEN_ID && receiver.is_zero())
        );
    }

    #[motsu::test]
    fn set_token_royalty_reverts_if_invalid_fee_numerator(
        contract: Contract<Erc2981>,
        bob: Address,
    ) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(DEFAULT_FEE_DENOMINATOR);
        });

        let new_fee_numerator = uint!(11000_U96);

        let err = contract
            .sender(bob)
            ._set_token_royalty(TOKEN_ID, bob, new_fee_numerator)
            .expect_err("should return `Error::InvalidTokenRoyalty`");

        assert!(
            matches!(err, Error::InvalidTokenRoyalty(ERC2981InvalidTokenRoyalty{
            token_id,
            numerator,
            denominator
        }) if token_id == TOKEN_ID && numerator == U256::from(new_fee_numerator) &&
            denominator == U256::from(DEFAULT_FEE_DENOMINATOR))
        );
    }

    #[motsu::test]
    fn reset_token_royalty(
        contract: Contract<Erc2981>,
        bob: Address,
        dave: Address,
    ) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(DEFAULT_FEE_DENOMINATOR);
        });

        let new_fraction = uint!(8000_U96);

        contract
            .sender(bob)
            ._set_default_royalty(bob, FEE_NUMERATOR)
            .expect("should set default royalty");

        contract
            .sender(bob)
            ._set_token_royalty(TOKEN_ID, dave, new_fraction)
            .expect("should set token royalty");

        contract.sender(bob)._reset_token_royalty(TOKEN_ID);

        let (received_address, received_royalty_fraction) =
            contract.sender(bob).royalty_info(TOKEN_ID, SALE_PRICE);

        assert_eq!(bob, received_address);
        assert_eq!(
            calculate_royalty_fraction(
                FEE_NUMERATOR,
                SALE_PRICE,
                DEFAULT_FEE_DENOMINATOR
            ),
            received_royalty_fraction
        );
    }

    #[motsu::test]
    #[should_panic = "division by zero in `royalty_amount` calculation."]
    fn royalty_info_reverts_on_division_by_zero_in_default_royalty(
        contract: Contract<Erc2981>,
        bob: Address,
    ) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(U96::ZERO);
        });

        contract
            .sender(bob)
            ._set_default_royalty(bob, U96::ZERO)
            .expect("should set default royalty");

        // should revert on division by zero.
        _ = contract.sender(bob).royalty_info(TOKEN_ID, SALE_PRICE);
    }

    #[motsu::test]
    #[should_panic = "multiplication overflowed in `royalty_amount` calculation."]
    fn royalty_info_reverts_on_overflow_in_default_royalty(
        contract: Contract<Erc2981>,
        bob: Address,
    ) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(DEFAULT_FEE_DENOMINATOR);
        });

        contract
            .sender(bob)
            ._set_default_royalty(bob, FEE_NUMERATOR)
            .expect("should set default royalty");
        // should overflow.
        _ = contract.sender(bob).royalty_info(TOKEN_ID, U256::MAX);
    }

    #[motsu::test]
    #[should_panic = "division by zero in `royalty_amount` calculation."]
    fn royalty_info_reverts_on_division_by_zero_in_token_royalty(
        contract: Contract<Erc2981>,
        bob: Address,
    ) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(U96::ZERO);
        });

        contract
            .sender(bob)
            ._set_token_royalty(TOKEN_ID, bob, U96::ZERO)
            .expect("should set default royalty");

        // should revert on division by zero.
        _ = contract.sender(bob).royalty_info(TOKEN_ID, SALE_PRICE);
    }
    #[motsu::test]
    #[should_panic = "multiplication overflowed in `royalty_amount` calculation."]
    fn royalty_info_reverts_on_overflow_in_token_royalty(
        contract: Contract<Erc2981>,
        bob: Address,
    ) {
        contract.init(bob, |contract| {
            contract.fee_denominator.set(DEFAULT_FEE_DENOMINATOR);
        });

        contract
            .sender(bob)
            ._set_token_royalty(TOKEN_ID, bob, FEE_NUMERATOR)
            .expect("should set token royalty");
        // should overflow.
        _ = contract.sender(bob).royalty_info(TOKEN_ID, U256::MAX);
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc2981 as IErc2981>::INTERFACE_ID;
        // Value taken from official EIP
        // https://eips.ethereum.org/EIPS/eip-2981#checking-if-the-nft-being-sold-on-your-marketplace-implemented-royalties
        let expected = 0x2a55_205a;
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface() {
        assert!(Erc2981::supports_interface(
            <Erc2981 as IErc2981>::INTERFACE_ID.into()
        ));
        assert!(Erc2981::supports_interface(
            <Erc2981 as IErc165>::INTERFACE_ID.into()
        ));
    }
}
