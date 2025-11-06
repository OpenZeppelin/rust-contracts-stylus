//! This is a low-level set of contracts implementing different proxy patterns
//! with and without upgradeability.
use alloc::vec::Vec;

use alloy_primitives::Address;
use stylus_sdk::{
    call::{self, Call, Error},
    prelude::*,
};

pub mod abi;
pub mod beacon;
pub mod erc1967;
pub mod utils;

/// This trait provides a fallback function that delegates all calls to another
/// contract using the Stylus [`delegate_call`][delegate_call] function. We
/// refer to the second contract as the _implementation_ behind the proxy, and
/// it has to be specified by overriding the virtual [`IProxy::implementation`]
/// function.
///
/// Additionally, delegation to the implementation can be triggered manually
/// through the [`IProxy::do_fallback`] function, or to a different contract
/// through the [`IProxy::delegate`] function.
///
/// The success and return data of the delegated call will be returned back
/// to the caller of the proxy.
///
/// # Safety
///
/// This trait is unsafe to implement because it uses the `unsafe`
/// [`delegate_call`][delegate_call] function.
///
/// The caller must ensure that `self` is a valid contract storage context.
///
/// The caller must ensure that the implementation contract is a valid contract
/// address.
///
/// [delegate_call]: stylus_sdk::call::delegate_call
pub unsafe trait IProxy: TopLevelStorage + Sized {
    /// Delegates the current call to [`IProxy::implementation`].
    ///
    /// This function does not return to its internal call site, it will
    /// return directly to the external caller.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `implementation` - The address of the implementation contract.
    /// * `calldata` - The calldata to delegate to the implementation contract.
    ///
    /// # Errors
    ///
    /// Returns a [`stylus_sdk::call::Error`] if the delegate call fails. This
    /// error may represent:
    /// * A revert from the implementation contract, containing the revert data.
    /// * Failure to decode the return data from the implementation contract.
    /// * Other low-level call failures as defined by the Stylus SDK.
    #[allow(clippy::missing_safety_doc)]
    unsafe fn delegate(
        &mut self,
        implementation: Address,
        calldata: &[u8],
    ) -> Result<Vec<u8>, Error> {
        call::delegate_call(Call::new_in(self), implementation, calldata)
    }

    /// This is a virtual function that should be overridden so it
    /// returns the address to which the fallback function and
    /// [`IProxy::do_fallback`] should delegate.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// Implementing contracts should define their own error types for this
    /// function. Typically, errors may include:
    /// * The implementation address is invalid (e.g., not a contract).
    /// * The implementation is not a contract.
    ///
    /// The error should be encoded as a `Vec<u8>`.
    fn implementation(&self) -> Result<Address, Vec<u8>>;

    /// Fallback function that delegates calls to the address returned
    /// by [`IProxy::implementation`]. Will run if no other function in the
    /// contract matches the call data.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `calldata` - The calldata to delegate to the implementation contract.
    ///
    /// # Errors
    ///
    /// Implementing contracts should define their own error types for this
    /// function. Typically, errors may include:
    /// * The implementation address is invalid (e.g., not a contract).
    /// * The implementation is not a contract.
    /// * The implementation reverted with a reason.
    ///
    /// The error should be encoded as a `Vec<u8>`.
    #[allow(clippy::missing_safety_doc)]
    unsafe fn do_fallback(
        &mut self,
        calldata: &[u8],
    ) -> Result<Vec<u8>, Vec<u8>> {
        Ok(self.delegate(self.implementation()?, calldata)?)
    }
}

#[cfg(test)]
mod tests {
    use alloy_sol_types::{SolCall, SolError, SolValue};
    use motsu::prelude::*;
    use stylus_sdk::{
        alloy_primitives::{uint, Address, U256},
        prelude::*,
        storage::{StorageAddress, StorageBool},
        ArbResult,
    };

