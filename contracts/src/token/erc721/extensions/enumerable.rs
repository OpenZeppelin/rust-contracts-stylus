//! Optional `Enumerable` extension of the ERC-721 standard.
//!
//! This implements an optional extension of [`Erc721`] defined in the EIP
//! that adds enumerability of all the token ids in the contract
//! as well as all token ids owned by each account.
//!
//! CAUTION: [`Erc721`] extensions that implement custom
//! [`Erc721::balance_of`] logic, such as [`Erc721Consecutive`], interfere with
//! enumerability and should not be used together with [`Erc721Enumerable`].

use core::marker::PhantomData;

use alloy_primitives::{uint, Address, U256};
use alloy_sol_types::sol;
use openzeppelin_stylus_proc::r#override;
use stylus_proc::{external, sol_storage};
use stylus_sdk::prelude::TopLevelStorage;

use crate::token::erc721::{Erc721, Error, IErc721, IErc721Virtual};

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

sol_storage! {
    /// State of an Enumerable extension.
    #[cfg_attr(all(test, feature = "std"), derive(motsu::DefaultStorageLayout))]
    pub struct Erc721Enumerable<V: IErc721Virtual> {
        /// Maps owners to a mapping of indices to tokens ids.
        mapping(address => mapping(uint256 => uint256)) _owned_tokens;
        /// Maps tokens ids to indices in `_owned_tokens`.
        mapping(uint256 => uint256) _owned_tokens_index;
        /// Stores all tokens ids.
        uint256[] _all_tokens;
        /// Maps indices at `_all_tokens` to tokens ids.
        mapping(uint256 => uint256) _all_tokens_index;
        PhantomData<V> _phantom_data;
    }
}

/// This is the interface of the optional `Enumerable` extension
/// of the ERC-721 standard.
pub trait IErc721Enumerable {
    // TODO: fn supports_interface (#33)

    /// Returns a token ID owned by `owner`
    /// at a given `index` of its token list.
    ///
    /// Use along with [`Erc721::balance_of`]
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
    /// Use along with [`Erc721::total_supply`] to enumerate all tokens.
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
impl<V: IErc721Virtual> IErc721Enumerable for Erc721Enumerable<V> {
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
        self._all_tokens.get(index).ok_or(
            ERC721OutOfBoundsIndex { owner: Address::ZERO, index }.into(),
        )
    }
}

