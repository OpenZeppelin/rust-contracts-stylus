use alloy_primitives::Address;
use stylus_sdk::{call::RawCall, prelude::*, ArbResult};

pub mod beacon;
pub mod erc1967;

/**
 * @dev This abstract contract provides a fallback function that delegates
 * all calls to another contract using the EVM instruction `delegatecall`.
 * We refer to the second contract as the _implementation_ behind the proxy,
 * and it has to be specified by overriding the virtual {_implementation}
 * function.
 *
 * Additionally, delegation to the implementation can be triggered manually
 * through the {_fallback} function, or to a different contract through the
 * {_delegate} function.
 *
 * The success and return data of the delegated call will be returned back
 * to the caller of the proxy.
 */
pub trait IProxy: TopLevelStorage {
    /**
     * @dev Delegates the current call to `implementation`.
     *
     * This function does not return to its internal call site, it will
     * return directly to the external caller.
     */
    fn delegate(
        &mut self,
        implementation: Address,
        calldata: &[u8],
    ) -> ArbResult {
        unsafe {
            RawCall::new_delegate()
                .flush_storage_cache()
                .call(implementation, calldata)
        }
    }

    /**
     * @dev This is a virtual function that should be overridden so it
     * returns the address to which the fallback function and
     * {_fallback} should delegate.
     */
    fn implementation(&self) -> Address;

    /**
     * @dev Fallback function that delegates calls to the address returned
     * by `_implementation()`. Will run if no other function in the
     * contract matches the call data.
     */
    fn do_fallback(&mut self, calldata: &[u8]) -> ArbResult {
        self.delegate(self.implementation(), calldata)
    }
}
