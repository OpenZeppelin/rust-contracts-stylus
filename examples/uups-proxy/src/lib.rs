#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::U32;
use openzeppelin_stylus::{
    access::ownable::{self, IOwnable, Ownable},
    proxy::{
        erc1967,
        utils::{
            erc1822::IErc1822Proxiable,
            uups_upgradeable::{self, IUUPSUpgradeable, UUPSUpgradeable},
        },
    },
    token::erc20::{self, Erc20, IErc20},
    utils::address,
};
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{Address, B256, U256},
    prelude::*,
};

#[derive(SolidityError, Debug)]
enum Error {
    UnauthorizedAccount(ownable::OwnableUnauthorizedAccount),
    InvalidOwner(ownable::OwnableInvalidOwner),
    UnauthorizedCallContext(uups_upgradeable::UUPSUnauthorizedCallContext),
    UnsupportedProxiableUUID(uups_upgradeable::UUPSUnsupportedProxiableUUID),
    InvalidImplementation(erc1967::utils::ERC1967InvalidImplementation),
    InvalidAdmin(erc1967::utils::ERC1967InvalidAdmin),
    InvalidBeacon(erc1967::utils::ERC1967InvalidBeacon),
    NonPayable(erc1967::utils::ERC1967NonPayable),
    EmptyCode(address::AddressEmptyCode),
    FailedCall(address::FailedCall),
    FailedCallWithReason(address::FailedCallWithReason),
    InvalidInitialization(uups_upgradeable::InvalidInitialization),
    InvalidVersion(uups_upgradeable::InvalidVersion),
}

impl From<uups_upgradeable::Error> for Error {
    fn from(e: uups_upgradeable::Error) -> Self {
        match e {
            uups_upgradeable::Error::InvalidImplementation(e) => {
                Error::InvalidImplementation(e)
            }
            uups_upgradeable::Error::InvalidAdmin(e) => Error::InvalidAdmin(e),
            uups_upgradeable::Error::InvalidBeacon(e) => {
                Error::InvalidBeacon(e)
            }
            uups_upgradeable::Error::NonPayable(e) => Error::NonPayable(e),
            uups_upgradeable::Error::EmptyCode(e) => Error::EmptyCode(e),
            uups_upgradeable::Error::FailedCall(e) => Error::FailedCall(e),
            uups_upgradeable::Error::FailedCallWithReason(e) => {
                Error::FailedCallWithReason(e)
            }
            uups_upgradeable::Error::InvalidInitialization(e) => {
                Error::InvalidInitialization(e)
            }
            uups_upgradeable::Error::UnauthorizedCallContext(e) => {
                Error::UnauthorizedCallContext(e)
            }
            uups_upgradeable::Error::UnsupportedProxiableUUID(e) => {
                Error::UnsupportedProxiableUUID(e)
            }
            uups_upgradeable::Error::InvalidVersion(e) => {
                Error::InvalidVersion(e)
            }
        }
    }
}

impl From<ownable::Error> for Error {
    fn from(e: ownable::Error) -> Self {
        match e {
            ownable::Error::UnauthorizedAccount(e) => {
                Error::UnauthorizedAccount(e)
            }
            ownable::Error::InvalidOwner(e) => Error::InvalidOwner(e),
        }
    }
}

#[entrypoint]
#[storage]
struct UUPSProxyErc20Example {
    erc20: Erc20,
    ownable: Ownable,
    uups: UUPSUpgradeable,
}

#[public]
#[implements(IErc20<Error = erc20::Error>, IUUPSUpgradeable, IErc1822Proxiable, IOwnable<Error = ownable::Error>)]
impl UUPSProxyErc20Example {
    // Accepting owner here only to enable invoking functions directly on the
    // UUPS
    #[constructor]
    fn constructor(&mut self, owner: Address) -> Result<(), Error> {
        self.uups.constructor();
        self.ownable.constructor(owner)?;
        Ok(())
    }

    fn mint(&mut self, to: Address, value: U256) -> Result<(), erc20::Error> {
        self.erc20._mint(to, value)
    }

    /// Initializes the contract.
    fn initialize(&mut self, owner: Address) -> Result<(), Error> {
        self.uups.set_version()?;
        self.ownable.constructor(owner)?;
        Ok(())
    }

    fn set_version(&mut self) -> Result<(), Error> {
        Ok(self.uups.set_version()?)
    }

    fn get_version(&self) -> U32 {
        self.uups.get_version()
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

    #[payable]
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
    type Error = ownable::Error;

    fn owner(&self) -> Address {
        self.ownable.owner()
    }

    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Self::Error> {
        self.ownable.transfer_ownership(new_owner)
    }

    fn renounce_ownership(&mut self) -> Result<(), Self::Error> {
        self.ownable.renounce_ownership()
    }
}

#[public]
impl IErc1822Proxiable for UUPSProxyErc20Example {
    #[selector(name = "proxiableUUID")]
    fn proxiable_uuid(&self) -> Result<B256, Vec<u8>> {
        self.uups.proxiable_uuid()
    }
}