#[r#override]
impl IErc721Virtual for Erc721EnumerableOverride {
    fn _update(
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, crate::token::erc721::Error> {
        let previous_owner =
            Super::_update::<This>(storage, to, token_id, auth)?;
        if previous_owner == Address::ZERO {
            storage
                .inner_mut::<Erc721Enumerable<This>>()
                ._add_token_to_all_tokens_enumeration(token_id);
        } else if previous_owner != to {
            Erc721Enumerable::<This>::_remove_token_from_owner_enumeration(
                storage,
                previous_owner,
                token_id,
            )?;
        }

        if to == Address::ZERO {
            storage
                .inner_mut::<Erc721Enumerable<This>>()
                ._remove_token_from_all_tokens_enumeration(token_id);
        } else {
            Erc721Enumerable::<This>::_add_token_to_owner_enumeration(
                storage, to, token_id,
            )?;
        }

        Ok(previous_owner)
    }
}

impl<V: IErc721Virtual> Erc721Enumerable<V> {
    /// Function to add a token to this extension's
    /// ownership-tracking data structures.
    ///
    /// # Arguments
    ///
    /// * `storage` - Write access to the contract's state.
    /// * `to` - Address representing the new owner of the given `token_id`.
    /// * `token_id` - ID of the token to be added to the tokens list of the
    ///   given address.
    ///
    /// # Errors
    ///
    /// If owner address is `Address::ZERO`, then the error
    /// [`Error::InvalidOwner`] is returned.
    pub fn _add_token_to_owner_enumeration(
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        let balance_of_to = storage.inner::<Erc721<V>>().balance_of(to)?;
        let length = balance_of_to - uint!(1_U256);
        let enumerable = storage.inner_mut::<Erc721Enumerable<V>>();
        enumerable._owned_tokens.setter(to).setter(length).set(token_id);
        enumerable._owned_tokens_index.setter(token_id).set(length);
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

    /// # Arguments
    ///
    /// * `storage` - Write access to the contract's state.
    /// * `from` - Address representing the previous owner of the given
    ///   `token_id`.
    /// * `token_id` - ID of the token to be removed from the tokens list of the
    ///   given address.
    ///
    /// # Errors
    ///
    /// If owner address is `Address::ZERO`, then the error
    /// [`Error::InvalidOwner`] is returned.
    pub fn _remove_token_from_owner_enumeration(
        storage: &mut impl TopLevelStorage,
        from: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        // To prevent a gap in from's tokens array,
        // we store the last token in the index of the token to delete,
        // and then delete the last slot (swap and pop).
        let last_token_index = storage.inner::<Erc721<V>>().balance_of(from)?;
        let enumerable = storage.inner_mut::<Erc721Enumerable<V>>();
        let token_index = enumerable._owned_tokens_index.get(token_id);

        let mut owned_tokens_by_owner = enumerable._owned_tokens.setter(from);

        // When the token to delete is the last token,
        // the swap operation is unnecessary.
        if token_index != last_token_index {
            let last_token_id = owned_tokens_by_owner.get(last_token_index);

            // Move the last token to the slot of the to-delete token.
            owned_tokens_by_owner.setter(token_index).set(last_token_id);
            // Update the moved token's index.
            enumerable
                ._owned_tokens_index
                .setter(last_token_id)
                .set(token_index);
        }

        // This also deletes the contents at the last position of the array.
        enumerable._owned_tokens_index.delete(token_id);
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

    // TODO#q: increase balance should be virtual
    /// See [`Erc721::_increase_balance`].
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
    /// * If an `amount` is greater than `0`, then the error
    /// [`Error::EnumerableForbiddenBatchMint`] is returned.
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
    use std::marker::PhantomData;

    use alloy_primitives::{address, uint, Address, U256};
    use openzeppelin_stylus_proc::r#override;
    use stylus_proc::sol_storage;
    use stylus_sdk::{
        msg,
        prelude::{StorageType, TopLevelStorage},
        storage::{StorageMap, StorageVec},
    };

    use super::{
        Erc721Enumerable, Erc721EnumerableOverride, Error, IErc721Enumerable,
    };
    use crate::token::erc721::{
        tests::random_token_id, Erc721, Erc721Override, IErc721, IErc721Virtual,
    };

    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");

    sol_storage! {
        pub struct Token{
            Erc721<Override> erc721;
            Erc721Enumerable<Override> enumerable;
        }
    }

    #[r#override]
    #[inherit(Erc721EnumerableOverride)]
    #[inherit(Erc721Override)]
    impl IErc721Virtual for TokenOverride {}

    unsafe impl TopLevelStorage for Token {}

    impl Default for Token {
        fn default() -> Self {
            let root = U256::ZERO;

            Token {
                erc721: Erc721 {
                    _owners: unsafe { StorageMap::new(root, 0) },
                    _balances: unsafe {
                        StorageMap::new(root + U256::from(32), 0)
                    },
                    _token_approvals: unsafe {
                        StorageMap::new(root + U256::from(64), 0)
                    },
                    _operator_approvals: unsafe {
                        StorageMap::new(root + U256::from(96), 0)
                    },
                    _phantom_data: PhantomData,
                },
                enumerable: Erc721Enumerable {
                    _owned_tokens: unsafe {
                        StorageMap::new(root + U256::from(128), 0)
                    },
                    _owned_tokens_index: unsafe {
                        StorageMap::new(root + U256::from(160), 0)
                    },
                    _all_tokens: unsafe {
                        StorageVec::new(root + U256::from(192), 0)
                    },
                    _all_tokens_index: unsafe {
                        StorageMap::new(root + U256::from(224), 0)
                    },
                    _phantom_data: PhantomData,
                },
            }
        }
    }

    #[motsu::test]
    fn total_supply_no_tokens(contract: Token) {
        assert_eq!(U256::ZERO, contract.enumerable.total_supply());
    }

    #[motsu::test]
    fn error_when_token_by_index_is_out_of_bound(contract: Token) {
        assert_eq!(U256::ZERO, contract.enumerable.total_supply());

        let token_idx = uint!(2024_U256);

        let err = contract.enumerable.token_by_index(token_idx).unwrap_err();
        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }

    #[motsu::test]
    fn add_token_to_all_tokens_enumeration_works(contract: Token) {
        assert_eq!(U256::ZERO, contract.enumerable.total_supply());

        let tokens_len = 10;

        let mut tokens_ids = Vec::new();
        for _ in 0..tokens_len {
            let token_id = random_token_id();

            // Store ids for test.
            tokens_ids.push(token_id);

            contract.enumerable._add_token_to_all_tokens_enumeration(token_id);
        }

        assert_eq!(U256::from(tokens_len), contract.enumerable.total_supply());

        tokens_ids.iter().enumerate().for_each(|(idx, expected_token_id)| {
            let token_id = contract
                .enumerable
                .token_by_index(U256::from(idx))
                .expect("should return token id for");
            assert_eq!(*expected_token_id, token_id);
        });

        let err = contract
            .enumerable
            .token_by_index(U256::from(tokens_len))
            .unwrap_err();

        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }

    #[motsu::test]
    fn remove_token_from_all_tokens_enumeration_works(contract: Token) {
        assert_eq!(U256::ZERO, contract.enumerable.total_supply());

        let initial_tokens_len = 10;

        let mut tokens_ids = Vec::new();
        for _ in 0..initial_tokens_len {
            let token_id = random_token_id();

            // Store ids for test.
            tokens_ids.push(token_id);

            contract.enumerable._add_token_to_all_tokens_enumeration(token_id);
        }
        assert_eq!(
            U256::from(initial_tokens_len),
            contract.enumerable.total_supply()
        );

        // Remove the last token.
        let last_token_id = tokens_ids.swap_remove(initial_tokens_len - 1);
        contract
            .enumerable
            ._remove_token_from_all_tokens_enumeration(last_token_id);
        assert_eq!(
            U256::from(initial_tokens_len - 1),
            contract.enumerable.total_supply()
        );

        // Remove the second (`idx = 1`) element
        // to check that swap_remove operation works as expected.
        let token_to_remove = tokens_ids.swap_remove(1);
        contract
            .enumerable
            ._remove_token_from_all_tokens_enumeration(token_to_remove);
        assert_eq!(
            U256::from(initial_tokens_len - 2),
            contract.enumerable.total_supply()
        );

        // Add a new token.
        let token_id = random_token_id();
        tokens_ids.push(token_id);
        contract.enumerable._add_token_to_all_tokens_enumeration(token_id);
        assert_eq!(
            U256::from(initial_tokens_len - 1),
            contract.enumerable.total_supply()
        );

        // Check proper indices of tokens.
        tokens_ids.iter().enumerate().for_each(|(idx, expected_token_id)| {
            let token_id = contract
                .enumerable
                .token_by_index(U256::from(idx))
                .expect("should return token id");
            assert_eq!(*expected_token_id, token_id);
        });

        let err = contract
            .enumerable
            .token_by_index(U256::from(initial_tokens_len - 1))
            .unwrap_err();

        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }

    #[motsu::test]
    fn check_increase_balance() {
        assert!(
            Erc721Enumerable::<Override>::_check_increase_balance(0).is_ok()
        );
        let err = Erc721Enumerable::<Override>::_check_increase_balance(1)
            .unwrap_err();
        assert!(matches!(err, Error::EnumerableForbiddenBatchMint(_)));
    }

    #[motsu::test]
    fn token_of_owner_by_index_works(contract: Token) {
        let alice = msg::sender();
        assert_eq!(
            U256::ZERO,
            contract
                .erc721
                .balance_of(alice)
                .expect("should return balance of ALICE")
        );

        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token for ALICE");
        let owner = contract
            .erc721
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);

        let res = Erc721Enumerable::<Override>::_add_token_to_owner_enumeration(
            contract, alice, token_id,
        );
        assert!(res.is_ok());

        let test_token_id = contract
            .enumerable
            .token_of_owner_by_index(alice, U256::ZERO)
            .expect("should return `token_id`");

        assert_eq!(token_id, test_token_id);
    }

    #[motsu::test]
    fn error_when_token_of_owner_for_index_out_of_bound(contract: Token) {
        let alice = msg::sender();
        assert_eq!(
            U256::ZERO,
            contract
                .erc721
                .balance_of(alice)
                .expect("should return balance of ALICE")
        );

        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token for ALICE");
        let owner = contract
            .erc721
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);

        let res = Erc721Enumerable::<Override>::_add_token_to_owner_enumeration(
            contract, alice, token_id,
        );
        assert!(res.is_ok());

        let err = contract
            .enumerable
            .token_of_owner_by_index(alice, uint!(1_U256))
            .unwrap_err();
        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }

