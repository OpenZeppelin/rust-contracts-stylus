//! Proxy contracts.
use alloc::vec::Vec;

use alloy_primitives::Address;
use stylus_sdk::{
    call::{self, Call, Error},
    prelude::*,
};

pub mod beacon;
pub mod erc1967;

/// This trait provides a fallback function that delegates
/// all calls to another contract using the EVM instruction `delegatecall`.
/// We refer to the second contract as the _implementation_ behind the proxy,
/// and it has to be specified by overriding the virtual
/// [`IProxy::implementation`] function.
///
/// Additionally, delegation to the implementation can be triggered manually
/// through the [`IProxy::do_fallback`] function, or to a different contract
/// through the [`IProxy::delegate`] function.
///
/// The success and return data of the delegated call will be returned back
/// to the caller of the proxy.
pub trait IProxy: TopLevelStorage + Sized {
    /// Delegates the current call to `implementation`.
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
    /// `do_fallback` should delegate.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn implementation(&self) -> Result<Address, Error>;

    /// Fallback function that delegates calls to the address returned
    /// by `implementation()`. Will run if no other function in the
    /// contract matches the call data.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `calldata` - The calldata to delegate to the implementation contract.
    fn do_fallback(&mut self, calldata: &[u8]) -> Result<Vec<u8>, Error> {
        self.delegate(self.implementation()?, calldata)
    }
}
