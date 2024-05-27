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
use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use stylus_proc::{external, sol_storage, SolidityError};

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
        _owner: Address,
        _index: U256,
    ) -> Result<U256, Error> {
        // TODO
        unimplemented!()
    }

    fn total_supply(&self) -> U256 {
        // TODO
        unimplemented!()
    }

    fn token_by_index(&self, _index: U256) -> Result<U256, Error> {
        // TODO
        unimplemented!()
    }
}

impl ERC721Enumerable {
    fn _add_token_to_owner_enumeration(
        &mut self,
        _to: Address,
        _token_id: U256,
    ) {
        // TODO
        unimplemented!()
    }

    fn _add_token_to_all_tokens_enumeration(&mut self, _token_id: U256) {
        // TODO
        unimplemented!()
    }

    fn _remove_token_from_owner_enumeration(
        &mut self,
        _from: Address,
        _token_id: U256,
    ) {
        // TODO
        unimplemented!()
    }

    fn _remove_token_from_all_tokens_enumeration(&mut self, _token_id: U256) {
        // TODO
        unimplemented!()
    }

    fn _increase_balance(&mut self, _account: Address, _amount: u128) {
        // TODO
        unimplemented!()
    }
}
