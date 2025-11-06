//! This contract implements an upgradeable proxy. It is upgradeable because
//! calls are delegated to an implementation address that can be changed. This
//! address is stored in storage in the location specified by
//! [ERC-1967], so that it doesn't conflict with the storage layout of the
//! implementation behind the proxy.
//!
//! [ERC-1967]: https://eips.ethereum.org/EIPS/eip-1967
use alloc::{vec, vec::Vec};

use alloy_primitives::Address;
use stylus_sdk::{abi::Bytes, prelude::*};

use crate::proxy::IProxy;

pub mod utils;

pub use sol::*;
pub use utils::{Erc1967Utils, Error};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when the implementation is upgraded.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event Upgraded(address indexed implementation);

        /// Emitted when the admin account has changed.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event AdminChanged(address previous_admin, address new_admin);

        /// Emitted when the beacon is changed.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event BeaconUpgraded(address indexed beacon);
    }
}

/// State of an [`Erc1967Proxy`] token.
#[storage]
pub struct Erc1967Proxy;

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc1967Proxy {}

impl Erc1967Proxy {
    /// Initializes the upgradeable proxy with an initial implementation
    /// specified by `implementation`.
    ///
    /// If `data` is nonempty, it's used as data in a delegate call to
    /// `implementation`. This will typically be an encoded function call,
    /// and allows initializing the storage of the proxy like a Solidity
    /// constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `implementation` - Address of the implementation contract.
    /// * `data` - Data to pass to the implementation contract.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidImplementation`] - If the implementation is not a
    ///   valid implementation.
    /// * [`Error::NonPayable`] - If `data` is empty and
    ///   [`msg::value`][msg_value] is not [`U256::ZERO`][U256].
    /// * [`Error::FailedCall`] - If the call to the implementation fails.
    /// * [`Error::FailedCallWithReason`] - If the call to the implementation
    ///   fails with a revert reason.
    ///
    /// [msg_value]: stylus_sdk::msg::value
    /// [U256]: alloy_primitives::U256
    pub fn constructor(
        &mut self,
        implementation: Address,
        data: &Bytes,
    ) -> Result<(), Error> {
        Erc1967Utils::upgrade_to_and_call(self, implementation, data)
    }
}

unsafe impl IProxy for Erc1967Proxy {
    fn implementation(&self) -> Result<Address, Vec<u8>> {
        Ok(Erc1967Utils::get_implementation())
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
        proxy::tests::Erc20Example,
        token::erc20::{self, abi::Erc20Abi},
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
        ) -> Result<(), Error> {
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

    #[motsu::test]
    fn constructs(
        proxy: Contract<Erc1967ProxyExample>,
        erc20: Contract<Erc20Example>,
        alice: Address,
    ) {
        proxy
            .sender(alice)
            .constructor(erc20.address(), vec![].into())
            .motsu_expect("should be able to construct");

        let implementation = proxy
            .sender(alice)
            .implementation()
            .motsu_expect("should be able to get implementation");
        assert_eq!(implementation, erc20.address());
    }

    #[motsu::test]
    fn constructs_with_data(
        proxy: Contract<Erc1967ProxyExample>,
        erc20: Contract<Erc20Example>,
        alice: Address,
    ) {
        let amount = uint!(1000_U256);

        let data = Erc20Abi::mintCall { to: alice, value: amount }.abi_encode();

        proxy
            .sender(alice)
            .constructor(erc20.address(), data.into())
            .motsu_expect("should be able to construct");

        let implementation = proxy
            .sender(alice)
            .implementation()
            .motsu_expect("should be able to get implementation");
        assert_eq!(implementation, erc20.address());

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
        proxy: Contract<Erc1967ProxyExample>,
        erc20: Contract<Erc20Example>,
        alice: Address,
        bob: Address,
    ) {
        proxy
            .sender(alice)
            .constructor(erc20.address(), vec![].into())
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
        proxy: Contract<Erc1967ProxyExample>,
        erc20: Contract<Erc20Example>,
        alice: Address,
        bob: Address,
    ) {
        proxy
            .sender(alice)
            .constructor(erc20.address(), vec![].into())
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
