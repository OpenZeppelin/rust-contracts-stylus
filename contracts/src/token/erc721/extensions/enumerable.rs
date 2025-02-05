//! Optional `Enumerable` extension of the ERC-721 standard.
//!
//! This implements an optional extension of [`super::super::Erc721`] defined in
//! the EIP that adds enumerability of all the token ids in the contract as well
//! as all token ids owned by each account.
//!
//! CAUTION: [`super::super::Erc721`] extensions that implement custom
//! [`super::super::Erc721::balance_of`] logic, such as `Erc721Consecutive`,
//! interfere with enumerability and should not be used together with
//! [`Erc721Enumerable`].

use alloc::vec::Vec;

use alloy_primitives::{uint, Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    prelude::storage,
    storage::{StorageMap, StorageU256, StorageVec},
    stylus_proc::{public, SolidityError},
};

use crate::{
    token::erc721::{self, IErc721},
    utils::introspection::erc165::IErc165,
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Indicates an error when an `owner`'s token query
        /// was out of bounds for `index`.
        ///
        /// NOTE: The owner being `Address::ZERO`
        /// indicates a global out of bounds index.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721OutOfBoundsIndex(address owner, uint256 index);

        /// Indicates an error related to batch minting not allowed.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721EnumerableForbiddenBatchMint();
    }
}

/// An [`Erc721Enumerable`] extension error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error when an `owner`'s token query
    /// was out of bounds for `index`.
    ///
    /// NOTE: The owner being `Address::ZERO`
    /// indicates a global out of bounds index.
    OutOfBoundsIndex(ERC721OutOfBoundsIndex),

    /// Indicates an error related to batch minting not allowed.
    EnumerableForbiddenBatchMint(ERC721EnumerableForbiddenBatchMint),
}

/// State of an [`Erc721Enumerable`] contract.
#[storage]
pub struct Erc721Enumerable {
    /// Maps owners to a mapping of indices to tokens ids.
    #[allow(clippy::used_underscore_binding)]
    pub _owned_tokens: StorageMap<Address, StorageMap<U256, StorageU256>>,
    /// Maps tokens ids to indices in `_owned_tokens`.
    #[allow(clippy::used_underscore_binding)]
    pub _owned_tokens_index: StorageMap<U256, StorageU256>,
    /// Stores all tokens ids.
    #[allow(clippy::used_underscore_binding)]
    pub _all_tokens: StorageVec<StorageU256>,
    /// Maps indices at `_all_tokens` to tokens ids.
    #[allow(clippy::used_underscore_binding)]
    pub _all_tokens_index: StorageMap<U256, StorageU256>,
}

/// This is the interface of the optional `Enumerable` extension
/// of the ERC-721 standard.
#[interface_id]
pub trait IErc721Enumerable {
    /// The error type associated to this ERC-721 enumerable trait
    /// implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns a token ID owned by `owner` at a given `index` of its token
    /// list.
    ///
    /// Use along with [`super::super::Erc721::balance_of`] to enumerate all of
    /// `owner`'s tokens.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Address of token's owner.
    /// * `index` - Index of the token at `owner`'s tokens list.
    ///
    /// # Errors
    ///
    /// * [`Error::OutOfBoundsIndex`] - If an `owner`'s token query is out of
    ///   bounds for `index`.
    fn token_of_owner_by_index(
        &self,
        owner: Address,
        index: U256,
    ) -> Result<U256, Self::Error>;

    /// Returns the total amount of tokens stored by the contract.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn total_supply(&self) -> U256;

    /// Returns a token ID at a given `index` of all the tokens
    /// stored by the contract.
    ///
    /// Use along with [`Self::total_supply`] to
    /// enumerate all tokens.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `index` - Index of the token in all tokens list.
    ///
    /// # Errors
    ///
    /// * [`Error::OutOfBoundsIndex`] - If an `owner`'s token query is out of
    ///   bounds for `index`.
    fn token_by_index(&self, index: U256) -> Result<U256, Self::Error>;
}

#[public]
impl IErc721Enumerable for Erc721Enumerable {
    type Error = Error;

    fn token_of_owner_by_index(
        &self,
        owner: Address,
        index: U256,
    ) -> Result<U256, Self::Error> {
        let token = self._owned_tokens.getter(owner).get(index);

        if token.is_zero() {
            Err(ERC721OutOfBoundsIndex { owner, index }.into())
        } else {
            Ok(token)
        }
    }

