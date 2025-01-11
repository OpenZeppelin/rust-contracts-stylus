//! Extension of the ERC-721 token contract to support token wrapping.
//!
//! Users can deposit and withdraw an "underlying token" and receive a "wrapped
//! token" with a matching tokenId. This is useful in conjunction with other
//! modules.
use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, U256};
use stylus_sdk::{
    call::Call,
    contract,
    prelude::storage,
    storage::{StorageAddress, TopLevelStorage},
    stylus_proc::SolidityError,
};

use crate::token::{
    erc721,
    erc721::{ERC721IncorrectOwner, Erc721},
};

/// State of an [`Erc721Wrapper`] token.
#[storage]
pub struct Erc721Wrapper {
    /// Erc721 contract storage.
    pub _underlying: StorageAddress,
    /// The ERC-721 token.
    pub erc721: Erc721,
}

unsafe impl TopLevelStorage for Erc721Wrapper {}

pub use sol::*;
#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// The received ERC-721 token couldn't be wrapped.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721UnsupportedToken(uint256 token_id);
    }
}

/// An [`Erc721Wrapper`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`Erc721`] contract [`erc721::Error`].
    Erc721(erc721::Error),
    /// The received ERC-721 token couldn't be wrapped.
    ERC721UnsupportedToken(ERC721UnsupportedToken),
}

pub use token::IErc721;
mod token {
    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]
    use alloc::vec;

    stylus_sdk::stylus_proc::sol_interface! {
        /// Interface of the ERC-721 token.
        interface IErc721 {
            function ownerOf(uint256 token_id) external view returns (address);
        }
    }
}

impl Erc721Wrapper {
    /// Allow a user to deposit underlying tokens and mint the corresponding
    /// tokenIds.
    pub fn deposit_for(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
    ) -> bool {
        let length = token_ids.len();

        true
    }

    /// Allow a user to burn wrapped tokens and withdraw the corresponding
    /// tokenIds of the underlying tokens.
    pub fn withdraw_to(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
    ) -> bool {
        let length = token_ids.len();

        true
    }

    /// Returns the underlying token.
    pub fn underlying(&self) -> Address {
        self._underlying.get()
    }
}

// ************** ERC-721 Internal **************

impl Erc721Wrapper {
    fn _recover(
        &mut self,
        account: Address,
        token_id: U256,
    ) -> Result<U256, Error> {
        let underlying = IErc721::new(self.underlying());
        let owner = match underlying.owner_of(Call::new_in(self), token_id) {
            Ok(owner) => owner,
            Err(e) => return Err(Error::Erc721(e.into())),
        };
        if owner != contract::address() {
            return Err(erc721::Error::IncorrectOwner(ERC721IncorrectOwner {
                sender: contract::address(),
                token_id,
                owner,
            })
            .into());
        }
        self.erc721._safe_mint(account, token_id, &vec![].into())?;
        Ok(token_id)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, uint, Address, U256};
    use stylus_sdk::msg;

    #[motsu::test]
    fn recover(contract: Erc721Wrapper) {}
}
