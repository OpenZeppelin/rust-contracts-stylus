//! This is a low-level set of contracts implementing different proxy patterns
//! with and without upgradeability.
use alloc::vec::Vec;

use alloy_primitives::Address;
use stylus_sdk::{
    call::{self, Call, Error},
    prelude::*,
};

pub mod beacon;
pub mod erc1967;

/// This trait provides a fallback function that delegates all calls to another
/// contract using the EVM instruction `delegatecall`. We refer to the second
/// contract as the _implementation_ behind the proxy, and it has to be
/// specified by overriding the virtual [`IProxy::implementation`] function.
///
/// Additionally, delegation to the implementation can be triggered manually
/// through the [`IProxy::do_fallback`] function, or to a different contract
/// through the [`IProxy::delegate`] function.
///
/// The success and return data of the delegated call will be returned back
/// to the caller of the proxy.
pub trait IProxy: TopLevelStorage + Sized {
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
    fn delegate(
        &mut self,
        implementation: Address,
        calldata: &[u8],
    ) -> Result<Vec<u8>, Error> {
        unsafe {
            call::delegate_call(Call::new_in(self), implementation, calldata)
        }
    }

    /// This is a virtual function that should be overridden so it
    /// returns the address to which the fallback function and
    /// [`IProxy::do_fallback`] should delegate.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn implementation(&self) -> Result<Address, Vec<u8>>;

    /// Fallback function that delegates calls to the address returned
    /// by [`IProxy::implementation`]. Will run if no other function in the
    /// contract matches the call data.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `calldata` - The calldata to delegate to the implementation contract.
    fn do_fallback(&mut self, calldata: &[u8]) -> Result<Vec<u8>, Vec<u8>> {
        Ok(self.delegate(self.implementation()?, calldata)?)
    }
}

#[cfg(test)]
mod tests {
    use alloy_sol_macro::sol;
    use alloy_sol_types::{SolCall, SolError, SolValue};
    use motsu::prelude::*;
    use stylus_sdk::{
        alloy_primitives::{Address, U256},
        prelude::*,
        storage::StorageAddress,
        ArbResult,
    };

    use super::*;
    use crate::token::erc20::{self, Erc20, IErc20};

    #[entrypoint]
    #[storage]
    struct ProxyExample {
        implementation: StorageAddress,
    }

    #[public]
    impl ProxyExample {
        #[constructor]
        fn constructor(&mut self, implementation: Address) {
            self.implementation.set(implementation);
        }

        fn implementation(&self) -> Result<Address, Vec<u8>> {
            IProxy::implementation(self)
        }

        #[fallback]
        fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
            self.do_fallback(calldata)
        }
    }

    impl IProxy for ProxyExample {
        fn implementation(&self) -> Result<Address, Vec<u8>> {
            Ok(self.implementation.get())
        }
    }

    #[storage]
    struct Erc20Example {
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

    sol! {
        interface IERC20 {
            function balanceOf(address account) external view returns (uint256);
            function totalSupply() external view returns (uint256);
            function mint(address to, uint256 value) external;
            function transfer(address to, uint256 value) external returns (bool);
        }
    }

    #[motsu::test]
    fn constructs(
        contract: Contract<ProxyExample>,
        erc20: Contract<Erc20Example>,
        alice: Address,
    ) {
        contract.sender(alice).constructor(erc20.address());

        let implementation = contract
            .sender(alice)
            .implementation()
            .expect("should be able to get implementation");
        assert_eq!(implementation, erc20.address());
    }

    #[motsu::test]
    fn delegate(
        contract: Contract<ProxyExample>,
        erc20: Contract<Erc20Example>,
        alice: Address,
        bob: Address,
    ) {
        contract.sender(alice).constructor(erc20.address());

        // verify initial balance is 0
        let balance_of_alice_call =
            IERC20::balanceOfCall { account: alice }.abi_encode();
        let balance = contract
            .sender(alice)
            .fallback(&balance_of_alice_call)
            .expect("should be able to get balance");
        assert_eq!(balance, U256::ZERO.abi_encode());

        let total_supply_call = IERC20::totalSupplyCall {}.abi_encode();
        let total_supply = contract
            .sender(alice)
            .fallback(&total_supply_call)
            .expect("should be able to get total supply");
        assert_eq!(total_supply, U256::ZERO.abi_encode());

        // mint 1000 tokens
        let amount = U256::from(1000);

        let mint_call =
            IERC20::mintCall { to: alice, value: amount }.abi_encode();
        contract
            .sender(alice)
            .fallback(&mint_call)
            .expect("should be able to mint");
        // TODO: this should assert that the transfer event was emitted on the
        // proxy
        // https://github.com/OpenZeppelin/stylus-test-helpers/issues/111
        erc20.assert_emitted(&erc20::Transfer {
            from: Address::ZERO,
            to: alice,
            value: amount,
        });

        // check that the balance can be accurately fetched through the proxy
        let balance = contract
            .sender(alice)
            .fallback(&balance_of_alice_call)
            .expect("should be able to get balance");
        assert_eq!(balance, amount.abi_encode());

        let total_supply = contract
            .sender(alice)
            .fallback(&total_supply_call)
            .expect("should be able to get total supply");
        assert_eq!(total_supply, amount.abi_encode());

        // check that the balance can be transferred through the proxy
        let transfer_call =
            IERC20::transferCall { to: bob, value: amount }.abi_encode();
        contract
            .sender(alice)
            .fallback(&transfer_call)
            .expect("should be able to transfer");

        // TODO: this should assert that the transfer event was emitted on the
        // proxy
        // https://github.com/OpenZeppelin/stylus-test-helpers/issues/111
        erc20.assert_emitted(&erc20::Transfer {
            from: alice,
            to: bob,
            value: amount,
        });

        let balance = contract
            .sender(alice)
            .fallback(&balance_of_alice_call)
            .expect("should be able to get balance");
        assert_eq!(balance, U256::ZERO.abi_encode());

        let balance_of_bob_call =
            IERC20::balanceOfCall { account: bob }.abi_encode();
        let balance = contract
            .sender(alice)
            .fallback(&balance_of_bob_call)
            .expect("should be able to get balance");
        assert_eq!(balance, amount.abi_encode());

        let total_supply = contract
            .sender(alice)
            .fallback(&total_supply_call)
            .expect("should be able to get total supply");
        assert_eq!(total_supply, amount.abi_encode());
    }

    #[motsu::test]
    fn delegate_returns_error(
        contract: Contract<ProxyExample>,
        erc20: Contract<Erc20Example>,
        alice: Address,
        bob: Address,
    ) {
        contract.sender(alice).constructor(erc20.address());

        let amount = U256::from(1000);
        let transfer_call =
            IERC20::transferCall { to: bob, value: amount }.abi_encode();
        let err = contract
            .sender(alice)
            .fallback(&transfer_call)
            .expect_err("should revert on transfer");

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
