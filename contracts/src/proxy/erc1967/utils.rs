//! This library provides getters and event emitting update functions for
//! [ERC-1967] slots.
//!
//! [ERC-1967]: https://eips.ethereum.org/EIPS/eip-1967

use alloy_primitives::{aliases::B256, Address, U256};
pub use sol::*;
use stylus_sdk::{
    abi::Bytes, call::MethodError, evm, msg, prelude::*,
    storage::StorageAddress,
};

use crate::{
    proxy::{abi::BeaconInterface, erc1967},
    utils::{
        address::{self, AddressUtils},
        storage_slot::StorageSlot,
    },
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Indicates an error related to the fact that the `implementation`
        /// of the proxy is invalid.
        ///
        /// * `implementation` - Address of the invalid implementation.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC1967InvalidImplementation(address implementation);

        /// Indicates an error related to the fact that the `admin` of the
        /// proxy is invalid.
        ///
        /// * `admin` - Address of the invalid admin.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC1967InvalidAdmin(address admin);

        /// Indicates an error related to the fact that the `beacon`
        /// of the proxy is invalid.
        ///
        /// * `beacon` - Address of the invalid `beacon` of the proxy.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC1967InvalidBeacon(address beacon);

        /// Indicates an error relatoed to the fact that an upgrade function
        /// sees [`stylus_sdk::msg::value()`] > [`U256::ZERO`] that may be lost.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC1967NonPayable();
    }
}

/// An [`Erc1967Utils`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the fact that the `implementation`
    /// of the proxy is invalid.
    InvalidImplementation(ERC1967InvalidImplementation),
    /// Indicates an error related to the fact that the `admin` of the
    /// proxy is invalid.
    InvalidAdmin(ERC1967InvalidAdmin),
    /// Indicates an error related to the fact that the `beacon`
    /// of the proxy is invalid.
    InvalidBeacon(ERC1967InvalidBeacon),
    /// Indicates an error relatoed to the fact that an upgrade function
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

#[cfg_attr(coverage_nightly, coverage(off))]
impl From<address::Error> for Error {
    fn from(e: address::Error) -> Self {
        match e {
            address::Error::EmptyCode(e) => Error::EmptyCode(e),
            address::Error::FailedCall(e) => Error::FailedCall(e),
            address::Error::FailedCallWithReason(e) => {
                Error::FailedCallWithReason(e)
            }
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// Storage slot with the address of the current implementation.
pub const IMPLEMENTATION_SLOT: B256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"eip1967.proxy.implementation")
        .finalize();
    B256::new(U256::from_be_bytes(HASH).wrapping_sub(U256::ONE).to_be_bytes())
};

/// Storage slot with the admin of the contract.
pub const ADMIN_SLOT: B256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"eip1967.proxy.admin")
        .finalize();
    B256::new(U256::from_be_bytes(HASH).wrapping_sub(U256::ONE).to_be_bytes())
};

/// The storage slot of the beacon contract which defines the implementation
/// for this proxy.
pub const BEACON_SLOT: B256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"eip1967.proxy.beacon")
        .finalize();
    B256::new(U256::from_be_bytes(HASH).wrapping_sub(U256::ONE).to_be_bytes())
};

/// This library provides getters and event emitting update functions for
/// [ERC-1967] slots.
///
/// [ERC-1967]: https://eips.ethereum.org/EIPS/eip-1967
pub struct Erc1967Utils;

/// Implementation of the [`Erc1967Utils`] library.
impl Erc1967Utils {
    /// Returns the current implementation address.
    #[must_use]
    pub fn get_implementation() -> Address {
        StorageSlot::get_slot::<StorageAddress>(IMPLEMENTATION_SLOT).get()
    }

