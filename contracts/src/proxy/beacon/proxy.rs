//! This contract implements a proxy that gets the implementation address for
//! each call from an [`UpgradeableBeacon`][UpgradeableBeacon].
//!
//! The beacon address can only be set once during construction, and cannot be
//! changed afterwards. It is stored in an immutable variable to avoid
//! unnecessary storage reads, and also in the beacon storage slot specified by
//! [ERC-1967] so that it can be accessed externally.
//!
//! CAUTION: Since the beacon address can never be changed, you must ensure that
//! you either control the beacon, or trust the beacon to not upgrade the
//! implementation maliciously.
//!
//! IMPORTANT: Do not use the implementation logic to modify the beacon storage
//! slot. Doing so would leave the proxy in an inconsistent state where the
//! beacon storage slot does not match the beacon address.
//!
//! [UpgradeableBeacon]: super::UpgradeableBeacon
//! [ERC-1967]: https://eips.ethereum.org/EIPS/eip-1967

use alloc::{vec, vec::Vec};

use alloy_primitives::Address;
use stylus_sdk::{abi::Bytes, prelude::*, storage::StorageAddress};

use crate::proxy::{
    beacon::BeaconInterface,
    erc1967::{Erc1967Utils, Error},
    IProxy,
};

/// State of an [`BeaconProxy`] contract.
#[storage]
pub struct BeaconProxy {
    beacon: StorageAddress,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for BeaconProxy {}

impl BeaconProxy {
    /// Initializes the proxy with `beacon`.
    ///
    /// If `data` is nonempty, it's used as data in a delegate call to the
    /// implementation returned by the beacon. This will typically be an
    /// encoded function call, and allows initializing the storage of the proxy
    /// like a Solidity constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `beacon` - The beacon address.
    /// * `data` - The data to pass to the beacon.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidBeacon`] - If the beacon is not a contract with the
    ///   interface [`IBeacon`][IBeacon].
    /// * [`Error::NonPayable`] - If the data is empty and
    ///   [`msg::value`][msg_value] is not [`U256::ZERO`][U256].
    ///
    /// [msg_value]: stylus_sdk::msg::value
    /// [IBeacon]: super::IBeacon
    /// [U256]: alloy_primitives::U256
    pub fn constructor(
        &mut self,
        beacon: Address,
        data: &Bytes,
    ) -> Result<(), Error> {
        Erc1967Utils::upgrade_beacon_to_and_call(self, beacon, data)?;
        self.beacon.set(beacon);
        Ok(())
    }

    /// Returns the beacon.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[must_use]
    pub fn get_beacon(&self) -> Address {
        self.beacon.get()
    }
}

unsafe impl IProxy for BeaconProxy {
    fn implementation(&self) -> Result<Address, Vec<u8>> {
        Ok(BeaconInterface::new(self.get_beacon()).implementation(self)?)
    }
}

#[cfg(test)]
#[allow(clippy::needless_pass_by_value)]
mod tests {
    use alloy_sol_types::{SolCall, SolError, SolValue};
    use motsu::prelude::*;
    use stylus_sdk::{
        alloy_primitives::{uint, Address, U256},
        prelude::*,
        ArbResult,
    };

    use super::*;
    use crate::{
        proxy::{beacon::IBeacon, tests::Erc20Example},
        token::erc20::{self, abi::Erc20Abi},
    };

    #[entrypoint]
    #[storage]
    struct BeaconProxyExample {
        beacon_proxy: BeaconProxy,
    }

    #[public]
    impl BeaconProxyExample {
        #[constructor]
        fn constructor(
            &mut self,
            beacon: Address,
            data: Bytes,
        ) -> Result<(), Error> {
            self.beacon_proxy.constructor(beacon, &data)
        }

        fn get_beacon(&self) -> Address {
            self.beacon_proxy.get_beacon()
        }

        fn implementation(&self) -> Result<Address, Vec<u8>> {
            self.beacon_proxy.implementation()
        }

        #[fallback]
        fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
            unsafe { self.beacon_proxy.do_fallback(calldata) }
        }
    }

    #[storage]
    struct Beacon {
        implementation: StorageAddress,
    }

    unsafe impl TopLevelStorage for Beacon {}

    #[public]
    #[implements(IBeacon)]
    impl Beacon {
        #[constructor]
        fn constructor(&mut self, implementation: Address) {
            self.implementation.set(implementation);
        }
    }

    #[public]
    impl IBeacon for Beacon {
        fn implementation(&self) -> Result<Address, Vec<u8>> {
            Ok(self.implementation.get())
        }
    }

    #[motsu::test]
    fn constructs(
        proxy: Contract<BeaconProxyExample>,
        beacon: Contract<Beacon>,
        erc20: Contract<Erc20Example>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address());

        proxy
            .sender(alice)
            .constructor(beacon.address(), vec![].into())
            .motsu_unwrap();

        let implementation = proxy
            .sender(alice)
            .implementation()
            .motsu_expect("should be able to get implementation");
        assert_eq!(implementation, erc20.address());

