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

use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    call::MethodError,
    prelude::*,
    storage::{StorageMap, StorageU256, StorageVec},
};

use crate::{
    token::erc721::{self, IErc721},
    utils::introspection::erc165::{Erc165, IErc165},
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

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of an [`Erc721Enumerable`] contract.
#[storage]
pub struct Erc721Enumerable {
    /// Maps owners to a mapping of indices to tokens ids.
    pub(crate) owned_tokens: StorageMap<Address, StorageMap<U256, StorageU256>>,
    /// Maps tokens ids to indices in `owned_tokens`.
    pub(crate) owned_tokens_index: StorageMap<U256, StorageU256>,
    /// Stores all tokens ids.
    pub(crate) all_tokens: StorageVec<StorageU256>,
    /// Maps indices at `all_tokens` to tokens ids.
    pub(crate) all_tokens_index: StorageMap<U256, StorageU256>,
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
    ) -> Result<U256, <Self as IErc721Enumerable>::Error>;

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
    fn token_by_index(
        &self,
        index: U256,
    ) -> Result<U256, <Self as IErc721Enumerable>::Error>;
}

impl IErc721Enumerable for Erc721Enumerable {
    type Error = Error;

    fn token_of_owner_by_index(
        &self,
        owner: Address,
        index: U256,
    ) -> Result<U256, <Self as IErc721Enumerable>::Error> {
        let token = self.owned_tokens.getter(owner).get(index);

        if token.is_zero() {
            Err(ERC721OutOfBoundsIndex { owner, index }.into())
        } else {
            Ok(token)
        }
    }

    fn total_supply(&self) -> U256 {
        let tokens_length = self.all_tokens.len();
        U256::from(tokens_length)
    }

    fn token_by_index(
        &self,
        index: U256,
    ) -> Result<U256, <Self as IErc721Enumerable>::Error> {
        self.all_tokens.get(index).ok_or(
            ERC721OutOfBoundsIndex { owner: Address::ZERO, index }.into(),
        )
    }
}

impl IErc165 for Erc721Enumerable {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        <Self as IErc721Enumerable>::interface_id() == interface_id
            || Erc165::interface_id() == interface_id
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
        self.owned_tokens.setter(to).setter(length).set(token_id);
        self.owned_tokens_index.setter(token_id).set(length);

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