    /// Performs implementation upgrade with additional setup call if
    /// data is nonempty. This function is payable only if the setup call
    /// is performed, otherwise [`msg::value()`] is rejected to avoid stuck
    /// value in the contract.
    ///
    /// # Arguments
    ///
    /// * `context` - Mutable access to the contract's state.
    /// * `new_implementation` - The new implementation address.
    /// * `data` - The data to pass to the setup call.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidImplementation`] - If the `new_implementation` address
    ///   is not a valid implementation.
    /// * [`Error::NonPayable`] - If `data` is empty and [`msg::value`] is not
    ///   [`U256::ZERO`].
    /// * [`Error::FailedCall`] - If the call to the implementation contract
    ///   fails.
    /// * [`Error::FailedCallWithReason`] - If the call to the implementation
    ///   contract fails with a revert reason.
    pub fn upgrade_to_and_call<T: TopLevelStorage>(
        context: &mut T,
        new_implementation: Address,
        data: &Bytes,
    ) -> Result<(), Error> {
        Erc1967Utils::set_implementation(new_implementation)?;

        evm::log(erc1967::Upgraded { implementation: new_implementation });

        if data.is_empty() {
            Erc1967Utils::check_non_payable()?;
        } else {
            AddressUtils::function_delegate_call(
                context,
                new_implementation,
                data.as_slice(),
            )?;
        }

        Ok(())
    }

    /// Returns the current admin.
    #[must_use]
    pub fn get_admin() -> Address {
        StorageSlot::get_slot::<StorageAddress>(ADMIN_SLOT).get()
    }

    /// Changes the admin of the proxy.
    ///
    /// # Arguments
    ///
    /// * `new_admin` - The new admin address.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAdmin`] - If the `new_admin` address is not a valid
    ///   admin.
    pub fn change_admin(new_admin: Address) -> Result<(), Error> {
        evm::log(erc1967::AdminChanged {
            previous_admin: Erc1967Utils::get_admin(),
            new_admin,
        });

        Erc1967Utils::set_admin(new_admin)
    }

    /// Returns the current beacon.
    #[must_use]
    pub fn get_beacon() -> Address {
        StorageSlot::get_slot::<StorageAddress>(BEACON_SLOT).get()
    }

    /// Change the beacon and trigger a setup call if data is nonempty.
    /// This function is payable only if the setup call is performed,
    /// otherwise [`msg::value()`] is rejected to avoid stuck value in the
    /// contract.
    ///
    /// # Arguments
    ///
    /// * `context` - Mutable access to the contract's state.
    /// * `new_beacon` - The new beacon address.
    /// * `data` - The data to pass to the setup call.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidBeacon`] - If the `new_beacon` address is not a valid
    ///   beacon.
    /// * [`Error::InvalidImplementation`] - If the beacon implementation is not
    ///   a valid implementation.
    /// * [`Error::NonPayable`] - If `data` is empty and [`msg::value`] is not
    ///   [`U256::ZERO`].
    /// * [`Error::FailedCall`] - If the call to the beacon implementation
    ///   fails.
    /// * [`Error::FailedCallWithReason`] - If the call to the beacon
    ///   implementation fails with a revert reason.
    pub fn upgrade_beacon_to_and_call<T: TopLevelStorage>(
        context: &mut T,
        new_beacon: Address,
        data: &Bytes,
    ) -> Result<(), Error> {
        Erc1967Utils::set_beacon(context, new_beacon)?;
        evm::log(erc1967::BeaconUpgraded { beacon: new_beacon });

        if data.is_empty() {
            Erc1967Utils::check_non_payable()?;
        } else {
            let beacon_implementation =
                Erc1967Utils::get_beacon_implementation(context, new_beacon)?;

            AddressUtils::function_delegate_call(
                context,
                beacon_implementation,
                data.as_slice(),
            )?;
        }

        Ok(())
    }
}

impl Erc1967Utils {
    /// Reverts if [`msg::value()`] is not [`alloy_primitives::U256::ZERO`]. It
    /// can be used to avoid [`msg::value()`] stuck in the contract if an
    /// upgrade does not perform an initialization call.
    ///
    /// # Errors
    ///
    /// * [`Error::NonPayable`] - If [`msg::value()`] is not
    ///   [`alloy_primitives::U256::ZERO`].
    fn check_non_payable() -> Result<(), Error> {
        if msg::value().is_zero() {
            Ok(())
        } else {
            Err(ERC1967NonPayable {}.into())
        }
    }

    /// Stores a new address in the ERC-1967 implementation slot.
    ///
    /// # Arguments
    ///
    /// * `new_implementation` - The new implementation address.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidImplementation`] - If the `new_implementation` address
    ///   is not a valid implementation.
    fn set_implementation(new_implementation: Address) -> Result<(), Error> {
        if !new_implementation.has_code() {
            return Err(ERC1967InvalidImplementation {
                implementation: new_implementation,
            }
            .into());
        }

        StorageSlot::get_slot::<StorageAddress>(IMPLEMENTATION_SLOT)
            .set(new_implementation);

        Ok(())
    }

