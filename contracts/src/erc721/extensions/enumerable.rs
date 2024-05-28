//! Optional `Enumerable` extension of the [`ERC-721`] standard.
//!
//! This implements an optional extension of [`ERC-721`] defined in the EIP
//! that adds enumerability of all the token ids in the contract
//! as well as all token ids owned by each account.
//!
//! CAUTION: [`ERC721`] extensions that implement
//! custom [`ERC721::balance_of`] logic, such as [`ERC721Consecutive`],
//! interfere with enumerability and should not be used together
//! with [`ERC721Enumerable`].
use alloy_primitives::{private::derive_more::From, Address, U256};
use alloy_sol_types::sol;
use stylus_proc::{external, sol_storage, SolidityError};

use crate::erc721::IERC721;

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

/// An [`ERC721Enumerable`] extension error.
#[derive(SolidityError, Debug, From)]
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

sol_storage! {
    /// State of an Enumerable extension.
    pub struct ERC721Enumerable {
        /// TODO
        mapping(address => mapping(uint256 => uint256)) _owned_tokens;
        /// TODO
        mapping(uint256 => uint256) _owned_tokens_index;
        /// TODO
        uint256[] _all_tokens;
        /// TODO
        mapping(uint256 => uint256) _all_tokens_index;
    }

}

/// This is an interface of the optional `Enumerable` extension
/// of the [`ERC721`] standard.
#[allow(clippy::module_name_repetitions)]
pub trait IERC721Enumerable {
    // TODO: fn supports_interface (#33)

    /// Returns a token ID owned by `owner`
    /// at a given `index` of its token list.
    ///
    /// Use along with [`ERC721::balance_of`]
    /// to enumerate all of `owner`'s tokens.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * If an `owner`'s token query is out of bounds for `index`,
    /// then the error [`Error::OutOfBoundsIndex`] is returned.
    fn token_of_owner_by_index(
        &self,
        owner: Address,
        index: U256,
    ) -> Result<U256, Error>;

    /// Returns the total amount of tokens stored by the contract.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn total_supply(&self) -> U256;

    /// Returns a token ID at a given `index` of all the tokens
    /// stored by the contract.
    ///
    /// Use along with [`ERC721::total_supply`] to enumerate all tokens.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * If an `owner`'s token query is out of bounds for `index`,
    /// then the error [`Error::OutOfBoundsIndex`] is returned.
    fn token_by_index(&self, index: U256) -> Result<U256, Error>;
}