        self.all_tokens_index.setter(token_id).set(index);
        self.all_tokens.push(token_id);
    }

    /// Function to remove a token from this extension's
    /// ownership-tracking data structures.
    ///
    /// Note that while the token is not assigned a new owner,
    /// the `self.owned_tokens_index` mapping is NOT updated:
    /// this allows for  gas optimizations e.g.
    /// when performing a transfer operation (avoiding double writes).
    ///
    /// This has O(1) time complexity, but alters the order
    /// of the `self.owned_tokens` array.
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
        let token_index = self.owned_tokens_index.get(token_id);

        let mut owned_tokens_by_owner = self.owned_tokens.setter(from);

        // When the token to delete is the last token,
        // the swap operation is unnecessary.
        if token_index != last_token_index {
            let last_token_id = owned_tokens_by_owner.get(last_token_index);

            // Move the last token to the slot of the to-delete token.
            owned_tokens_by_owner.setter(token_index).set(last_token_id);
            // Update the moved token's index.
            self.owned_tokens_index.setter(last_token_id).set(token_index);
        }

        // This also deletes the contents at the last position of the array.
        self.owned_tokens_index.delete(token_id);
        owned_tokens_by_owner.delete(last_token_index);

        Ok(())
    }

    /// Function to remove a token from this extension's
    /// token tracking data structures.
    ///
    /// This has O(1) time complexity,
    /// but alters the order of the `self.all_tokens` array.
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
        let last_token_index = U256::from(self.all_tokens.len() - 1);
        let token_index = self.all_tokens_index.get(token_id);

        // When the token to delete is the last token,
        // the swap operation is unnecessary.
        // However, since this occurs so
        // rarely (when the last minted token is burnt)
        // that we still do the swap here
        // to avoid the gas cost of adding an 'if' statement
        // (like in `self._remove_token_from_owner_enumeration`).
        let last_token_id = self
            .all_tokens
            .get(last_token_index)
            .expect("token at given index must exist");

        // Move the last token to the slot of the to-delete token.
        self.all_tokens
            .setter(token_index)
            .expect("slot at given `token_index` must exist")
            .set(last_token_id);

        // Update the moved token's index.
        self.all_tokens_index.setter(last_token_id).set(token_index);

        // This also deletes the contents at the last position of the array.
        self.all_tokens_index.delete(token_id);
        self.all_tokens.pop();
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
    use motsu::prelude::*;
    use stylus_sdk::prelude::*;

    use super::*;
    use crate::token::erc721::Erc721;

    #[storage]
    struct Erc721EnumerableTestExample {
        pub erc721: Erc721,
        pub enumerable: Erc721Enumerable,
    }

    #[public]
    #[implements(IErc721Enumerable<Error=Error>, IErc165)]
    impl Erc721EnumerableTestExample {}

    #[public]
    impl IErc721Enumerable for Erc721EnumerableTestExample {
        type Error = Error;

        fn total_supply(&self) -> U256 {
            self.enumerable.total_supply()
        }

        fn token_by_index(&self, index: U256) -> Result<U256, Error> {
            Ok(self.enumerable.token_by_index(index)?)
        }

        fn token_of_owner_by_index(
            &self,
            owner: Address,
            index: U256,
        ) -> Result<U256, Error> {
            Ok(self.enumerable.token_of_owner_by_index(owner, index)?)
        }
    }

    #[public]
    impl IErc165 for Erc721EnumerableTestExample {
        fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
            <Erc721EnumerableTestExample as IErc721Enumerable>::interface_id()
                == interface_id
                || Erc165::interface_id() == interface_id
        }
    }

    unsafe impl TopLevelStorage for Erc721EnumerableTestExample {}

    #[motsu::test]
    fn total_supply_no_tokens(
        contract: Contract<Erc721EnumerableTestExample>,
        alice: Address,
    ) {
        assert_eq!(
            U256::ZERO,
            contract.sender(alice).enumerable.total_supply()
        );
    }

    #[motsu::test]
    fn reverts_when_token_by_index_is_out_of_bound(
        contract: Contract<Erc721EnumerableTestExample>,
        alice: Address,
    ) {
        let token_idx = uint!(2024_U256);

        let err = contract
            .sender(alice)
            .enumerable
            .token_by_index(token_idx)
            .expect_err("should return Error::OutOfBoundsIndex");

        assert!(matches!(
            err,
            Error::OutOfBoundsIndex(ERC721OutOfBoundsIndex {
                owner,
                index
            }) if owner.is_zero() && index == token_idx
        ));
    }

    #[motsu::test]
    fn add_token_to_all_tokens_enumeration_works(
        contract: Contract<Erc721EnumerableTestExample>,
        alice: Address,
    ) {
        let tokens_len = 10;

        let mut tokens_ids = Vec::new();
        for token_id in 0..tokens_len {
            let token_id = U256::from(token_id);
            // Store ids for test.
            tokens_ids.push(token_id);
            contract
                .sender(alice)
                .enumerable
                ._add_token_to_all_tokens_enumeration(token_id);
        }

        assert_eq!(
            U256::from(tokens_len),
            contract.sender(alice).enumerable.total_supply()
        );

        tokens_ids.iter().enumerate().for_each(|(idx, expected_token_id)| {
            let token_id = contract
                .sender(alice)
                .enumerable
                .token_by_index(U256::from(idx))
                .expect("should return token id for");
            assert_eq!(*expected_token_id, token_id);
        });

        let err = contract
            .sender(alice)
            .enumerable
            .token_by_index(U256::from(tokens_len))
            .expect_err("should return Error::OutOfBoundsIndex");

        assert!(matches!(
            err,
            Error::OutOfBoundsIndex(ERC721OutOfBoundsIndex {
                owner,
                index
            }) if owner.is_zero() && index == U256::from(tokens_len)
        ));
    }

    #[motsu::test]
    fn remove_token_from_all_tokens_enumeration_works(
        contract: Contract<Erc721EnumerableTestExample>,
        alice: Address,
    ) {
        let initial_tokens_len = 10;

        let mut tokens_ids = Vec::new();
        for token_id in 0..initial_tokens_len {
            let token_id = U256::from(token_id);
            // Store ids for test.
            tokens_ids.push(token_id);

            contract
                .sender(alice)
                .enumerable
                ._add_token_to_all_tokens_enumeration(token_id);
        }
        assert_eq!(
            U256::from(initial_tokens_len),
            contract.sender(alice).enumerable.total_supply()
        );

        // Remove the last token.
        let last_token_id = tokens_ids.swap_remove(initial_tokens_len - 1);
        contract
            .sender(alice)
            .enumerable
            ._remove_token_from_all_tokens_enumeration(last_token_id);
        assert_eq!(
            U256::from(initial_tokens_len - 1),
            contract.sender(alice).enumerable.total_supply()
        );

        // Remove the second (`idx = 1`) element
        // to check that swap_remove operation works as expected.
        let token_to_remove = tokens_ids.swap_remove(1);
        contract
            .sender(alice)
            .enumerable
            ._remove_token_from_all_tokens_enumeration(token_to_remove);
        assert_eq!(
            U256::from(initial_tokens_len - 2),
            contract.sender(alice).enumerable.total_supply()
        );

        // Add a new token.
        let token_id = U256::from(initial_tokens_len);
        tokens_ids.push(token_id);
        contract
            .sender(alice)
            .enumerable
            ._add_token_to_all_tokens_enumeration(token_id);
        assert_eq!(
            U256::from(initial_tokens_len - 1),
            contract.sender(alice).enumerable.total_supply()
        );

        // Check proper indices of tokens.
        tokens_ids.iter().enumerate().for_each(|(idx, expected_token_id)| {
            let token_id = contract
                .sender(alice)
                .enumerable
                .token_by_index(U256::from(idx))
                .expect("should return token id");
            assert_eq!(*expected_token_id, token_id);
        });

        let err = contract
            .sender(alice)
            .enumerable
            .token_by_index(U256::from(initial_tokens_len - 1))
            .expect_err("should return Error::OutOfBoundsIndex");

        assert!(matches!(err, Error::OutOfBoundsIndex(ERC721OutOfBoundsIndex {
                owner,
                index
            }) if owner.is_zero() && index == U256::from(initial_tokens_len - 1)
        ));
    }

    #[motsu::test]
    fn check_increase_balance() {
        assert!(Erc721Enumerable::_check_increase_balance(0).is_ok());

        let err = Erc721Enumerable::_check_increase_balance(1)
            .expect_err("should return Error::EnumerableForbiddenBatchMint");

        assert!(matches!(
            err,
            Error::EnumerableForbiddenBatchMint(
                ERC721EnumerableForbiddenBatchMint {}
            )
        ));
    }

    #[motsu::test]
    fn token_of_owner_by_index_works(
        contract: Contract<Erc721EnumerableTestExample>,
        alice: Address,
    ) {
        let token_id = uint!(1_U256);
        contract
            .sender(alice)
            .erc721
            ._mint(alice, token_id)
            .expect("should mint a token for {{alice}}");

        let owner = contract
            .sender(alice)
            .erc721
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);

        contract
            .sender(alice)
            .enumerable
            ._add_token_to_owner_enumeration(
                alice,
                token_id,
                &contract.sender(alice).erc721,
            )
            .expect("should add token to owner enumeration");

        let test_token_id = contract
            .sender(alice)
            .enumerable
            .token_of_owner_by_index(alice, U256::ZERO)
            .expect("should return `token_id`");

        assert_eq!(token_id, test_token_id);
    }

    #[motsu::test]
    fn reverts_when_token_of_owner_for_index_out_of_bound(
        contract: Contract<Erc721EnumerableTestExample>,
        alice: Address,
    ) {
        let token_id = uint!(1_U256);
        contract
            .sender(alice)
            .erc721
            ._mint(alice, token_id)
            .expect("should mint a token for {{alice}}");

        let owner = contract
            .sender(alice)
            .erc721
            .owner_of(token_id)
            .expect("should return the owner of the token");

        assert_eq!(owner, alice);

        contract
            .sender(alice)
            .enumerable
            ._add_token_to_owner_enumeration(
                alice,
                token_id,
                &contract.sender(alice).erc721,
            )
            .expect("should add token to owner enumeration");

        let token_idx = uint!(1_U256);

        let err = contract
            .sender(alice)
            .enumerable
            .token_of_owner_by_index(alice, token_idx)
            .expect_err("should return Error::OutOfBoundsIndex");

        assert!(matches!(err, Error::OutOfBoundsIndex(ERC721OutOfBoundsIndex {
                owner,
                index
            }) if owner == alice && index == token_idx
        ));
    }

    #[motsu::test]
    fn reverts_when_token_of_owner_does_not_own_any_token(
        contract: Contract<Erc721EnumerableTestExample>,
        alice: Address,
    ) {
        let token_idx = U256::ZERO;

        let err = contract
            .sender(alice)
            .enumerable
            .token_of_owner_by_index(alice, token_idx)
            .expect_err("should return Error::OutOfBoundsIndex");

        assert!(matches!(err, Error::OutOfBoundsIndex(ERC721OutOfBoundsIndex {
                owner,
                index
            }) if owner == alice && index == token_idx
        ));
    }

    #[motsu::test]
    fn token_of_owner_by_index_after_transfer_works(
        contract: Contract<Erc721EnumerableTestExample>,
        alice: Address,
        bob: Address,
    ) {
        let token_id = uint!(1_U256);
        contract
            .sender(alice)
            .erc721
            ._mint(alice, token_id)
            .expect("should mint a token for {{alice}}");

        contract
            .sender(alice)
            .enumerable
            ._add_token_to_owner_enumeration(
                alice,
                token_id,
                &contract.sender(alice).erc721,
            )
            .expect("should add token to owner enumeration");

        // Transfer the token from alice to bob.
        contract
            .sender(alice)
            .erc721
            .transfer_from(alice, bob, token_id)
            .expect("should transfer the token from {{alice}} to {{bob}}");

        // Remove the token from alice's enumeration.
        contract
            .sender(alice)
            .enumerable
            ._remove_token_from_owner_enumeration(
                alice,
                token_id,
                &contract.sender(alice).erc721,
            )
            .expect("should remove token from {{alice}} enumeration");

        contract
            .sender(bob)
            .enumerable
            ._add_token_to_owner_enumeration(
                bob,
                token_id,
                &contract.sender(bob).erc721,
            )
            .expect("should add token to {{bob}} enumeration");

        let token_idx = U256::ZERO;

        let test_token_id = contract
            .sender(bob)
            .enumerable
            .token_of_owner_by_index(bob, token_idx)
            .expect("should return `token_id`");

        assert_eq!(token_id, test_token_id);

        let err = contract
            .sender(alice)
            .enumerable
            .token_of_owner_by_index(alice, token_idx)
            .expect_err("should return Error::OutOfBoundsIndex");

        assert!(matches!(err, Error::OutOfBoundsIndex(ERC721OutOfBoundsIndex {
                owner,
                index
            }) if owner == alice && index == token_idx
        ));
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc721Enumerable as IErc721Enumerable>::interface_id();
        let expected: FixedBytes<4> = 0x780e9d63.into();
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(
        contract: Contract<Erc721EnumerableTestExample>,
        alice: Address,
    ) {
        assert!(contract.sender(alice).enumerable.supports_interface(
            <Erc721Enumerable as IErc721Enumerable>::interface_id()
        ));
        assert!(contract
            .sender(alice)
            .enumerable
            .supports_interface(<Erc721Enumerable as IErc165>::interface_id()));

        let fake_interface_id = 0x12345678u32;
        assert!(!contract
            .sender(alice)
            .enumerable
            .supports_interface(fake_interface_id.into()));
    }
}