    /// Stores a new address in the ERC-1967 admin slot.
    ///
    /// # Arguments
    ///
    /// * `new_admin` - The new admin address.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAdmin`] - If the `new_admin` address is not a valid
    ///   admin.
    fn set_admin(new_admin: Address) -> Result<(), Error> {
        if new_admin.is_zero() {
            return Err(ERC1967InvalidAdmin { admin: new_admin }.into());
        }

        StorageSlot::get_slot::<StorageAddress>(ADMIN_SLOT).set(new_admin);

        Ok(())
    }

    /// Stores a new beacon in the ERC-1967 beacon slot.
    ///
    /// # Arguments
    ///
    /// * `context` - Mutable access to the contract's state.
    /// * `new_beacon` - The new beacon address.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidBeacon`] - If the `new_beacon` address is not a valid
    ///   beacon.
    /// * [`Error::InvalidImplementation`] - If the beacon implementation is not
    ///   a valid implementation.
    /// * [`Error::FailedCall`] - If the call to the beacon implementation
    ///   fails.
    /// * [`Error::FailedCallWithReason`] - If the call to the beacon
    ///   implementation fails with a revert reason.
    fn set_beacon<T: TopLevelStorage>(
        context: &mut T,
        new_beacon: Address,
    ) -> Result<(), Error> {
        if !new_beacon.has_code() {
            return Err(ERC1967InvalidBeacon { beacon: new_beacon }.into());
        }

        StorageSlot::get_slot::<StorageAddress>(BEACON_SLOT).set(new_beacon);

        let beacon_implementation =
            Erc1967Utils::get_beacon_implementation(context, new_beacon)?;

        if !beacon_implementation.has_code() {
            return Err(ERC1967InvalidImplementation {
                implementation: beacon_implementation,
            }
            .into());
        }

        Ok(())
    }
}

impl Erc1967Utils {
    /// Returns the implementation address of the beacon.
    ///
    /// # Arguments
    ///
    /// * `context` - Readonly access to the contract's state.
    /// * `beacon` - The beacon address.
    ///
    /// # Errors
    ///
    /// * [`Error::FailedCall`] - If the call to the beacon implementation
    ///   fails.
    /// * [`Error::FailedCallWithReason`] - If the call to the beacon
    ///   implementation fails with a revert reason.
    /// * [`Error::EmptyCode`] - If the beacon implementation has no code.
    fn get_beacon_implementation<T: TopLevelStorage>(
        context: &T,
        beacon: Address,
    ) -> Result<Address, Error> {
        Ok(AddressUtils::verify_call_result_from_target(
            beacon,
            BeaconInterface::new(beacon).implementation(context),
        )?)
    }
}

#[cfg(test)]
#[allow(clippy::needless_pass_by_value, clippy::unused_self)]
mod tests {

    use alloy_sol_types::SolCall;
    use motsu::prelude::*;
    use stylus_sdk::{
        alloy_primitives::{uint, Address},
        function_selector,
        prelude::*,
        storage::StorageAddress,
    };

    use super::*;
    use crate::proxy::beacon::IBeacon;

    #[entrypoint]
    #[storage]
    struct TestContract;

    #[public]
    impl TestContract {
        // test functions that wrap [`Erc1967Utils`] methods.
        fn test_get_implementation(&self) -> Address {
            Erc1967Utils::get_implementation()
        }

        fn test_upgrade_to_and_call(
            &mut self,
            new_implementation: Address,
            data: Bytes,
        ) -> Result<(), Vec<u8>> {
            Ok(Erc1967Utils::upgrade_to_and_call(
                self,
                new_implementation,
                &data,
            )?)
        }

        fn test_get_admin(&self) -> Address {
            Erc1967Utils::get_admin()
        }

        fn test_change_admin(
            &mut self,
            new_admin: Address,
        ) -> Result<(), Vec<u8>> {
            Ok(Erc1967Utils::change_admin(new_admin)?)
        }

        fn test_get_beacon(&self) -> Address {
            Erc1967Utils::get_beacon()
        }

