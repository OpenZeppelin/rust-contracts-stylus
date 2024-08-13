#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::{
    token::erc20::extensions::Erc20Permit, utils::cryptography::eip712::IEip712,
};
use stylus_sdk::prelude::{entrypoint, external, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Erc20PermitExample {
        #[borrow]
        Erc20Permit<Eip712> erc20_permit;
    }

    struct Eip712 {}
}

impl IEip712 for Eip712 {
    const NAME: &'static str = "ERC-20 Permit Example";
    const VERSION: &'static str = "1";
}

#[external]
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
