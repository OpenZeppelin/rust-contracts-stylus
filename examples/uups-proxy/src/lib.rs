#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::{
    access::ownable::{self, IOwnable, Ownable},
    proxy::utils::{
        erc1822::IErc1822Proxiable,
        uups_upgradeable::{IUUPSUpgradeable, UUPSUpgradeable},
    },
    token::erc20::{self, Erc20, IErc20},
};
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{Address, B256, U256},
    prelude::*,
};

#[entrypoint]
#[storage]
struct UUPSProxyErc20Example {
    erc20: Erc20,
    ownable: Ownable,
    uups: UUPSUpgradeable,
}

#[public]
#[implements(IErc20<Error = erc20::Error>, IUUPSUpgradeable, IErc1822Proxiable, IOwnable)]
impl UUPSProxyErc20Example {
    #[constructor]
    #[allow(deprecated)]
    fn constructor(
        &mut self,
        initial_owner: Address,
    ) -> Result<(), ownable::Error> {
        self.uups.self_address.set(stylus_sdk::contract::address());
        self.ownable.constructor(initial_owner)
    }

    fn mint(&mut self, to: Address, value: U256) -> Result<(), erc20::Error> {
        self.erc20._mint(to, value)
    }

    /// Initializes the contract.
    ///
    /// NOTE: Make sure to provide a proper initialization in your logic
    /// contract, [`Self::initialize`] should be invoked at most once.
    ///
    /// Ugly hack with setting the `self_address` storage value.
    ///
    /// Stylus SDK doesn't support setting the immutable storage values as
    /// in Solidity:
    ///
    /// ```solidity
    /// address private immutable __self = address(this);
    /// ```
    fn initialize(
        &mut self,
        self_address: Address,
        owner: Address,
    ) -> Result<(), ownable::Error> {
        self.uups.self_address.set(self_address);
        self.ownable.constructor(owner)
    }
}

#[public]
impl IErc20 for UUPSProxyErc20Example {
    type Error = erc20::Error;

    fn balance_of(&self, account: Address) -> U256 {
        self.erc20.balance_of(account)
    }

    fn total_supply(&self) -> U256 {
        self.erc20.total_supply()
    }

    fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.transfer(to, value)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.transfer_from(from, to, value)
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
}

#[public]
impl IUUPSUpgradeable for UUPSProxyErc20Example {
    #[selector(name = "UPGRADE_INTERFACE_VERSION")]
    fn upgrade_interface_version(&self) -> String {
        self.uups.upgrade_interface_version()
    }

    fn upgrade_to_and_call(
        &mut self,
        new_implementation: Address,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        // Make sure to provide upgrade authorization in your implementation
        // contract.
        self.ownable.only_owner()?;
        self.uups.upgrade_to_and_call(new_implementation, data)?;
        Ok(())
    }
}

#[public]
impl IOwnable for UUPSProxyErc20Example {
    fn owner(&self) -> Address {
        self.ownable.owner()
    }

    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Vec<u8>> {
        Ok(self.ownable.transfer_ownership(new_owner)?)
    }

    fn renounce_ownership(&mut self) -> Result<(), Vec<u8>> {
        Ok(self.ownable.renounce_ownership()?)
    }
}

#[public]
impl IErc1822Proxiable for UUPSProxyErc20Example {
    #[selector(name = "proxiableUUID")]
    fn proxiable_uuid(&self) -> Result<B256, Vec<u8>> {
        self.uups.proxiable_uuid()
    }
}