        fn test_upgrade_beacon_to_and_call(
            &mut self,
            new_beacon: Address,
            data: Bytes,
        ) -> Result<(), Vec<u8>> {
            Ok(Erc1967Utils::upgrade_beacon_to_and_call(
                self, new_beacon, &data,
            )?)
        }
    }

    // mock beacon contract for testing.
    #[storage]
    struct MockBeacon {
        implementation: StorageAddress,
    }

    unsafe impl TopLevelStorage for MockBeacon {}

    #[public]
    #[implements(IBeacon)]
    impl MockBeacon {
        #[constructor]
        fn constructor(&mut self, implementation: Address) {
            self.implementation.set(implementation);
        }
    }

    #[public]
    impl IBeacon for MockBeacon {
        fn implementation(&self) -> Result<Address, Vec<u8>> {
            Ok(self.implementation.get())
        }
    }

    use sol_types::*;

    #[cfg_attr(coverage_nightly, coverage(off))]
    mod sol_types {
        use stylus_sdk::alloy_sol_types::sol;

        sol! {
            #[derive(Debug)]
            #[allow(missing_docs)]
            error ImplementationSolidityError();

            #[derive(Debug)]
            #[allow(missing_docs)]
            event ImplementationEvent();
        }

        sol! {
            interface IImplementation {
                function emitOrError(bool should_error) external;
            }
        }
    }

    #[derive(SolidityError, Debug)]
    enum ImplementationError {
        ImplementationError(ImplementationSolidityError),
    }

    impl MethodError for ImplementationError {
        fn encode(self) -> Vec<u8> {
            self.into()
        }
    }

    #[storage]
    struct Implementation;

    unsafe impl TopLevelStorage for Implementation {}

    #[public]
    impl Implementation {
        fn emit_or_error(
            &mut self,
            should_error: bool,
        ) -> Result<(), ImplementationError> {
            if should_error {
                return Err(ImplementationError::ImplementationError(
                    ImplementationSolidityError {},
                ));
            }
            evm::log(ImplementationEvent {});
            Ok(())
        }
    }

    #[motsu::test]
    fn get_implementation_returns_zero_by_default(
        contract: Contract<TestContract>,
        alice: Address,
    ) {
        let implementation = contract.sender(alice).test_get_implementation();
        assert_eq!(implementation, Address::ZERO);
    }

    #[motsu::test]
    fn upgrade_to_and_call_with_valid_implementation_and_empty_data(
        contract: Contract<TestContract>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        contract
            .sender(alice)
            .test_upgrade_to_and_call(implementation.address(), vec![].into())
            .motsu_expect("should be able to upgrade to valid implementation");

        let implementation_addr =
            contract.sender(alice).test_get_implementation();
        assert_eq!(implementation_addr, implementation.address());

        // verify event was emitted.
        contract.assert_emitted(&erc1967::Upgraded {
            implementation: implementation.address(),
        });
    }

    #[motsu::test]
    fn upgrade_to_and_call_with_valid_implementation_and_data(
        contract: Contract<TestContract>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        let data = IImplementation::emitOrErrorCall { should_error: false }
            .abi_encode();
        contract
            .sender(alice)
            .test_upgrade_to_and_call(implementation.address(), data.into())
            .motsu_expect(
                "should be able to upgrade to valid implementation with data",
            );

        let implementation_addr =
            contract.sender(alice).test_get_implementation();
        assert_eq!(implementation_addr, implementation.address());

        // verify event was emitted.
        contract.assert_emitted(&erc1967::Upgraded {
            implementation: implementation.address(),
        });

        // TODO: this should assert that the event was emitted on `contract`
        // (as it's the one that delegate called the upgrade)
        // https://github.com/OpenZeppelin/stylus-test-helpers/issues/111
        implementation.assert_emitted(&ImplementationEvent {});
    }

    #[motsu::test]
    fn upgrade_to_and_call_with_invalid_implementation(
        contract: Contract<TestContract>,
        alice: Address,
    ) {
        let invalid_address = alice; // address with no code.
        let err = contract
            .sender(alice)
            .test_upgrade_to_and_call(invalid_address, vec![].into())
            .motsu_expect_err(
                "should fail when upgrading to invalid implementation",
            );

        assert_eq!(
            err,
            Error::InvalidImplementation(ERC1967InvalidImplementation {
                implementation: invalid_address,
            })
            .encode()
        );

        // implementation should remain unchanged.
        let implementation = contract.sender(alice).test_get_implementation();
        assert_eq!(implementation, Address::ZERO);
    }

