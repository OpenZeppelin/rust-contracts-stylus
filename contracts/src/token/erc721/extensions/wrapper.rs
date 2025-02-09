//! Extension of the ERC-721 token contract to support token wrapping.
//!
//! Users can deposit and withdraw an "underlying token" and receive a "wrapped
//! token" with a matching tokenId. This is useful in conjunction with other
//! modules.
use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, U256};
use stylus_sdk::{
    call::Call,
    contract, msg,
    prelude::storage,
    storage::{StorageAddress, TopLevelStorage},
    stylus_proc::SolidityError,
};

use crate::token::erc721::{
    self, Erc721, IErc721 as IErc721Solidity, RECEIVER_FN_SELECTOR,
};

/// State of an [`Erc721Wrapper`] token.
#[storage]
pub struct Erc721Wrapper {
    /// Erc721 contract storage.
    pub underlying_address: StorageAddress,
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
        ///
        /// * `token_id` - Token id as a number.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721UnsupportedToken(uint256 token_id);

        /// Indicates an error related to the ownership over a particular token.
        /// Used in transfers.
        ///
        /// * `sender` - Address whose tokens are being transferred.
        /// * `token_id` - Token id as a number.
        /// * `owner` - Address of the owner of the token.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721IncorrectOwner(address sender, uint256 token_id, address owner);
    }
}

/// An [`Erc721Wrapper`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`Erc721`] contract [`erc721::Error`].
    Erc721(erc721::Error),
    /// The received ERC-721 token couldn't be wrapped.
    UnsupportedToken(ERC721UnsupportedToken),
    /// Indicates an error related to the ownership over a particular token.
    IncorrectOwner(ERC721IncorrectOwner),
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
    ///
    /// Arguments:
    ///
    /// * `account` - The account to deposit tokens to.
    /// * `token_ids` - The tokenIds of the underlying tokens to deposit.
    /// * `erc721` - A mutable reference to the Erc721 contract.
    pub fn deposit_for(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        erc721: &mut Erc721,
    ) -> Result<bool, Error> {
        let sender = msg::sender();

        token_ids.iter().for_each(|&token_id| {
            self.erc721
                .transfer_from(sender, contract::address(), token_id)
                .expect("transfer failed");
            erc721
                ._safe_mint(account, token_id, &vec![].into())
                .expect("mint failed");
        });

        Ok(true)
    }

    /// Allow a user to burn wrapped tokens and withdraw the corresponding
    /// tokenIds of the underlying tokens.
    ///
    /// Arguments:
    ///
    /// * `account` - The account to withdraw tokens to.
    /// * `token_ids` - The tokenIds of the underlying tokens to withdraw.
    /// * `erc721` - A mutable reference to the Erc721 contract.
    pub fn withdraw_to(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        erc721: &mut Erc721,
    ) -> Result<bool, Error> {
        let sender = msg::sender();

        token_ids.iter().for_each(|&token_id| {
            erc721
                ._update(Address::ZERO, token_id, sender)
                .expect("update failed");
            self.erc721
                .safe_transfer_from(contract::address(), account, token_id)
                .expect("transfer failed");
        });

        Ok(true)
    }

    /// Overrides [`erc721::IERC721Receiver::on_erc_721_received`] to allow
    /// minting on direct ERC-721 transfers to this contract.
    pub fn on_erc721_received(
        &mut self,
        operator: Address,
        from: Address,
        token_id: U256,
        data: &Bytes,
    ) -> Result<(), Error> {
        let sender = msg::sender();

        if self.underlying() != sender {
            return Err(Error::UnsupportedToken(ERC721UnsupportedToken {
                token_id,
            }));
        }

        self.erc721._safe_mint(to, token_id, data);
        // RECEIVER_FN_SELECTOR
        Ok(())
    }

    /// Returns the underlying token.
    pub fn underlying(&self) -> Address {
        self.underlying_address.get()
    }
}

// ************** ERC-721 Internal **************

impl Erc721Wrapper {
    /// Mints wrapped tokens to cover any underlying tokens that would have been
    /// function taht can be exposed with access control if desired.
    ///
    /// Arguments:
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to mint tokens to.
    /// * `token_id` - A mutable reference to the Erc20 contract.
    ///
    /// # Errors
    ///
    /// If the underlying token is not owned by the contract, the error
    /// [`Error::`] is returned.
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
            return Err(Error::IncorrectOwner(ERC721IncorrectOwner {
                sender: contract::address(),
                token_id,
                owner,
            }));
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
