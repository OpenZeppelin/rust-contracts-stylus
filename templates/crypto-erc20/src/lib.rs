//! Template project for `openzeppelin-stylus` & `openzeppelin-crypto`.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use openzeppelin_crypto::{
    field::{instance::FpBN256, prime::PrimeField},
    poseidon2::{instance::bn256::BN256Params, Poseidon2},
};
use openzeppelin_stylus::{
    token::erc20::{
        self,
        extensions::{Erc20Metadata, IErc20Burnable, IErc20Metadata},
        Erc20, IErc20,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::{
    alloy_primitives::{aliases::B32, Address, U256, U8},
    prelude::*,
};

#[entrypoint]
#[storage]
struct CryptoErc20 {
    erc20: Erc20,
    metadata: Erc20Metadata,
}

#[public]
#[implements(IErc20<Error = erc20::Error>, IErc20Burnable<Error = erc20::Error>, IErc20Metadata, IErc165)]
impl CryptoErc20 {
    #[constructor]
    pub fn constructor(
        &mut self,
        name: String,
        symbol: String,
    ) -> Result<(), Vec<u8>> {
        self.metadata.constructor(name, symbol);
        Ok(())
    }

    pub fn mint(&mut self, to: Address, value: U256) -> Result<(), Vec<u8>> {
        self.erc20._mint(to, value)?;
        Ok(())
    }

    pub fn poseidon_hash(
        &mut self,
        inputs: [alloy_primitives::U256; 2],
    ) -> alloy_primitives::U256 {
        let mut hasher = Poseidon2::<BN256Params, FpBN256>::new();

        for input in inputs.iter() {
            let fp = FpBN256::from_bigint(
                openzeppelin_crypto::arithmetic::uint::U256::from(*input),
            );
            hasher.absorb(&fp);
        }

        let hash = hasher.squeeze();
        hash.into_bigint().into()
    }
}

#[public]
impl IErc20 for CryptoErc20 {
    type Error = erc20::Error;

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
        self.erc20.transfer(to, value)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.approve(spender, value)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.transfer_from(from, to, value)
    }
}

#[public]
impl IErc20Burnable for CryptoErc20 {
    type Error = erc20::Error;

    fn burn(&mut self, value: U256) -> Result<(), Self::Error> {
        self.erc20.burn(value)
    }

    fn burn_from(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        self.erc20.burn_from(account, value)
    }
}

#[public]
impl IErc20Metadata for CryptoErc20 {
    fn name(&self) -> String {
        self.metadata.name()
    }

    fn symbol(&self) -> String {
        self.metadata.symbol()
    }

    fn decimals(&self) -> U8 {
        self.metadata.decimals()
    }
}

#[public]
impl IErc165 for CryptoErc20 {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.erc20.supports_interface(interface_id)
            || self.metadata.supports_interface(interface_id)
    }
}

#[cfg(test)]
mod tests {}
