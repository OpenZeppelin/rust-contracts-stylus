//! This module provides functionality for handling contract calls with
//! safeguards for reentrancy.
//!
//! The [`ReentrantCallHandler`] trait allows performing raw contract calls
//! while managing reentrancy issues, particularly when the `reentrant` feature
//! is enabled. This ensures that storage aliasing does not occur during
//! reentrant calls by flushing storage caches before the call is made.
//!
//! The behavior of the trait method
//! [`ReentrantCallHandler::call_with_reentrant_handling`] varies depending on
//! the presence of the `reentrant` feature, providing either a safe or default
//! (unsafe) raw call mechanism. The module also interacts with raw calls and
//! storage management methods like [ReentrantCallHandler::flush_storage_cache]
//! to ensure that data integrity is maintained when making contract calls.
//!
//! For more details on the inner workings of raw calls and storage cache
//! management, see the documentation for [RawCall::call] and
//! [ReentrantCallHandler::flush_storage_cache].
//!
//! [RawCall::call]: https://docs.rs/stylus-sdk/0.6.0/stylus_sdk/call/struct.RawCall.html#method.call
//! [ReentrantCallHandler::flush_storage_cache]: https://docs.rs/stylus-sdk/0.6.0/stylus_sdk/call/struct.RawCall.html#method.flush_storage_cache

use alloy_primitives::Address;
use stylus_sdk::{call::RawCall, ArbResult};

/// A trait for handling calls that may require special handling for reentrancy.
///
/// This trait defines the method
/// [`ReentrantCallHandler::call_with_reentrant_handling`], which is intended to
/// perform a contract call with safeguards against reentrancy issues. The
/// behavior of the method can vary depending on whether the `reentrant` feature
/// is enabled:
///
/// - When the `reentrant` feature is enabled, the
///   [`ReentrantCallHandler::call_with_reentrant_handling`] method will ensure
///   that the storage cache is flushed before making the contract call to avoid
///   potential issues with aliasing storage during a reentrant call. This is
///   considered unsafe due to potential aliasing of storage in the middle of a
///   storage reference's lifetime. See the
///   [ReentrantCallHandler::flush_storage_cache] method for more details on
///   handling storage caches.
/// - When the `reentrant` feature is not enabled, the method simply makes the
///   call without any additional safeguards.
///
/// For more information on the safety of raw contract calls and storage
/// management, see:
/// - [RawCall::call]
/// - [ReentrantCallHandler::flush_storage_cache]
///
/// [RawCall::call]: https://docs.rs/stylus-sdk/0.6.0/stylus_sdk/call/struct.RawCall.html#method.call
/// [ReentrantCallHandler::flush_storage_cache]: https://docs.rs/stylus-sdk/0.6.0/stylus_sdk/call/struct.RawCall.html#method.flush_storage_cache
pub trait ReentrantCallHandler {
    /// Executes a contract call with reentrancy safeguards, returning the call
    /// result.
    ///
    /// This method performs a raw call to another contract using the provided
    /// `contract` and `call_data`. The method behavior changes based on the
    /// `reentrant` feature:
    ///
    /// - With the `reentrant` feature enabled, it flushes any cached storage
    ///   values before the call to prevent storage aliasing.
    /// - Without the `reentrant` feature, it makes the call directly without
    ///   additional safeguards.
    ///
    /// # Arguments
    ///
    /// * `contract` - The address of the contract being called.
    /// * `call_data` - The encoded data for the contract call.
    ///
    /// # Errors
    ///
    /// * Returns [`stylus_sdk::ArbResult`] indicating the success or failure of
    ///   the call.
    fn call_with_reentrant_handling(
        self,
        contract: Address,
        call_data: &[u8],
    ) -> ArbResult;
}

impl ReentrantCallHandler for RawCall {
    fn call_with_reentrant_handling(
        self,
        contract: Address,
        call_data: &[u8],
    ) -> ArbResult {
        #[cfg(feature = "reentrant")]
        unsafe {
            self.flush_storage_cache().call(contract, call_data)
        }
        #[cfg(not(feature = "reentrant"))]
        {
            self.call(contract, call_data)
        }
    }
}
