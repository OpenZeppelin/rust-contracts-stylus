//! An upgradeability mechanism designed for UUPS (Universal Upgradeable Proxy
//! Standard) proxies as defined in [ERC-1822].
//!
//! [ERC-1822]: https://eips.ethereum.org/EIPS/eip-1822
//!
//! The functions included here can perform an upgrade of an
//! [`Erc1967Proxy`], when this contract is set as the implementation
//! behind such a proxy.
//!
//! [`Erc1967Proxy`]: crate::proxy::erc1967::Erc1967Proxy
pub use alloc::{string::String, vec, vec::Vec};

use alloy_primitives::{aliases::B256, Address};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    abi::Bytes,
    call::{Call, MethodError},
    contract,
    prelude::*,
    storage::StorageAddress,
};

use crate::{
    proxy::{
        erc1967::{
            self,
            utils::{
                ERC1967InvalidAdmin, ERC1967InvalidBeacon,
                ERC1967InvalidImplementation, ERC1967NonPayable, Erc1967Utils,
                IMPLEMENTATION_SLOT,
            },
        },
        utils::erc1822::{Erc1822ProxiableInterface, IErc1822Proxiable},
    },
    utils::address,
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// The call is from an unauthorized context.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error UUPSUnauthorizedCallContext();

        /// The storage `slot` is unsupported as a UUID.
        /// * `slot` - The unsupported UUID returned by the implementation.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error UUPSUnsupportedProxiableUUID(bytes32 slot);
    }
}

/// An [`UUPSUpgradeable`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// The call is from an unauthorized context.
    UnauthorizedCallContext(UUPSUnauthorizedCallContext),
    /// The storage `slot` is unsupported as a UUID.
    UnsupportedProxiableUUID(UUPSUnsupportedProxiableUUID),
    /// Indicates an error related to the fact that the `implementation`
    /// of the proxy is invalid.
    InvalidImplementation(ERC1967InvalidImplementation),
    /// Indicates an error related to the fact that the `admin` of the
    /// proxy is invalid.
    InvalidAdmin(ERC1967InvalidAdmin),
    /// Indicates an error related to the fact that the `beacon`
    /// of the proxy is invalid.
    InvalidBeacon(ERC1967InvalidBeacon),
    /// Indicates an error related to the fact that an upgrade function
    /// sees [`stylus_sdk::msg::value()`] > [`alloy_primitives::U256::ZERO`]
    /// that may be lost.
    NonPayable(ERC1967NonPayable),
    /// There's no code at `target` (it is not a contract).
    EmptyCode(address::AddressEmptyCode),
    /// A call to an address target failed. The target may have reverted.
    FailedCall(address::FailedCall),
    /// Indicates an error related to the fact that the delegate call
    /// failed.
    FailedCallWithReason(address::FailedCallWithReason),
}

impl From<erc1967::utils::Error> for Error {
    fn from(e: erc1967::utils::Error) -> Self {
        match e {
            erc1967::utils::Error::InvalidImplementation(e) => {
                Error::InvalidImplementation(e)
            }
            erc1967::utils::Error::InvalidAdmin(e) => Error::InvalidAdmin(e),
            erc1967::utils::Error::InvalidBeacon(e) => Error::InvalidBeacon(e),
            erc1967::utils::Error::NonPayable(e) => Error::NonPayable(e),
            erc1967::utils::Error::EmptyCode(e) => Error::EmptyCode(e),
            erc1967::utils::Error::FailedCall(e) => Error::FailedCall(e),
            erc1967::utils::Error::FailedCallWithReason(e) => {
                Error::FailedCallWithReason(e)
            }
        }
    }
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for UUPSUpgradeable {}

/// Interface for a UUPS (Universal Upgradeable Proxy Standard) upgradeable
/// contract.
#[interface_id]
pub trait IUUPSUpgradeable: IErc1822Proxiable {
    /// The version of the upgrade interface of the contract.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[selector(name = "UPGRADE_INTERFACE_VERSION")]
    fn upgrade_interface_version(&self) -> String {
        String::from("5.0.0")
    }

