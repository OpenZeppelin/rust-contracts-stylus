//! Common Smart Contracts utilities.

pub mod math;
pub mod structs;

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc20_metadata", feature = "erc721_metadata"))] {
        pub mod metadata;
        pub use metadata::Metadata;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc20_pausable", feature = "erc721_pausable"))] {
        pub mod pausable;
        pub use pausable::Pausable;
    }
}
