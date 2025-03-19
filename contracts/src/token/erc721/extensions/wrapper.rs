//! Extension of the ERC-721 token contract to support token wrapping.
//!
//! Users can deposit and withdraw an "underlying token" and receive a "wrapped
//! token" with a matching tokenId. This is useful in conjunction with other
//! modules.
use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
pub use sol::*;
use stylus_sdk::{
    abi::Bytes,
    call::{Call, MethodError},
    contract, msg,
    prelude::*,
    storage::StorageAddress,
};

use crate::token::erc721::{self, Erc721, RECEIVER_FN_SELECTOR};
#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// The received ERC-721 token couldn't be wrapped.
        ///
        /// * `token` - The token address.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721UnsupportedToken(address token);
    }
}

/// An [`Erc721Wrapper`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`Erc721`] contract [`erc721::Error`].
    Erc721(erc721::Error),
    /// The received ERC-721 token couldn't be wrapped.
    UnsupportedToken(ERC721UnsupportedToken),
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

pub use token::IErc721 as IErc721Solidity;
mod token {
    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]
    use alloc::vec;

    stylus_sdk::prelude::sol_interface! {
        /// Interface of the ERC-721 token.
        interface IErc721 {
            function ownerOf(uint256 token_id) external view returns (address);
            function safeTransferFrom(address from, address to, uint256 token_id) external;
            function transferFrom(address from, address to, uint256 token_id) external;
        }
    }
}

/// State of an [`Erc721Wrapper`] token.
#[storage]
pub struct Erc721Wrapper {
    /// Address of the underlying token.
    underlying: StorageAddress,
}

/// ERC-721 Wrapper Standard Interface
pub trait IErc721Wrapper {
    /// The error type associated to the trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Allow a user to deposit underlying tokens and mint the corresponding
    /// tokenIds.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to deposit tokens to.
    /// * `token_ids` - List of underlying token ids to deposit.
    /// * `erc721` - Write access to an [`Erc721`] contract.
    ///
    /// # Errors
    fn deposit_for(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        erc721: &mut Erc721,
    ) -> Result<bool, Self::Error>;

    /// Allow a user to burn wrapped tokens and withdraw the corresponding
    /// tokenIds of the underlying tokens.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to withdraw tokens to.
    /// * `token_ids` - List of underlying token ids to withdraw.
    /// * `erc721` - Write access to an [`Erc721`] contract.
    ///
    /// # Errors
    fn withdraw_to(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        erc721: &mut Erc721,
    ) -> Result<bool, Self::Error>;

    /// Overrides [`erc721::IERC721Receiver::on_erc_721_received`] to allow
    /// minting on direct ERC-721 transfers to this contract.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - The operator of the transfer.
    /// * `from` - The sender of the transfer.
    /// * `token_id` - The token id of the transfer.
    /// * `data` - The data of the transfer.
    /// * `erc721` - Write access to an [`Erc721`] contract.
    ///
    /// # Errors
    fn on_erc721_received(
        &mut self,
        _operator: Address,
        from: Address,
        token_id: U256,
        data: Bytes,
        erc721: &mut Erc721,
    ) -> Result<FixedBytes<4>, Self::Error>;

    /// Returns the underlying token.
    fn underlying(&self) -> Address;
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc721Wrapper {}

impl IErc721Wrapper for Erc721Wrapper {
    type Error = Error;

    fn deposit_for(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        erc721: &mut Erc721,
    ) -> Result<bool, Self::Error> {
        let sender = msg::sender();
        let contract_address = contract::address();
        let underlying = IErc721Solidity::new(self.underlying());

        for token_id in token_ids {
            // This is an "unsafe" transfer that doesn't call any hook on
            // the receiver. With [`IErc721Wrapper::underlying()`] being trusted
            // (by design of this contract) and no other contracts expected to
            // be called from there, we are safe.
            underlying
                .transfer_from(
                    Call::new_in(self),
                    sender,
                    contract_address,
                    token_id,
                )
                .map_err(|e| Error::Erc721(e.into()))?;

            erc721._safe_mint(account, token_id, &vec![].into())?;
        }

        Ok(true)
    }