    /// Upgrade the implementation of the proxy to `new_implementation`, and
    /// subsequently execute the function call encoded in `data`.
    ///
    /// Note: This function should revert when [`stylus_sdk::msg::sender`] is
    /// not authorized to upgrade the contract.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_implementation` - The address of the new implementation contract.
    /// * `data` - Additional data to be passed to the new implementation.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedCallContext`] - If the call is not made through a
    ///   valid proxy context.
    /// * [`Error::InvalidImplementation`] - If the new implementation address
    ///   is invalid or doesn't implement the required interface.
    /// * [`Error::UnsupportedProxiableUUID`] - If the new implementation
    ///   returns an unsupported UUID.
    /// * [`Error::NonPayable`] - If the upgrade function receives ETH but is
    ///   not designed to handle it.
    /// * [`Error::EmptyCode`] - If there's no code at the new implementation
    ///   address.
    /// * [`Error::FailedCall`] - If the delegate call to the new implementation
    ///   fails.
    /// * [`Error::FailedCallWithReason`] - If the delegate call fails with a
    ///   specific reason.
    ///
    /// # Events
    ///
    /// * [`crate::proxy::erc1967::Upgraded`]: Emitted when the implementation
    ///   is upgraded.
    #[selector(name = "upgradeToAndCall")]
    fn upgrade_to_and_call(
        &mut self,
        new_implementation: Address,
        data: Bytes,
    ) -> Result<(), Vec<u8>>;
}

/// State of a [`UUPSUpgradeable`] contract.
#[storage]
pub struct UUPSUpgradeable {
    /// The address of this contract, used for context validation.
    self_address: StorageAddress,
}

#[public]
#[implements(IUUPSUpgradeable, IErc1822Proxiable)]
impl UUPSUpgradeable {
    /// Initializes the contract by storing its own address for later context
    /// validation.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    #[constructor]
    fn constructor(&mut self) {
        self.self_address.set(contract::address());
    }
}

#[public]
impl IUUPSUpgradeable for UUPSUpgradeable {
    #[selector(name = "upgradeToAndCall")]
    #[payable]
    fn upgrade_to_and_call(
        &mut self,
        new_implementation: Address,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.only_proxy()?;
        self._upgrade_to_and_call_uups(new_implementation, &data)?;
        Ok(())
    }
}

impl UUPSUpgradeable {
    /// Check that the execution is being performed through a
    /// [`stylus_sdk::call::delegate_call`] call and that the execution
    /// context is a proxy contract with an implementation (as defined in
    /// ERC-1967) pointing to `self`. This should only be the case for
    /// UUPS and transparent proxies that are using the current contract as
    /// their implementation. Execution of a function through ERC-1167
    /// minimal proxies (clones) would not normally pass this test, but is
    /// not guaranteed to fail.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedCallContext`] - If the execution is not performed
    ///   through a delegate call or the execution context is not of a proxy
    ///   with an ERC-1967 compliant implementation pointing to self.
    pub fn only_proxy(&self) -> Result<(), Error> {
        self._check_proxy()
    }

    /// Check that the execution is not being performed through a delegate call.
    /// This allows a function to be callable on the implementing contract
    /// but not through proxies.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedCallContext`] - If the execution is performed via
    ///   delegate call.
    pub fn not_delegated(&self) -> Result<(), Error> {
        self._check_not_delegated()
    }
}

#[public]
impl IErc1822Proxiable for UUPSUpgradeable {
    fn proxiable_uuid(&self) -> Result<B256, Vec<u8>> {
        self.not_delegated()?;
        Ok(IMPLEMENTATION_SLOT)
    }
}

impl UUPSUpgradeable {
    /// Reverts if the execution is performed via delegate call.
    ///
    /// See [`Self::not_delegated`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedCallContext`] - If the execution is performed via
    ///   delegate call.
    fn _check_not_delegated(&self) -> Result<(), Error> {
        if contract::address() == self.self_address.get() {
            Ok(())
        } else {
            Err(Error::UnauthorizedCallContext(UUPSUnauthorizedCallContext {}))
        }
    }

