#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, B256, U256};
use openzeppelin_stylus::{
    token::erc20::{
        extensions::{permit, Erc20Permit},
        Erc20,
    },
    utils::{cryptography::eip712::IEip712, nonces::Nonces},
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc20PermitExample {
    #[borrow]
    erc20: Erc20,
    #[borrow]
    nonces: Nonces,
    #[borrow]
    erc20_permit: Erc20Permit<Eip712>,
}

#[storage]
struct Eip712;

impl IEip712 for Eip712 {
    const NAME: &'static str = "ERC-20 Permit Example";
    const VERSION: &'static str = "1";
}

#[public]
#[inherit(Erc20, Nonces, Erc20Permit<Eip712>)]
impl Erc20PermitExample {
    // Add token minting feature.
    fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), permit::Error> {
        Ok(self.erc20._mint(account, value)?)
    }

    #[allow(clippy::too_many_arguments)]
    fn permit(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
        deadline: U256,
        v: u8,
        r: B256,
        s: B256,
    ) -> Result<(), permit::Error> {
        self.erc20_permit.permit(
            owner,
            spender,
            value,
            deadline,
            v,
            r,
            s,
            &mut self.erc20,
            &mut self.nonces,
        )
    }
}
