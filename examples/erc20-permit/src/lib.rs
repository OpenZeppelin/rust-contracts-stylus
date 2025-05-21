#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, B256, U256};
use openzeppelin_stylus::{
    token::erc20::{
        extensions::{permit, Erc20Permit, IErc20Permit},
        Erc20, IErc20,
    },
    utils::{
        cryptography::eip712::IEip712,
        nonces::{INonces, Nonces},
    },
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc20PermitExample {
    erc20: Erc20,
    nonces: Nonces,
    erc20_permit: Erc20Permit<Eip712>,
}

#[storage]
struct Eip712;

impl IEip712 for Eip712 {
    const NAME: &'static str = "ERC-20 Permit Example";
    const VERSION: &'static str = "1";
}

#[public]
#[implements(IErc20<Error = permit::Error>, INonces, IErc20Permit<Error = permit::Error>)]
impl Erc20PermitExample {
    // Add token minting feature.
    fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), permit::Error> {
        Ok(self.erc20._mint(account, value)?)
    }
}

#[public]
impl IErc20 for Erc20PermitExample {
    type Error = permit::Error;

    fn total_supply(&self) -> U256 {
        self.erc20.total_supply()
    }

    fn balance_of(&self, account: Address) -> U256 {
        self.erc20.balance_of(account)
    }

    fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        Ok(self.erc20.transfer(to, value)?)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        Ok(self.erc20.approve(spender, value)?)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        Ok(self.erc20.transfer_from(from, to, value)?)
    }
}

#[public]
impl INonces for Erc20PermitExample {
    fn nonces(&self, owner: Address) -> U256 {
        self.nonces.nonces(owner)
    }
}

#[public]
impl IErc20Permit for Erc20PermitExample {
    type Error = permit::Error;

    #[selector(name = "DOMAIN_SEPARATOR")]
    fn domain_separator(&self) -> B256 {
        self.erc20_permit.domain_separator()
    }

    fn permit(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
        deadline: U256,
        v: u8,
        r: B256,
        s: B256,
    ) -> Result<(), Self::Error> {
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