    /// Reverts if the execution is not performed via delegate call or the
    /// execution context is not of a proxy with an ERC-1967 compliant
    /// implementation pointing to self.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedCallContext`] - If the execution is not performed
    ///   via delegate call or the execution context is not of a proxy with an
    ///   ERC-1967 compliant implementation pointing to self.
    fn _check_proxy(&self) -> Result<(), Error> {
        let self_address = self.self_address.get();
        if contract::address() == self_address
            || Erc1967Utils::get_implementation() != self_address
        {
            Err(Error::UnauthorizedCallContext(UUPSUnauthorizedCallContext {}))
        } else {
            Ok(())
        }
    }

    /// Performs an implementation upgrade with a security check for UUPS
    /// proxies, and additional setup call.
    ///
    /// As a security check, [`IErc1822Proxiable::proxiable_uuid`] is invoked
    /// in the new implementation, and the return value is expected to be the
    /// implementation slot in ERC-1967.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_implementation` - The address of the new implementation.
    /// * `data` - The data to pass to the new implementation.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidImplementation`] - If the new implementation doesn't
    ///   support the required interface or is invalid.
    /// * [`Error::UnsupportedProxiableUUID`] - If the new implementation
    ///   returns an unsupported UUID.
    /// * [`Error::NonPayable`] - If the upgrade function receives ETH but is
    ///   not designed to handle it.
    /// * [`Error::EmptyCode`] - If there's no code at the new implementation
    ///   address.
    /// * [`Error::FailedCall`] - If the delegate call to the new implementation
    ///   fails.
    /// * [`Error::FailedCallWithReason`] - If the delegate call fails with a
    ///   specific reason.
    ///
    /// # Events
    ///
    /// * [`crate::proxy::erc1967::Erc1967Proxy::Upgraded`]: Emitted when the
    ///   implementation is upgraded.
    fn _upgrade_to_and_call_uups(
        &mut self,
        new_implementation: Address,
        data: &Bytes,
    ) -> Result<(), Error> {
        let slot = Erc1822ProxiableInterface::new(new_implementation)
            .proxiable_uuid(Call::new_in(self))
            .map_err(|_e| {
                Error::InvalidImplementation(ERC1967InvalidImplementation {
                    implementation: new_implementation,
                })
            })?;

        if slot == IMPLEMENTATION_SLOT {
            Erc1967Utils::upgrade_to_and_call(self, new_implementation, data)
                .map_err(Error::from)
        } else {
            Err(Error::UnsupportedProxiableUUID(UUPSUnsupportedProxiableUUID {
                slot,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_sol_macro::sol;
    use alloy_sol_types::{SolCall, SolValue};
    use motsu::prelude::*;
    use stylus_sdk::{
        alloy_primitives::{Address, U256},
        prelude::*,
        ArbResult,
    };

    use super::*;
    use crate::{
        access::ownable::{self, IOwnable, Ownable},
        proxy::{self, erc1967::Erc1967Proxy, IProxy},
        token::erc20::{self, Erc20, IErc20},
    };

    #[entrypoint]
    #[storage]
    struct Erc1967ProxyExample {
        erc1967: Erc1967Proxy,
    }

    #[public]
    impl Erc1967ProxyExample {
        #[constructor]
        fn constructor(
            &mut self,
            implementation: Address,
            data: Bytes,
        ) -> Result<(), proxy::erc1967::utils::Error> {
            self.erc1967.constructor(implementation, &data)
        }

        fn implementation(&self) -> Result<Address, Vec<u8>> {
            self.erc1967.implementation()
        }

        #[fallback]
        fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
            unsafe { self.erc1967.do_fallback(calldata) }
        }
    }

    #[storage]
    struct UUPSErc20Example {
        erc20: Erc20,
        ownable: Ownable,
        uups: UUPSUpgradeable,
    }

    #[public]
    #[implements(IErc20<Error = erc20::Error>, IUUPSUpgradeable, IErc1822Proxiable, IOwnable)]
    impl UUPSErc20Example {
        #[constructor]
        fn constructor(
            &mut self,
            initial_owner: Address,
        ) -> Result<(), ownable::Error> {
            self.uups.constructor();
            self.ownable.constructor(initial_owner)
        }

        fn mint(
            &mut self,
            to: Address,
            value: U256,
        ) -> Result<(), erc20::Error> {
            self.erc20._mint(to, value)
        }
    }

    unsafe impl TopLevelStorage for UUPSErc20Example {}

    #[public]
    impl IErc20 for UUPSErc20Example {
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
    impl IUUPSUpgradeable for UUPSErc20Example {
        #[selector(name = "UPGRADE_INTERFACE_VERSION")]
        fn upgrade_interface_version(&self) -> String {
            self.uups.upgrade_interface_version()
        }

        fn upgrade_to_and_call(
            &mut self,
            new_implementation: Address,
            data: Bytes,
        ) -> Result<(), Vec<u8>> {
            self.ownable.only_owner()?;
            self.uups.upgrade_to_and_call(new_implementation, data)?;
            Ok(())
        }
    }

    #[public]
    impl IOwnable for UUPSErc20Example {
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
    impl IErc1822Proxiable for UUPSErc20Example {
        #[selector(name = "proxiableUUID")]
        fn proxiable_uuid(&self) -> Result<B256, Vec<u8>> {
            self.uups.proxiable_uuid()
        }
    }

    #[storage]
    struct FakeImplementation {}

    #[public]
    #[implements(IErc1822Proxiable)]
    impl FakeImplementation {}

    #[public]
    impl IErc1822Proxiable for FakeImplementation {
        /// Returns an incorrect UUID to simulate an invalid UUPS upgrade
        /// target.
        #[selector(name = "proxiableUUID")]
        fn proxiable_uuid(&self) -> Result<B256, Vec<u8>> {
            // Return a UUID that is NOT equal to IMPLEMENTATION_SLOT
            Ok(B256::from([0xFF; 32])) // Invalid slot
        }
    }

    unsafe impl TopLevelStorage for FakeImplementation {}

    sol! {
        interface ERC20Interface {
            function balanceOf(address account) external view returns (uint256);
            function totalSupply() external view returns (uint256);
            function mint(address to, uint256 value) external;
            function transfer(address to, uint256 value) external returns (bool);
        }

        interface UUPSUpgradeableInterface {
            function upgradeToAndCall(address newImplementation, bytes calldata data) external payable;
        }

        interface OwnableInterface {
            function owner() external view returns (address);
            function transferOwnership(address newOwner) external;
        }
    }

    #[motsu::test]
    fn test_initialization(
        proxy: Contract<Erc1967ProxyExample>,
        logic: Contract<UUPSErc20Example>,
        alice: Address,
    ) {
        logic.sender(alice).constructor(alice).unwrap();
        let mint_amount = U256::from(1000);
        let mint_data =
            ERC20Interface::mintCall { to: alice, value: mint_amount }
                .abi_encode();

        proxy
            .sender(alice)
            .constructor(logic.address(), mint_data.into())
            .unwrap();

        let bal_call =
            ERC20Interface::balanceOfCall { account: alice }.abi_encode();
        let result = proxy.sender(alice).fallback(&bal_call).unwrap();

        assert_eq!(result, mint_amount.abi_encode());
    }

    #[motsu::test]
    fn test_upgrade_by_non_owner_fails(
        proxy: Contract<Erc1967ProxyExample>,
        logic_v1: Contract<UUPSErc20Example>,
        logic_v2: Contract<UUPSErc20Example>,
        alice: Address,
        bob: Address,
    ) {
        logic_v1.sender(alice).constructor(alice).unwrap();
        logic_v2.sender(alice).constructor(alice).unwrap();

        let mint_data =
            ERC20Interface::mintCall { to: alice, value: U256::from(1) }
                .abi_encode();
        proxy
            .sender(alice)
            .constructor(logic_v1.address(), mint_data.into())
            .unwrap();

        let upgrade_data = UUPSUpgradeableInterface::upgradeToAndCallCall {
            newImplementation: logic_v2.address(),
            data: vec![].into(),
        }
        .abi_encode();

        let err = proxy.sender(bob).fallback(&upgrade_data).unwrap_err();
        assert_eq!(
            err,
            ownable::Error::UnauthorizedAccount(
                ownable::OwnableUnauthorizedAccount { account: bob }
            )
            .encode()
        );
    }

    #[motsu::test]
    fn test_upgrade_with_wrong_uuid_fails(
        proxy: Contract<Erc1967ProxyExample>,
        logic_v1: Contract<UUPSErc20Example>,
        fake_impl: Contract<FakeImplementation>,
        alice: Address,
    ) {
        logic_v1.sender(alice).constructor(alice).unwrap();

        let init_data =
            ERC20Interface::mintCall { to: alice, value: U256::from(1) }
                .abi_encode();
        proxy
            .sender(alice)
            .constructor(logic_v1.address(), init_data.into())
            .unwrap();

        let upgrade_data = UUPSUpgradeableInterface::upgradeToAndCallCall {
            newImplementation: fake_impl.address(),
            data: vec![].into(),
        }
        .abi_encode();

        let err = proxy.sender(alice).fallback(&upgrade_data).unwrap_err();
        assert_eq!(
            err,
            Error::UnsupportedProxiableUUID(UUPSUnsupportedProxiableUUID {
                slot: B256::from([0xFF; 32]),
            })
            .encode()
        );
    }

    #[motsu::test]
    fn test_direct_call_upgrade_fails(
        logic: Contract<UUPSErc20Example>,
        logic_v2: Contract<UUPSErc20Example>,
        alice: Address,
    ) {
        logic.sender(alice).constructor(alice).unwrap();
        logic_v2.sender(alice).constructor(alice).unwrap();

        let result = logic
            .sender(alice)
            .upgrade_to_and_call(logic_v2.address(), vec![].into());

        assert_eq!(
            result.unwrap_err(),
            Error::UnauthorizedCallContext(UUPSUnauthorizedCallContext {})
                .encode()
        );
    }

    #[motsu::test]
    fn test_direct_uuid_check_succeeds(
        logic: Contract<UUPSErc20Example>,
        alice: Address,
    ) {
        logic.sender(alice).constructor(alice).unwrap();

        let result = logic.sender(alice).proxiable_uuid();
        assert_eq!(result, Ok(IMPLEMENTATION_SLOT));
    }

    #[motsu::test]
    fn test_transfer_ownership_and_upgrade(
        proxy: Contract<Erc1967ProxyExample>,
        logic_v1: Contract<UUPSErc20Example>,
        logic_v2: Contract<UUPSErc20Example>,
        alice: Address,
        bob: Address,
    ) {
        logic_v1.sender(alice).constructor(alice).unwrap();
        logic_v2.sender(alice).constructor(alice).unwrap();

        let init_data =
            ERC20Interface::mintCall { to: alice, value: U256::from(1) }
                .abi_encode();
        proxy
            .sender(alice)
            .constructor(logic_v1.address(), init_data.into())
            .unwrap();

        let transfer_call =
            OwnableInterface::transferOwnershipCall { newOwner: bob }
                .abi_encode();
        proxy.sender(alice).fallback(&transfer_call).unwrap();

        let upgrade_data = UUPSUpgradeableInterface::upgradeToAndCallCall {
            newImplementation: logic_v2.address(),
            data: vec![].into(),
        }
        .abi_encode();

        proxy.sender(bob).fallback(&upgrade_data).unwrap();

        let balance_call =
            ERC20Interface::balanceOfCall { account: alice }.abi_encode();
        let bal = proxy.sender(alice).fallback(&balance_call).unwrap();
        assert_eq!(bal, U256::from(1).abi_encode());
    }
}