    fn total_supply(&self) -> U256 {
        let tokens_length = self._all_tokens.len();
        U256::from(tokens_length)
    }

    fn token_by_index(&self, index: U256) -> Result<U256, Self::Error> {
        self._all_tokens.get(index).ok_or(
            ERC721OutOfBoundsIndex { owner: Address::ZERO, index }.into(),
        )
    }
}

impl IErc165 for Erc721Enumerable {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IErc721Enumerable>::INTERFACE_ID
            == u32::from_be_bytes(*interface_id)
    }
}

impl Erc721Enumerable {
    /// Function to add a token to this extension's
    /// ownership-tracking data structures.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Address representing the new owner of the given `token_id`.
    /// * `token_id` - ID of the token to be added to the tokens list of the
    ///   given address.
    /// * `erc721` - Read access to a contract providing [`IErc721`] interface.
    ///
    /// # Errors
    ///
    /// * [`erc721::Error::InvalidOwner`] - If owner address is `Address::ZERO`.
    pub fn _add_token_to_owner_enumeration(
        &mut self,
        to: Address,
        token_id: U256,
        erc721: &impl IErc721<Error = erc721::Error>,
    ) -> Result<(), erc721::Error> {
        let length = erc721.balance_of(to)? - uint!(1_U256);
        self._owned_tokens.setter(to).setter(length).set(token_id);
        self._owned_tokens_index.setter(token_id).set(length);

        Ok(())
    }

    /// Function to add a token to this extension's token
    /// tracking data structures.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token_id` - ID of the token to be added to the tokens list.
    pub fn _add_token_to_all_tokens_enumeration(&mut self, token_id: U256) {
        let index = self.total_supply();

        self._all_tokens_index.setter(token_id).set(index);
        self._all_tokens.push(token_id);
    }

    /// Function to remove a token from this extension's
    /// ownership-tracking data structures.
    ///
    /// Note that while the token is not assigned a new owner,
    /// the `self._owned_tokens_index` mapping is NOT updated:
    /// this allows for  gas optimizations e.g.
    /// when performing a transfer operation (avoiding double writes).
    ///
    /// This has O(1) time complexity, but alters the order
    /// of the `self._owned_tokens` array.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Address representing the previous owner of the given
    ///   `token_id`.
    /// * `token_id` - ID of the token to be removed from the tokens list of the
    ///   given address.
    /// * `erc721` - Read access to a contract providing [`IErc721`] interface.
    ///
    /// # Errors
    ///
    /// * [`erc721::Error::InvalidOwner`] - If owner address is `Address::ZERO`.
    pub fn _remove_token_from_owner_enumeration(
        &mut self,
        from: Address,
        token_id: U256,
        erc721: &impl IErc721<Error = erc721::Error>,
    ) -> Result<(), erc721::Error> {
        // To prevent a gap in from's tokens array,
        // we store the last token in the index of the token to delete,
        // and then delete the last slot (swap and pop).
        let last_token_index = erc721.balance_of(from)?;
        let token_index = self._owned_tokens_index.get(token_id);

        let mut owned_tokens_by_owner = self._owned_tokens.setter(from);

        // When the token to delete is the last token,
        // the swap operation is unnecessary.
        if token_index != last_token_index {
            let last_token_id = owned_tokens_by_owner.get(last_token_index);

            // Move the last token to the slot of the to-delete token.
            owned_tokens_by_owner.setter(token_index).set(last_token_id);
            // Update the moved token's index.
            self._owned_tokens_index.setter(last_token_id).set(token_index);
        }

        // This also deletes the contents at the last position of the array.
        self._owned_tokens_index.delete(token_id);
        owned_tokens_by_owner.delete(last_token_index);

        Ok(())
    }

