#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{
    extensions::{flash_mint, Erc20FlashMint, IErc3156FlashLender},
    IErc20,
};
use stylus_sdk::{abi::Bytes, prelude::*};

#[entrypoint]
#[storage]
struct Erc20FlashMintExample {
    flash_mint: Erc20FlashMint,
}

#[public]
#[implements(IErc20<Error = flash_mint::Error>, IErc3156FlashLender<Error = flash_mint::Error>)]
impl Erc20FlashMintExample {
    fn mint(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<(), flash_mint::Error> {
        Ok(self.flash_mint._mint(to, value)?)
    }

    /// WARNING: These functions are intended for **testing purposes** only. In
    /// **production**, ensure strict access control to prevent unauthorized
    /// operations, which can disrupt contract functionality. Remove or secure
    /// these functions before deployment.
    fn set_flash_fee_receiver(&mut self, new_receiver: Address) {
        self.flash_mint.flash_fee_receiver_address.set(new_receiver);
    }

    fn set_flash_fee_value(&mut self, new_value: U256) {
        self.flash_mint.flash_fee_value.set(new_value);
    }
}

#[public]
impl IErc3156FlashLender for Erc20FlashMintExample {
    type Error = flash_mint::Error;

    fn max_flash_loan(&self, token: Address) -> U256 {
        self.flash_mint.max_flash_loan(token)
    }

    fn flash_fee(
        &self,
        token: Address,
        value: U256,
    ) -> Result<U256, flash_mint::Error> {
        Ok(self.flash_mint.flash_fee(token, value)?)
    }

    fn flash_loan(
        &mut self,
        receiver: Address,
        token: Address,
        value: U256,
        data: Bytes,
    ) -> Result<bool, flash_mint::Error> {
        Ok(self.flash_mint.flash_loan(receiver, token, value, data)?)
    }
}

#[public]
impl IErc20 for Erc20FlashMintExample {
    type Error = flash_mint::Error;

    fn total_supply(&self) -> U256 {
        self.flash_mint.total_supply()
    }

    fn balance_of(&self, account: Address) -> U256 {
        self.flash_mint.balance_of(account)
    }

    fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, <Self as IErc20>::Error> {
        Ok(self.flash_mint.transfer(to, value)?)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.flash_mint.allowance(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, <Self as IErc20>::Error> {
        Ok(self.flash_mint.approve(spender, value)?)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, <Self as IErc20>::Error> {
        Ok(self.flash_mint.transfer_from(from, to, value)?)
    }
}