    #[motsu::test]
    fn upgrade_to_and_call_with_zero_implementation(
        contract: Contract<TestContract>,
        alice: Address,
    ) {
        let err = contract
            .sender(alice)
            .test_upgrade_to_and_call(Address::ZERO, vec![].into())
            .motsu_expect_err(
                "should fail when upgrading to zero implementation",
            );

        assert_eq!(
            err,
            Error::InvalidImplementation(ERC1967InvalidImplementation {
                implementation: Address::ZERO,
            })
            .encode()
        );
    }

    #[motsu::test]
    fn upgrade_to_and_call_with_same_implementation(
        contract: Contract<TestContract>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        // first upgrade.
        contract
            .sender(alice)
            .test_upgrade_to_and_call(implementation.address(), vec![].into())
            .motsu_expect("should be able to upgrade first time");

        // upgrade to same implementation.
        contract
            .sender(alice)
            .test_upgrade_to_and_call(implementation.address(), vec![].into())
            .motsu_expect("should be able to upgrade to same implementation");

        let implementation_addr =
            contract.sender(alice).test_get_implementation();
        assert_eq!(implementation_addr, implementation.address());

        // event should still be emitted.
        contract.assert_emitted(&erc1967::Upgraded {
            implementation: implementation.address(),
        });
    }

    #[motsu::test]
    fn upgrade_to_and_call_with_delegate_call_failure(
        contract: Contract<TestContract>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        let data = function_selector!("nonExistentFunction").to_vec();

        let err = contract
            .sender(alice)
            .test_upgrade_to_and_call(
                implementation.address(),
                data.clone().into(),
            )
            .motsu_expect_err("should fail when delegate call fails");

        let vec = format!(
            "function not found for selector '{0}' and no fallback defined",
            u32::from_be_bytes(TryInto::try_into(data).unwrap())
        )
        .as_bytes()
        .to_vec();

        assert_eq!(
            err,
            Error::FailedCallWithReason(address::FailedCallWithReason {
                reason: stylus_sdk::call::Error::Revert(vec).encode().into()
            })
            .encode(),
        );
    }

    #[motsu::test]
    fn upgrade_to_and_call_with_implementation_reverts(
        contract: Contract<TestContract>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        let data = IImplementation::emitOrErrorCall { should_error: true }
            .abi_encode();

        let err = contract
            .sender(alice)
            .test_upgrade_to_and_call(
                implementation.address(),
                data.clone().into(),
            )
            .motsu_expect_err("should fail when implementation reverts");

        assert_eq!(
            err,
            Error::FailedCallWithReason(address::FailedCallWithReason {
                reason: ImplementationError::ImplementationError(
                    ImplementationSolidityError {}
                )
                .encode()
                .into(),
            })
            .encode()
        );
    }

    #[motsu::test]
    fn upgrade_to_and_call_reverts_with_empty_data_with_value(
        contract: Contract<TestContract>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        alice.fund(uint!(1000_U256));

        let err = contract
            .sender_and_value(alice, uint!(1000_U256))
            .test_upgrade_to_and_call(implementation.address(), vec![].into())
            .motsu_expect_err("should fail with ERC1967NonPayable");

        assert_eq!(err, Error::NonPayable(ERC1967NonPayable {}).encode());
    }

    #[motsu::test]
    fn get_admin_returns_zero_by_default(
        contract: Contract<TestContract>,
        alice: Address,
    ) {
        let admin = contract.sender(alice).test_get_admin();
        assert_eq!(admin, Address::ZERO);
    }

    #[motsu::test]
    fn change_admin_with_valid_address(
        contract: Contract<TestContract>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            .test_change_admin(bob)
            .motsu_expect("should be able to change admin to valid address");

        let admin = contract.sender(alice).test_get_admin();
        assert_eq!(admin, bob);

        // Verify event was emitted
        contract.assert_emitted(&erc1967::AdminChanged {
            previous_admin: Address::ZERO,
            new_admin: bob,
        });

        // verify that changing admin to [`Address::ZERO`] fails.
        let err = contract
            .sender(alice)
            .test_change_admin(Address::ZERO)
            .motsu_expect_err(
                "should fail when changing admin to zero address",
            );

        assert_eq!(
            err,
            Error::InvalidAdmin(ERC1967InvalidAdmin { admin: Address::ZERO })
                .encode()
        );

        // admin should remain unchanged.
        let admin = contract.sender(alice).test_get_admin();
        assert_eq!(admin, bob);
    }