    /// Function to remove a token from this extension's
    /// token tracking data structures.
    ///
    /// This has O(1) time complexity,
    /// but alters the order of the `self._all_tokens` array.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token_id` -  ID of the token to be removed from the tokens list.
    ///
    /// # Panics
    ///
    /// * The function should not panic in a regular way.
    pub fn _remove_token_from_all_tokens_enumeration(
        &mut self,
        token_id: U256,
    ) {
        // To prevent a gap in the tokens array,
        // we store the last token in the index of the token to delete,
        // and then delete the last slot (swap and pop).
        let last_token_index = U256::from(self._all_tokens.len() - 1);
        let token_index = self._all_tokens_index.get(token_id);

        // When the token to delete is the last token,
        // the swap operation is unnecessary.
        // However, since this occurs so
        // rarely (when the last minted token is burnt)
        // that we still do the swap here
        // to avoid the gas cost of adding an 'if' statement
        // (like in `self._remove_token_from_owner_enumeration`).
        let last_token_id = self
            ._all_tokens
            .get(last_token_index)
            .expect("token at given index must exist");

        // Move the last token to the slot of the to-delete token.
        self._all_tokens
            .setter(token_index)
            .expect("slot at given `token_index` must exist")
            .set(last_token_id);

        // Update the moved token's index.
        self._all_tokens_index.setter(last_token_id).set(token_index);

        // This also deletes the contents at the last position of the array.
        self._all_tokens_index.delete(token_id);
        self._all_tokens.pop();
    }

    /// See [`erc721::Erc721::_increase_balance`].
    /// Check if tokens can be minted in batch.
    ///
    /// Mechanism to be consistent with [Solidity version](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.0.0/contracts/token/ERC721/extensions/ERC721Enumerable.sol#L163-L171)
    ///
    /// # Arguments
    ///
    /// * `amount` - The number of tokens to increase balance.
    ///
    /// # Errors
    ///
    /// * [`Error::EnumerableForbiddenBatchMint`] - If an `amount` is greater
    ///   than `0`.
    pub fn _check_increase_balance(amount: u128) -> Result<(), Error> {
        if amount > 0 {
            Err(ERC721EnumerableForbiddenBatchMint {}.into())
        } else {
            Ok(())
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use stylus_sdk::{
        alloy_primitives::{address, uint, Address, U256},
        msg,
    };

    use super::{Erc721Enumerable, Error, IErc721Enumerable};
    use crate::token::erc721::{Erc721, IErc721};

    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");

    #[motsu::test]
    fn total_supply_no_tokens(contract: Erc721Enumerable) {
        assert_eq!(U256::ZERO, contract.total_supply());
    }

    #[motsu::test]
    fn error_when_token_by_index_is_out_of_bound(contract: Erc721Enumerable) {
        assert_eq!(U256::ZERO, contract.total_supply());

        let token_idx = uint!(2024_U256);

        let err = contract.token_by_index(token_idx).unwrap_err();
        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }

    #[motsu::test]
    fn add_token_to_all_tokens_enumeration_works(contract: Erc721Enumerable) {
        assert_eq!(U256::ZERO, contract.total_supply());

        let tokens_len = 10;

        let mut tokens_ids = Vec::new();
        for token_id in 0..tokens_len {
            let token_id = U256::from(token_id);

            // Store ids for test.
            tokens_ids.push(token_id);

            contract._add_token_to_all_tokens_enumeration(token_id);
        }

        assert_eq!(U256::from(tokens_len), contract.total_supply());

        tokens_ids.iter().enumerate().for_each(|(idx, expected_token_id)| {
            let token_id = contract
                .token_by_index(U256::from(idx))
                .expect("should return token id for");
            assert_eq!(*expected_token_id, token_id);
        });

        let err = contract.token_by_index(U256::from(tokens_len)).unwrap_err();

        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }

    #[motsu::test]
    fn remove_token_from_all_tokens_enumeration_works(
        contract: Erc721Enumerable,
    ) {
        assert_eq!(U256::ZERO, contract.total_supply());

        let initial_tokens_len = 10;

        let mut tokens_ids = Vec::new();
        for token_id in 0..initial_tokens_len {
            let token_id = U256::from(token_id);

            // Store ids for test.
            tokens_ids.push(token_id);

            contract._add_token_to_all_tokens_enumeration(token_id);
        }
        assert_eq!(U256::from(initial_tokens_len), contract.total_supply());

        // Remove the last token.
        let last_token_id = tokens_ids.swap_remove(initial_tokens_len - 1);
        contract._remove_token_from_all_tokens_enumeration(last_token_id);
        assert_eq!(U256::from(initial_tokens_len - 1), contract.total_supply());

        // Remove the second (`idx = 1`) element
        // to check that swap_remove operation works as expected.
        let token_to_remove = tokens_ids.swap_remove(1);
        contract._remove_token_from_all_tokens_enumeration(token_to_remove);
        assert_eq!(U256::from(initial_tokens_len - 2), contract.total_supply());

        // Add a new token.
        let token_id = U256::from(initial_tokens_len);
        tokens_ids.push(token_id);
        contract._add_token_to_all_tokens_enumeration(token_id);
        assert_eq!(U256::from(initial_tokens_len - 1), contract.total_supply());

        // Check proper indices of tokens.
        tokens_ids.iter().enumerate().for_each(|(idx, expected_token_id)| {
            let token_id = contract
                .token_by_index(U256::from(idx))
                .expect("should return token id");
            assert_eq!(*expected_token_id, token_id);
        });

        let err = contract
            .token_by_index(U256::from(initial_tokens_len - 1))
            .unwrap_err();

        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }

    #[motsu::test]
    fn check_increase_balance() {
        assert!(Erc721Enumerable::_check_increase_balance(0).is_ok());
        let err = Erc721Enumerable::_check_increase_balance(1).unwrap_err();
        assert!(matches!(err, Error::EnumerableForbiddenBatchMint(_)));
    }

    #[motsu::test]
    fn token_of_owner_by_index_works(contract: Erc721Enumerable) {
        let alice = msg::sender();
        let mut erc721 = Erc721::default();
        assert_eq!(
            U256::ZERO,
            erc721.balance_of(alice).expect("should return balance of ALICE")
        );

        let token_id = uint!(1_U256);
        erc721._mint(alice, token_id).expect("should mint a token for ALICE");
        let owner = erc721
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);

        let res =
            contract._add_token_to_owner_enumeration(alice, token_id, &erc721);
        assert!(res.is_ok());

        let test_token_id = contract
            .token_of_owner_by_index(alice, U256::ZERO)
            .expect("should return `token_id`");

        assert_eq!(token_id, test_token_id);
    }

    #[motsu::test]
    fn error_when_token_of_owner_for_index_out_of_bound(
        contract: Erc721Enumerable,
    ) {
        let alice = msg::sender();
        let mut erc721 = Erc721::default();
        assert_eq!(
            U256::ZERO,
            erc721.balance_of(alice).expect("should return balance of ALICE")
        );

        let token_id = uint!(1_U256);
        erc721._mint(alice, token_id).expect("should mint a token for ALICE");
        let owner = erc721
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);

        let res =
            contract._add_token_to_owner_enumeration(alice, token_id, &erc721);
        assert!(res.is_ok());

        let err =
            contract.token_of_owner_by_index(alice, uint!(1_U256)).unwrap_err();
        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }

    #[motsu::test]
    fn error_when_token_of_owner_does_not_own_any_token(
        contract: Erc721Enumerable,
    ) {
        let erc721 = Erc721::default();
        assert_eq!(
            U256::ZERO,
            erc721.balance_of(BOB).expect("should return balance of BOB")
        );

        let err =
            contract.token_of_owner_by_index(BOB, U256::ZERO).unwrap_err();
        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }

    #[motsu::test]
    fn token_of_owner_by_index_after_transfer_works(
        contract: Erc721Enumerable,
    ) {
        let alice = msg::sender();
        let mut erc721 = Erc721::default();
        assert_eq!(
            U256::ZERO,
            erc721.balance_of(alice).expect("should return balance of ALICE")
        );

        let token_id = uint!(1_U256);
        erc721._mint(alice, token_id).expect("should mint a token for ALICE");
        let owner = erc721
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);

        let res =
            contract._add_token_to_owner_enumeration(alice, token_id, &erc721);
        assert!(res.is_ok());

        // Transfer the token from ALICE to BOB.
        erc721
            .transfer_from(alice, BOB, token_id)
            .expect("should transfer the token from ALICE to BOB");
        let owner = erc721
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, BOB);

        let res = contract
            ._remove_token_from_owner_enumeration(alice, token_id, &erc721);
        assert!(res.is_ok());

        let res =
            contract._add_token_to_owner_enumeration(BOB, token_id, &erc721);
        assert!(res.is_ok());

        let test_token_id = contract
            .token_of_owner_by_index(BOB, U256::ZERO)
            .expect("should return `token_id`");

        assert_eq!(token_id, test_token_id);

        let err =
            contract.token_of_owner_by_index(alice, U256::ZERO).unwrap_err();
        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc721Enumerable as IErc721Enumerable>::INTERFACE_ID;
        let expected = 0x780e9d63;
        assert_eq!(actual, expected);
    }
}
