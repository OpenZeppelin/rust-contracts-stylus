//! Extension of ERC-1155 that adds tracking of total supply per id.
//!
//! Useful for scenarios where Fungible and Non-fungible tokens have to be
//! clearly identified. Note: While a totalSupply of 1 might mean the
//! corresponding is an NFT, there is no guarantees that no other token
//! with the same id are not going to be minted.
//!
//! NOTE: This contract implies a global limit of 2**256 - 1 to the number
//! of tokens that can be minted.
//!
//! CAUTION: This extension should not be added in an upgrade to an already
//! deployed contract.
use alloc::vec::Vec;

use alloy_primitives::{uint, Address, U256};
use stylus_proc::sol_storage;
use stylus_sdk::prelude::*;

use crate::{
    token::erc1155::{Erc1155, Error},
    utils::math::storage::SubAssignUnchecked,
};

sol_storage! {
    /// State of [`crate::token::erc1155::Erc1155`] token's supply.
    pub struct Erc1155Supply {
        /// Erc1155 contract storage.
        Erc1155 erc1155;
        /// Mapping from token ID to total supply.
        mapping(uint256 => uint256) _total_supply;
        /// Total supply of all token IDs.
        uint256 _total_supply_all;
    }
}

#[public]
impl Erc1155Supply {
    fn total_supply(&self, token_id: U256) -> U256 {
        self._total_supply.get(token_id)
    }

    fn total_supply_all(&self) -> U256 {
        *self._total_supply_all
    }

    fn exists(&self, token_id: U256) -> bool {
        self.total_supply(token_id) > uint!(0_U256)
    }
}

impl Erc1155Supply {
    /// Override of [`Erc1155::_update`] that restricts normal minting to after
    /// construction.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_ids` - Array of all token id.
    /// * `values` - Array of all amount of tokens to be supplied.
    ///
    /// # Events
    ///
    /// Emits a [`TransferSingle`] event if the arrays contain one element, and
    /// [`TransferBatch`] otherwise.
    fn _update(
        &mut self,
        from: Address,
        to: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Error> {
        self.erc1155._update(from, to, token_ids.clone(), values.clone())?;

        if from.is_zero() {
            let mut total_mint_value = uint!(0_U256);
            token_ids.iter().zip(values.iter()).for_each(
                |(&token_id, &value)| {
                    let total_supply =
                        self.total_supply(token_id).checked_add(value).expect(
                            "should not exceed `U256::MAX` for `_total_supply`",
                        );
                    self._total_supply.setter(token_id).set(total_supply);
                    total_mint_value += value;
                },
            );
            let total_supply_all =
                self.total_supply_all().checked_add(total_mint_value).expect(
                    "should not exceed `U256::MAX` for `_total_supply_all`",
                );
            self._total_supply_all.set(total_supply_all);
        }

        if to.is_zero() {
            let mut total_burn_value = uint!(0_U256);
            token_ids.iter().zip(values.iter()).for_each(
                |(&token_id, &value)| {
                    self._total_supply
                        .setter(token_id)
                        .sub_assign_unchecked(value);
                    total_burn_value += value;
                },
            );
            let total_supply_all =
                self._total_supply_all.get() - total_burn_value;
            self._total_supply_all.set(total_supply_all);
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, Address, U256};

    use super::Erc1155Supply;
    use crate::token::erc1155::IErc1155;

    const ALICE: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");

    pub(crate) fn random_token_ids(size: usize) -> Vec<U256> {
        (0..size).map(|_| U256::from(rand::random::<u32>())).collect()
    }

    pub(crate) fn random_values(size: usize) -> Vec<U256> {
        (0..size).map(|_| U256::from(rand::random::<u128>())).collect()
    }

    fn init(
        contract: &mut Erc1155Supply,
        receiver: Address,
        size: usize,
    ) -> (Vec<U256>, Vec<U256>) {
        let token_ids = random_token_ids(size);
        let values = random_values(size);

        contract
            ._update(Address::ZERO, receiver, token_ids.clone(), values.clone())
            .expect("Supply failed");
        (token_ids, values)
    }

    #[motsu::test]
    fn test_supply_of_zero_supply(contract: Erc1155Supply) {
        let token_ids = random_token_ids(1);
        assert_eq!(U256::ZERO, contract.total_supply(token_ids[0]));
        assert_eq!(U256::ZERO, contract.total_supply_all());
        assert!(!contract.exists(token_ids[0]));
    }

    #[motsu::test]
    fn test_supply_with_zero_address_sender(contract: Erc1155Supply) {
        let token_ids = random_token_ids(1);
        let values = random_values(1);
        contract
            ._update(Address::ZERO, ALICE, token_ids.clone(), values.clone())
            .expect("Supply failed");
        assert_eq!(values[0], contract.total_supply(token_ids[0]));
        assert_eq!(values[0], contract.total_supply_all());
        assert!(contract.exists(token_ids[0]));
    }

    #[motsu::test]
    fn test_supply_with_zero_address_receiver(contract: Erc1155Supply) {
        let (token_ids, values) = init(contract, ALICE, 1);
        contract
            ._update(ALICE, Address::ZERO, token_ids.clone(), values.clone())
            .expect("Supply failed");
        assert_eq!(U256::ZERO, contract.total_supply(token_ids[0]));
        assert_eq!(U256::ZERO, contract.total_supply_all());
        assert!(!contract.exists(token_ids[0]));
    }

    #[motsu::test]
    fn test_supply_batch(contract: Erc1155Supply) {
        let (token_ids, values) = init(contract, BOB, 4);
        assert_eq!(
            values[0],
            contract.erc1155.balance_of(BOB, token_ids[0]).unwrap()
        );
        assert_eq!(
            values[1],
            contract.erc1155.balance_of(BOB, token_ids[1]).unwrap()
        );
        assert_eq!(
            values[2],
            contract.erc1155.balance_of(BOB, token_ids[2]).unwrap()
        );
        assert_eq!(
            values[3],
            contract.erc1155.balance_of(BOB, token_ids[3]).unwrap()
        );
        assert!(contract.exists(token_ids[0]));
        assert!(contract.exists(token_ids[1]));
        assert!(contract.exists(token_ids[2]));
        assert!(contract.exists(token_ids[3]));
    }
}