        let beacon_address = proxy.sender(alice).get_beacon();
        assert_eq!(beacon_address, beacon.address());
    }

    #[motsu::test]
    fn constructs_with_data(
        proxy: Contract<BeaconProxyExample>,
        beacon: Contract<Beacon>,
        erc20: Contract<Erc20Example>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address());

        let amount = uint!(1000_U256);

        let data = Erc20Abi::mintCall { to: alice, value: amount }.abi_encode();

        proxy
            .sender(alice)
            .constructor(beacon.address(), data.into())
            .motsu_expect("should be able to construct");

        let implementation = proxy
            .sender(alice)
            .implementation()
            .motsu_expect("should be able to get implementation");
        assert_eq!(implementation, erc20.address());

        let beacon_address = proxy.sender(alice).get_beacon();
        assert_eq!(beacon_address, beacon.address());

        let balance_of_alice_call =
            Erc20Abi::balanceOfCall { account: alice }.abi_encode();
        let balance = proxy
            .sender(alice)
            .fallback(&balance_of_alice_call)
            .motsu_expect("should be able to get balance");
        assert_eq!(balance, amount.abi_encode());

        let total_supply_call = Erc20Abi::totalSupplyCall {}.abi_encode();
        let total_supply = proxy
            .sender(alice)
            .fallback(&total_supply_call)
            .motsu_expect("should be able to get total supply");
        assert_eq!(total_supply, amount.abi_encode());
    }

    #[motsu::test]
    fn fallback(
        proxy: Contract<BeaconProxyExample>,
        beacon: Contract<Beacon>,
        erc20: Contract<Erc20Example>,
        alice: Address,
        bob: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address());

        proxy
            .sender(alice)
            .constructor(beacon.address(), vec![].into())
            .motsu_expect("should be able to construct");

        // verify initial balance is [`U256::ZERO`].
        let balance_of_alice_call =
            Erc20Abi::balanceOfCall { account: alice }.abi_encode();
        let balance = proxy
            .sender(alice)
            .fallback(&balance_of_alice_call)
            .motsu_expect("should be able to get balance");
        assert_eq!(balance, U256::ZERO.abi_encode());

        let total_supply_call = Erc20Abi::totalSupplyCall {}.abi_encode();
        let total_supply = proxy
            .sender(alice)
            .fallback(&total_supply_call)
            .motsu_expect("should be able to get total supply");
        assert_eq!(total_supply, U256::ZERO.abi_encode());

        // mint 1000 tokens.
        let amount = uint!(1000_U256);

        let mint_call =
            Erc20Abi::mintCall { to: alice, value: amount }.abi_encode();
        proxy
            .sender(alice)
            .fallback(&mint_call)
            .motsu_expect("should be able to mint");
        // TODO: this should assert that the transfer event was emitted on the
        // proxy
        // https://github.com/OpenZeppelin/stylus-test-helpers/issues/111
        erc20.assert_emitted(&erc20::Transfer {
            from: Address::ZERO,
            to: alice,
            value: amount,
        });

        // check that the balance can be accurately fetched through the proxy.
        let balance = proxy
            .sender(alice)
            .fallback(&balance_of_alice_call)
            .motsu_expect("should be able to get balance");
        assert_eq!(balance, amount.abi_encode());

        let total_supply = proxy
            .sender(alice)
            .fallback(&total_supply_call)
            .motsu_expect("should be able to get total supply");
        assert_eq!(total_supply, amount.abi_encode());

        // check that the balance can be transferred through the proxy.
        let transfer_call =
            Erc20Abi::transferCall { to: bob, value: amount }.abi_encode();
        proxy
            .sender(alice)
            .fallback(&transfer_call)
            .motsu_expect("should be able to transfer");

        // TODO: this should assert that the transfer event was emitted on the
        // proxy
        // https://github.com/OpenZeppelin/stylus-test-helpers/issues/111
        erc20.assert_emitted(&erc20::Transfer {
            from: alice,
            to: bob,
            value: amount,
        });

        let balance = proxy
            .sender(alice)
            .fallback(&balance_of_alice_call)
            .motsu_expect("should be able to get balance");
        assert_eq!(balance, U256::ZERO.abi_encode());

        let balance_of_bob_call =
            Erc20Abi::balanceOfCall { account: bob }.abi_encode();
        let balance = proxy
            .sender(alice)
            .fallback(&balance_of_bob_call)
            .motsu_expect("should be able to get balance");
        assert_eq!(balance, amount.abi_encode());

        let total_supply = proxy
            .sender(alice)
            .fallback(&total_supply_call)
            .motsu_expect("should be able to get total supply");
        assert_eq!(total_supply, amount.abi_encode());
    }

    #[motsu::test]
    fn fallback_returns_error(
        proxy: Contract<BeaconProxyExample>,
        beacon: Contract<Beacon>,
        erc20: Contract<Erc20Example>,
        alice: Address,
        bob: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address());

        proxy
            .sender(alice)
            .constructor(beacon.address(), vec![].into())
            .motsu_expect("should be able to construct");

        let amount = uint!(1000_U256);
        let transfer_call =
            Erc20Abi::transferCall { to: bob, value: amount }.abi_encode();
        let err = proxy
            .sender(alice)
            .fallback(&transfer_call)
            .motsu_expect_err("should revert on transfer");

        assert_eq!(
            err,
            erc20::ERC20InsufficientBalance {
                sender: alice,
                balance: U256::ZERO,
                needed: amount,
            }
            .abi_encode()
        );
    }
}