    #[motsu::test]
    fn error_when_token_of_owner_does_not_own_any_token(contract: Token) {
        assert_eq!(
            U256::ZERO,
            contract
                .erc721
                .balance_of(BOB)
                .expect("should return balance of BOB")
        );

        let err = contract
            .enumerable
            .token_of_owner_by_index(BOB, U256::ZERO)
            .unwrap_err();
        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }

    #[motsu::test]
    fn token_of_owner_by_index_after_transfer_works(contract: Token) {
        let alice = msg::sender();
        assert_eq!(
            U256::ZERO,
            contract
                .erc721
                .balance_of(alice)
                .expect("should return balance of ALICE")
        );

        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token for ALICE");
        let owner = contract
            .erc721
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);

        let res = Erc721Enumerable::<Override>::_add_token_to_owner_enumeration(
            contract, alice, token_id,
        );
        assert!(res.is_ok());

        // Transfer the token from ALICE to BOB.
        Erc721::<Override>::transfer_from(contract, alice, BOB, token_id)
            .expect("should transfer the token from ALICE to BOB");
        let owner = contract
            .erc721
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, BOB);

        let res =
            Erc721Enumerable::<Override>::_remove_token_from_owner_enumeration(
                contract, alice, token_id,
            );
        assert!(res.is_ok());

        let res = Erc721Enumerable::<Override>::_add_token_to_owner_enumeration(
            contract, BOB, token_id,
        );
        assert!(res.is_ok());

        let test_token_id = contract
            .enumerable
            .token_of_owner_by_index(BOB, U256::ZERO)
            .expect("should return `token_id`");

        assert_eq!(token_id, test_token_id);

        let err = contract
            .enumerable
            .token_of_owner_by_index(alice, U256::ZERO)
            .unwrap_err();
        assert!(matches!(err, Error::OutOfBoundsIndex(_)));
    }
}