    use super::*;
    use crate::token::erc20::{self, abi::Erc20Abi, Erc20, IErc20};

    #[entrypoint]
    #[storage]
    struct ProxyExample {
        implementation: StorageAddress,
        error_on_implementation: StorageBool,
    }

    #[public]
    impl ProxyExample {
        #[constructor]
        fn constructor(&mut self, implementation: Address) {
            self.implementation.set(implementation);
        }

        fn set_error_on_implementation(
            &mut self,
            error_on_implementation: bool,
        ) {
            self.error_on_implementation.set(error_on_implementation);
        }

        fn implementation(&self) -> Result<Address, Vec<u8>> {
            if self.error_on_implementation.get() {
                return Err("implementation error".abi_encode());
            }
            Ok(self.implementation.get())
        }

        #[fallback]
        fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
            unsafe { self.do_fallback(calldata) }
        }
    }

    unsafe impl IProxy for ProxyExample {
        fn implementation(&self) -> Result<Address, Vec<u8>> {
            Self::implementation(self)
        }
    }

    #[storage]
    pub(crate) struct Erc20Example {
        erc20: Erc20,
    }

    #[public]
    #[implements(IErc20<Error = erc20::Error>)]
    impl Erc20Example {
        fn mint(
            &mut self,
            to: Address,
            value: U256,
        ) -> Result<(), erc20::Error> {
            self.erc20._mint(to, value)
        }
    }

    unsafe impl TopLevelStorage for Erc20Example {}

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[public]
    impl IErc20 for Erc20Example {
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

    #[motsu::test]
    fn constructs(
        proxy: Contract<ProxyExample>,
        erc20: Contract<Erc20Example>,
        alice: Address,
    ) {
        proxy.sender(alice).constructor(erc20.address());

        let implementation = proxy
            .sender(alice)
            .implementation()
            .motsu_expect("should be able to get implementation");
        assert_eq!(implementation, erc20.address());
    }

    #[motsu::test]
    fn delegate(
        proxy: Contract<ProxyExample>,
        erc20: Contract<Erc20Example>,
        alice: Address,
        bob: Address,
    ) {
        proxy.sender(alice).constructor(erc20.address());

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
    fn delegate_returns_error(
        proxy: Contract<ProxyExample>,
        erc20: Contract<Erc20Example>,
        alice: Address,
        bob: Address,
    ) {
        proxy.sender(alice).constructor(erc20.address());

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

    #[motsu::test]
    fn direct_delegate_to_different_implementation(
        proxy: Contract<ProxyExample>,
        erc20: Contract<Erc20Example>,
        erc20_2: Contract<Erc20Example>,
        alice: Address,
    ) {
        proxy.sender(alice).constructor(erc20.address());

        // mint tokens to the second contract.
        let amount = uint!(500_U256);
        erc20_2
            .sender(alice)
            .mint(alice, amount)
            .motsu_expect("should be able to mint");

        // use direct delegate to call the second contract.
        let balance_of_call =
            Erc20Abi::balanceOfCall { account: alice }.abi_encode();
        let balance = unsafe {
            proxy
                .sender(alice)
                .delegate(erc20_2.address(), &balance_of_call)
                .motsu_expect(
                    "should be able to delegate to different implementation",
                )
        };
        assert_eq!(balance, amount.abi_encode());
    }

    #[motsu::test]
    fn fallback_reverts_on_implementation_error(
        proxy: Contract<ProxyExample>,
        alice: Address,
    ) {
        // create proxy with [`Address::ZERO`] as implementation (invalid).
        proxy.sender(alice).set_error_on_implementation(true);

        let balance_of_call =
            Erc20Abi::balanceOfCall { account: alice }.abi_encode();
        let err =
            proxy.sender(alice).fallback(&balance_of_call).motsu_expect_err(
                "should fail when implementation is zero address",
            );

        assert_eq!(err, "implementation error".abi_encode());
    }
}
