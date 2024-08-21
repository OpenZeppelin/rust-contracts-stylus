//! Common Smart Contracts utilities.
pub mod cryptography;
pub mod math;
pub mod metadata;
pub mod nonces;
pub mod pausable;
pub mod structs;
pub use metadata::Metadata;
pub use pausable::Pausable;

/// Implement [`stylus_sdk::call::MethodError`] trait for the error type.
/// Will make it possible to reuse error in the other contract.
#[macro_export]
macro_rules! impl_method_error {
    ($error_ty:ty) => {
        impl stylus_sdk::call::MethodError for $error_ty {
            fn encode(self) -> alloc::vec::Vec<u8> {
                self.into()
            }
        }
    };
}