    fn withdraw_to(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        erc721: &mut Erc721,
    ) -> Result<bool, Self::Error> {
        let sender = msg::sender();
        let underlying = IErc721Solidity::new(self.underlying());

        for token_id in token_ids {
            erc721._update(Address::ZERO, token_id, sender)?;
            underlying
                .safe_transfer_from(
                    Call::new_in(self),
                    contract::address(),
                    account,
                    token_id,
                )
                .map_err(|e| Error::Erc721(e.into()))?;
        }

        Ok(true)
    }

    fn on_erc721_received(
        &mut self,
        _operator: Address,
        from: Address,
        token_id: U256,
        data: Bytes,
        erc721: &mut Erc721,
    ) -> Result<FixedBytes<4>, Error> {
        let sender = msg::sender();
        if self.underlying() != sender {
            return Err(Error::UnsupportedToken(ERC721UnsupportedToken {
                token: sender,
            }));
        }

        erc721._safe_mint(from, token_id, &data)?;

        Ok(RECEIVER_FN_SELECTOR.into())
    }

    fn underlying(&self) -> Address {
        self.underlying.get()
    }
}

impl Erc721Wrapper {
    /// Mints wrapped tokens to cover any underlying tokens that would have been
    /// function that can be exposed with access control if desired.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to mint tokens to.
    /// * `token_id` - A mutable reference to the Erc20 contract.
    ///
    /// # Errors
    fn _recover(
        &mut self,
        account: Address,
        token_id: U256,
        erc721: &mut Erc721,
    ) -> Result<U256, Error> {
        let underlying = IErc721Solidity::new(self.underlying());

        let owner = underlying
            .owner_of(Call::new_in(self), token_id)
            .map_err(|e| Error::Erc721(e.into()))?;

        let contract_address = contract::address();
        if owner != contract_address {
            return Err(Error::Erc721(erc721::Error::IncorrectOwner(
                erc721::ERC721IncorrectOwner {
                    sender: contract_address,
                    token_id,
                    owner,
                },
            )));
        }

        erc721._safe_mint(account, token_id, &vec![].into())?;

        Ok(token_id)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use motsu::prelude::Contract;

    use super::*;
    use crate::token::erc721::IErc721;

    pub(crate) fn random_token_ids(size: usize) -> Vec<U256> {
        (0..size).map(U256::from).collect()
    }

    #[storage]
    struct Erc721WrapperTestExample {
        wrapper: Erc721Wrapper,
        erc721: Erc721,
    }

    #[public]
    impl Erc721WrapperTestExample {
        fn underlying(&self) -> Address {
            self.wrapper.underlying()
        }

        fn deposit_for(
            &mut self,
            account: Address,
            token_ids: Vec<U256>,
        ) -> Result<bool, Error> {
            self.wrapper.deposit_for(account, token_ids, &mut self.erc721)
        }

        // fn withdraw_to(
        //     &mut self,
        //     account: Address,
        //     value: U256,
        // ) -> Result<bool, Error> {
        //     self.wrapper.withdraw_to(account, value, &mut self.erc20)
        // }

        // fn recover(&mut self, account: Address) -> Result<U256, Error> {
        //     self.wrapper._recover(account, &mut self.erc20)
        // }
    }

    unsafe impl TopLevelStorage for Erc721WrapperTestExample {}

    #[motsu::test]
    fn underlying_works(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let erc721_address = erc721_contract.address();

        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc721_address);
        });

        assert_eq!(contract.sender(alice).underlying(), erc721_address);
    }

    #[motsu::test]
    fn deposit_for_works(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let token_ids = random_token_ids(2);

        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc721_contract.address());
        });

        for token_id in &token_ids {
            erc721_contract
                .sender(alice)
                ._mint(alice, *token_id)
                .expect("should mint {token_id} for {alice}");

            erc721_contract
                .sender(alice)
                .approve(contract.address(), *token_id)
                .expect("should approve {token_id} for {contract.address()}");
        }

        assert!(contract
            .sender(alice)
            .deposit_for(alice, token_ids)
            .expect("should deposit"));
    }
}