#[external]
impl IERC721Enumerable for ERC721Enumerable {
    fn token_of_owner_by_index(
        &self,
        owner: Address,
        index: U256,
    ) -> Result<U256, Error> {
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

    fn token_by_index(&self, index: U256) -> Result<U256, Error> {
        match self._all_tokens.get(index) {
            Some(token_id) => Ok(token_id),
            None => {
                Err(ERC721OutOfBoundsIndex { owner: Address::ZERO, index }
                    .into())
            }
        }
    }
}

impl ERC721Enumerable {
    /// Function to add a token to this extension's
    /// ownership-tracking data structures.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Address representing the new owner of the given `token_id`.
    /// * `token_id` - ID of the token to be added to the tokens list of the
    ///   given address.
    /// * `erc721` - Read access to a contract providing [`IERC721`] interface.
    ///
    /// # Panics
    ///
    /// * The function should not panic in a regular way.
    pub fn _add_token_to_owner_enumeration(
        &mut self,
        to: Address,
        token_id: U256,
        erc721: &impl IERC721,
    ) {
        let length =
            erc721.balance_of(to).expect("`from` cannot be `Address::ZERO`")
                - U256::from(1);
        self._owned_tokens.setter(to).setter(length).set(token_id);
        self._owned_tokens_index.setter(token_id).set(length);
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

    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Address representing the previous owner of the given
    ///   `token_id`.
    /// * `token_id` - ID of the token to be removed from the tokens list of the
    ///   given address.
    /// * `erc721` - Read access to a contract providing [`IERC721`] interface.
    ///
    /// # Panics
    ///
    /// * The function should not panic in a regular way.
    pub fn _remove_token_from_owner_enumeration(
        &mut self,
        from: Address,
        token_id: U256,
        erc721: &impl IERC721,
    ) {
        // To prevent a gap in from's tokens array,
        // we store the last token in the index of the token to delete,
        // and then delete the last slot (swap and pop).
        let last_token_index =
            erc721.balance_of(from).expect("`from` cannot be `Address::ZERO`");
        let token_index = self._owned_tokens_index.get(token_id);

        let mut owned_tokens_by_owner = self._owned_tokens.setter(from);

        // When the token to delete is the last token,
        // the swap operation is unnecessary
        if token_index != last_token_index {
            let last_token_id = owned_tokens_by_owner.get(last_token_index);

            // Move the last token to the slot of the to-delete token
            owned_tokens_by_owner.setter(token_index).set(last_token_id);
            // Update the moved token's index
            self._owned_tokens_index.setter(last_token_id).set(token_index);
        }

        // This also deletes the contents at the last position of the array
        self._owned_tokens_index.delete(token_id);
        owned_tokens_by_owner.delete(last_token_index);
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
        // (like in `self._remove_token_from_owner_enumeration`)
        let last_token_id = self
            ._all_tokens
            .get(last_token_index)
            .expect("Token at given index must exist");

        // Move the last token to the slot of the to-delete token
        self._all_tokens
            .setter(token_index)
            .expect("Slot at given `token_index` must exist")
            .set(last_token_id);

        // Update the moved token's index
        self._all_tokens_index.setter(last_token_id).set(token_index);

        // This also deletes the contents at the last position of the array
        self._all_tokens_index.delete(token_id);
        self._all_tokens.pop();
    }

    /// See [`ERC721::_increase_balance`].
    /// Check if tokens can be minted in batch.
    ///
    /// # Arguments
    ///
    /// * `amount` - The number of tokens to increase balance.
    ///
    /// # Errors
    ///
    /// * If an `amount` is greater than `0`,
    /// then the error [`Error::EnumerableForbiddenBatchMint`] is returned.
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
    use alloy_primitives::U256;
    use stylus_sdk::{
        prelude::StorageType,
        storage::{StorageMap, StorageVec},
    };

    use super::{ERC721Enumerable, Error, IERC721Enumerable};
    use crate::erc721::tests::random_token_id;

    impl Default for ERC721Enumerable {
        fn default() -> Self {
            let root = U256::ZERO;

            ERC721Enumerable {
                _owned_tokens: unsafe { StorageMap::new(root, 0) },
                _owned_tokens_index: unsafe {
                    StorageMap::new(root + U256::from(32), 0)
                },
                _all_tokens: unsafe {
                    StorageVec::new(root + U256::from(64), 0)
                },
                _all_tokens_index: unsafe {
                    StorageMap::new(root + U256::from(96), 0)
                },
            }
        }
    }

    #[grip::test]
    fn total_supply_no_tokens(contract: ERC721Enumerable) {
        assert_eq!(U256::ZERO, contract.total_supply());
    }

    #[grip::test]
    fn token_by_index_errors_when_index_out_of_bound(
        contract: ERC721Enumerable,
    ) {
        assert_eq!(U256::ZERO, contract.total_supply());

        let token_idx = U256::from(2024);

        let err = contract.token_by_index(token_idx).unwrap_err();
        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }

    #[grip::test]
    fn add_token_to_all_tokens_enumeration_works(contract: ERC721Enumerable) {
        assert_eq!(U256::ZERO, contract.total_supply());

        let tokens_len = 10;

        let mut tokens_ids = Vec::new();
        for _ in 0..tokens_len {
            let token_id = random_token_id();

            // store ids for test
            tokens_ids.push(token_id);

            contract._add_token_to_all_tokens_enumeration(token_id);
        }

        assert_eq!(U256::from(tokens_len), contract.total_supply());

        for idx in 0..tokens_len {
            let token_id = contract
                .token_by_index(U256::from(idx))
                .expect("Should return token id for");
            assert_eq!(tokens_ids[idx], token_id);
        }

        let err = contract.token_by_index(U256::from(tokens_len)).unwrap_err();

        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }
}
