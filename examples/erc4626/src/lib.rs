#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::{
    token::erc20::{
        extensions::{
            erc4626::{Erc4626Internal, Erc4626Storage},
            Erc20MetadataStorage, Erc4626, IErc20Metadata, IErc4626,
        },
        Erc20Internal, Erc20Storage, IErc20,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::{
    alloy_primitives::{aliases::B32, Address, U8},
    prelude::*,
    storage::{
        StorageAddress, StorageMap, StorageString, StorageU256, StorageU8,
    },
};

#[entrypoint]
#[storage]
struct Erc4626Example {
    erc4626: Erc4626,
    decimals_offset: StorageU8,
}

#[public]
#[implements(IErc4626, IErc20, IErc20Metadata)]
impl Erc4626Example {
    #[constructor]
    fn constructor(&mut self, asset: Address, decimals_offset: U8) {
        self.erc4626.constructor(asset);
        self.decimals_offset.set(decimals_offset);
    }
}

#[public]
impl IErc4626 for Erc4626Example {}

#[public]
impl IErc20 for Erc4626Example {}

#[public]
impl IErc20Metadata for Erc4626Example {
    fn decimals(&self) -> U8 {
        self.erc4626.decimals()
    }
}

impl Erc4626Internal for Erc4626Example {
    fn _decimals_offset(&self) -> U8 {
        self.decimals_offset.get()
    }
}

impl Erc4626Storage for Erc4626Example {
    fn asset(&self) -> &StorageAddress {
        Erc4626Storage::asset(&self.erc4626)
    }

    fn asset_mut(&mut self) -> &mut StorageAddress {
        Erc4626Storage::asset_mut(&mut self.erc4626)
    }

    fn underlying_decimals(&self) -> &StorageU8 {
        Erc4626Storage::underlying_decimals(&self.erc4626)
    }

    fn underlying_decimals_mut(&mut self) -> &mut StorageU8 {
        Erc4626Storage::underlying_decimals_mut(&mut self.erc4626)
    }
}

impl Erc20Internal for Erc4626Example {}

impl Erc20Storage for Erc4626Example {
    fn balances(&self) -> &StorageMap<Address, StorageU256> {
        Erc20Storage::balances(&self.erc4626)
    }

    fn balances_mut(&mut self) -> &mut StorageMap<Address, StorageU256> {
        Erc20Storage::balances_mut(&mut self.erc4626)
    }

    fn allowances(
        &self,
    ) -> &StorageMap<Address, StorageMap<Address, StorageU256>> {
        Erc20Storage::allowances(&self.erc4626)
    }

    fn allowances_mut(
        &mut self,
    ) -> &mut StorageMap<Address, StorageMap<Address, StorageU256>> {
        Erc20Storage::allowances_mut(&mut self.erc4626)
    }

    fn total_supply(&self) -> &StorageU256 {
        Erc20Storage::total_supply(&self.erc4626)
    }

    fn total_supply_mut(&mut self) -> &mut StorageU256 {
        Erc20Storage::total_supply_mut(&mut self.erc4626)
    }
}

impl Erc20MetadataStorage for Erc4626Example {
    fn name(&self) -> &StorageString {
        Erc20MetadataStorage::name(&self.erc4626)
    }

    fn symbol(&self) -> &StorageString {
        Erc20MetadataStorage::symbol(&self.erc4626)
    }
}

#[public]
impl IErc165 for Erc4626Example {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.erc4626.supports_interface(interface_id)
    }
}
