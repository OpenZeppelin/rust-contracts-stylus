#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::{
    token::erc20::extensions::Erc20Permit, utils::cryptography::eip712::IEip712,
};
use stylus_sdk::prelude::{entrypoint, public, storage};

#[entrypoint]
#[storage]
struct Erc20PermitExample {
    #[borrow]
    pub erc20_permit: Erc20Permit<Eip712>,
}
#[storage]
struct Eip712 {}

impl IEip712 for Eip712 {
    const NAME: &'static str = "ERC-20 Permit Example";
    const VERSION: &'static str = "1";
}

#[public]
#[inherit(Erc20Permit<Eip712>)]
impl Erc20PermitExample {
    // Add token minting feature.
    pub fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        self.erc20_permit.erc20._mint(account, value)?;
        Ok(())
    }
}