    #[motsu::test]
    fn change_admin_multiple_times(
        contract: Contract<TestContract>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        // first change.
        contract
            .sender(alice)
            .test_change_admin(bob)
            .motsu_expect("should be able to change admin first time");

        let admin = contract.sender(alice).test_get_admin();
        assert_eq!(admin, bob);

        // second change.
        contract
            .sender(alice)
            .test_change_admin(charlie)
            .motsu_expect("should be able to change admin second time");

        let admin = contract.sender(alice).test_get_admin();
        assert_eq!(admin, charlie);

        // verify events were emitted.
        contract.assert_emitted(&erc1967::AdminChanged {
            previous_admin: Address::ZERO,
            new_admin: bob,
        });
        contract.assert_emitted(&erc1967::AdminChanged {
            previous_admin: bob,
            new_admin: charlie,
        });
    }

    #[motsu::test]
    fn change_admin_with_same_address(
        contract: Contract<TestContract>,
        alice: Address,
        bob: Address,
    ) {
        // first change.
        contract
            .sender(alice)
            .test_change_admin(bob)
            .motsu_expect("should be able to change admin first time");

        // change to same address.
        contract
            .sender(alice)
            .test_change_admin(bob)
            .motsu_expect("should be able to change admin to same address");

        let admin = contract.sender(alice).test_get_admin();
        assert_eq!(admin, bob);

        // event should still be emitted.
        contract.assert_emitted(&erc1967::AdminChanged {
            previous_admin: Address::ZERO,
            new_admin: bob,
        });
        contract.assert_emitted(&erc1967::AdminChanged {
            previous_admin: bob,
            new_admin: bob,
        });
    }

    #[motsu::test]
    fn get_beacon_returns_zero_by_default(
        contract: Contract<TestContract>,
        alice: Address,
    ) {
        let beacon = contract.sender(alice).test_get_beacon();
        assert_eq!(beacon, Address::ZERO);
    }

    #[motsu::test]
    fn upgrade_beacon_to_and_call_with_valid_beacon_and_empty_data(
        contract: Contract<TestContract>,
        beacon: Contract<MockBeacon>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(implementation.address());

        contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(beacon.address(), vec![].into())
            .motsu_expect("should be able to upgrade to valid beacon");

        let beacon_address = contract.sender(alice).test_get_beacon();
        assert_eq!(beacon_address, beacon.address());

        // verify event was emitted.
        contract.assert_emitted(&erc1967::BeaconUpgraded {
            beacon: beacon.address(),
        });
    }

    #[motsu::test]
    fn upgrade_beacon_to_and_call_with_valid_beacon_and_data(
        contract: Contract<TestContract>,
        beacon: Contract<MockBeacon>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(implementation.address());

        let data = IImplementation::emitOrErrorCall { should_error: false }
            .abi_encode();
        contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(beacon.address(), data.into())
            .motsu_expect(
                "should be able to upgrade to valid beacon with data",
            );

        let beacon_address = contract.sender(alice).test_get_beacon();
        assert_eq!(beacon_address, beacon.address());

        // verify event was emitted.
        contract.assert_emitted(&erc1967::BeaconUpgraded {
            beacon: beacon.address(),
        });

        // TODO: this should assert that the event was emitted on `contract`
        // (as it's the one that delegate called the upgrade)
        // https://github.com/OpenZeppelin/stylus-test-helpers/issues/111
        implementation.assert_emitted(&ImplementationEvent {});
    }

    #[motsu::test]
    fn upgrade_beacon_to_and_call_with_invalid_beacon(
        contract: Contract<TestContract>,
        alice: Address,
    ) {
        let invalid_address = alice; // address with no code.
        let err = contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(invalid_address, vec![].into())
            .motsu_expect_err("should fail when upgrading to invalid beacon");

        assert_eq!(
            err,
            Error::InvalidBeacon(ERC1967InvalidBeacon {
                beacon: invalid_address,
            })
            .encode()
        );

        // beacon should remain unchanged.
        let beacon = contract.sender(alice).test_get_beacon();
        assert_eq!(beacon, Address::ZERO);
    }

    #[motsu::test]
    fn upgrade_beacon_to_and_call_with_zero_beacon(
        contract: Contract<TestContract>,
        beacon: Contract<MockBeacon>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(implementation.address());

        contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(beacon.address(), vec![].into())
            .motsu_expect("should be able to upgrade to valid beacon");

        let err = contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(Address::ZERO, vec![].into())
            .motsu_expect_err("should fail when upgrading to zero beacon");

        assert_eq!(
            err,
            Error::InvalidBeacon(ERC1967InvalidBeacon {
                beacon: Address::ZERO,
            })
            .encode()
        );
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[motsu::test]
    #[ignore = "TODO: motsu doesn't properly reset custom storage slots on transaction revert. See https://github.com/OpenZeppelin/stylus-test-helpers/issues/112"]
    fn upgrade_beacon_to_and_call_with_beacon_returning_invalid_implementation(
        contract: Contract<TestContract>,
        beacon: Contract<MockBeacon>,
        alice: Address,
    ) {
        // beacon returns an address with no code.
        let invalid_address = alice; // address with no code.
        beacon.sender(alice).constructor(invalid_address);

        let err = contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(beacon.address(), vec![].into())
            .motsu_expect_err(
                "should fail when beacon returns invalid implementation",
            );

        assert_eq!(
            err,
            Error::InvalidImplementation(ERC1967InvalidImplementation {
                implementation: invalid_address,
            })
            .encode()
        );

        // beacon should remain unchanged.
        let beacon_address = contract.sender(alice).test_get_beacon();
        assert_eq!(beacon_address, Address::ZERO);
    }

    #[motsu::test]
    fn upgrade_beacon_to_and_call_with_beacon_returning_zero_implementation(
        contract: Contract<TestContract>,
        beacon: Contract<MockBeacon>,
        alice: Address,
    ) {
        // beacon returns [`Address::ZERO`].
        beacon.sender(alice).constructor(Address::ZERO);

        let err = contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(beacon.address(), vec![].into())
            .motsu_expect_err(
                "should fail when beacon returns zero implementation",
            );

        assert_eq!(
            err,
            Error::InvalidImplementation(ERC1967InvalidImplementation {
                implementation: Address::ZERO,
            })
            .encode()
        );
    }

    #[motsu::test]
    fn upgrade_beacon_to_and_call_multiple_times(
        contract: Contract<TestContract>,
        beacon1: Contract<MockBeacon>,
        beacon2: Contract<MockBeacon>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        beacon1.sender(alice).constructor(implementation.address());
        beacon2.sender(alice).constructor(implementation.address());

        // first upgrade.
        contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(beacon1.address(), vec![].into())
            .motsu_expect("should be able to upgrade beacon first time");

        let beacon = contract.sender(alice).test_get_beacon();
        assert_eq!(beacon, beacon1.address());

        // second upgrade.
        contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(beacon2.address(), vec![].into())
            .motsu_expect("should be able to upgrade beacon second time");

        let beacon = contract.sender(alice).test_get_beacon();
        assert_eq!(beacon, beacon2.address());

        // verify events were emitted.
        contract.assert_emitted(&erc1967::BeaconUpgraded {
            beacon: beacon1.address(),
        });
        contract.assert_emitted(&erc1967::BeaconUpgraded {
            beacon: beacon2.address(),
        });
    }

    #[motsu::test]
    fn upgrade_beacon_to_and_call_with_same_beacon(
        contract: Contract<TestContract>,
        beacon: Contract<MockBeacon>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(implementation.address());

        // first upgrade.
        contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(beacon.address(), vec![].into())
            .motsu_expect("should be able to upgrade beacon first time");

        // upgrade to same beacon.
        contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(beacon.address(), vec![].into())
            .motsu_expect("should be able to upgrade to same beacon");

        let beacon_address = contract.sender(alice).test_get_beacon();
        assert_eq!(beacon_address, beacon.address());

        // event should still be emitted.
        contract.assert_emitted(&erc1967::BeaconUpgraded {
            beacon: beacon.address(),
        });
    }

    #[motsu::test]
    fn upgrade_beacon_to_and_call_with_delegate_call_failure(
        contract: Contract<TestContract>,
        beacon: Contract<MockBeacon>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(implementation.address());

        let data = function_selector!("nonExistentFunction").to_vec();

        let err = contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(
                beacon.address(),
                data.clone().into(),
            )
            .motsu_expect_err("should fail when delegate call fails");

        let vec = format!(
            "function not found for selector '{0}' and no fallback defined",
            u32::from_be_bytes(TryInto::try_into(data).unwrap())
        )
        .as_bytes()
        .to_vec();

        assert_eq!(
            err,
            Error::FailedCallWithReason(address::FailedCallWithReason {
                reason: stylus_sdk::call::Error::Revert(vec).encode().into()
            })
            .encode(),
        );
    }

    #[motsu::test]
    fn upgrade_beacon_to_and_call_with_beacon_implementation_reverts(
        contract: Contract<TestContract>,
        beacon: Contract<MockBeacon>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(implementation.address());

        let data = IImplementation::emitOrErrorCall { should_error: true }
            .abi_encode();

        let err = contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(
                beacon.address(),
                data.clone().into(),
            )
            .motsu_expect_err("should fail when beacon implementation reverts");

        assert_eq!(
            err,
            Error::FailedCallWithReason(address::FailedCallWithReason {
                reason: ImplementationError::ImplementationError(
                    ImplementationSolidityError {}
                )
                .encode()
                .into(),
            })
            .encode()
        );
    }

    #[motsu::test]
    fn upgrade_beacon_to_and_call_reverts_with_empty_data_with_value(
        contract: Contract<TestContract>,
        beacon: Contract<MockBeacon>,
        implementation: Contract<Implementation>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(implementation.address());

        alice.fund(uint!(1000_U256));

        let err = contract
            .sender_and_value(alice, uint!(1000_U256))
            .test_upgrade_beacon_to_and_call(beacon.address(), vec![].into())
            .motsu_expect_err("should fail with ERC1967NonPayable");

        assert_eq!(err, Error::NonPayable(ERC1967NonPayable {}).encode());
    }

    // test storage slot isolation.
    #[motsu::test]
    fn storage_slots_are_independent(
        contract: Contract<TestContract>,
        implementation: Contract<Implementation>,
        beacon: Contract<MockBeacon>,
        alice: Address,
        bob: Address,
    ) {
        beacon.sender(alice).constructor(implementation.address());

        // set implementation.
        contract
            .sender(alice)
            .test_upgrade_to_and_call(implementation.address(), vec![].into())
            .motsu_expect("should be able to set implementation");

        // set admin.
        contract
            .sender(alice)
            .test_change_admin(bob)
            .motsu_expect("should be able to set admin");

        // set beacon.
        contract
            .sender(alice)
            .test_upgrade_beacon_to_and_call(beacon.address(), vec![].into())
            .motsu_expect("should be able to set beacon");

        // verify all are set correctly.
        let implementation_addr =
            contract.sender(alice).test_get_implementation();
        let admin = contract.sender(alice).test_get_admin();
        let beacon_address = contract.sender(alice).test_get_beacon();

        assert_eq!(implementation_addr, implementation.address());
        assert_eq!(admin, bob);
        assert_eq!(beacon_address, beacon.address());
    }

    #[storage]
    struct InvalidBeacon;

    unsafe impl TopLevelStorage for InvalidBeacon {}

    #[public]
    impl InvalidBeacon {
        fn implementation(&self) -> Result<Address, Vec<u8>> {
            Err("Invalid implementation".into())
        }
    }

    #[motsu::test]
    fn get_beacon_implementation_errors_with_empty_result_and_no_code(
        contract: Contract<TestContract>,
        invalid_beacon: Contract<InvalidBeacon>,
        alice: Address,
    ) {
        // Use an EOA address (alice) as the beacon. A call to an address with
        // no code returns success with empty returndata. Combined with
        // target.has_code() == false,
        // AddressUtils::verify_call_result_from_target must return
        // AddressEmptyCode.
        let err = Erc1967Utils::get_beacon_implementation(
            &*contract.sender(alice),
            invalid_beacon.address(),
        )
        .motsu_expect_err(
            "expected EmptyCode when beacon call returns empty and has no code",
        );

        assert!(matches!(
            err,
            Error::FailedCallWithReason(address::FailedCallWithReason {
                reason,
            }) if reason.to_vec() == Into::<Vec<u8>>::into("Invalid implementation")
        ));
    }
}
