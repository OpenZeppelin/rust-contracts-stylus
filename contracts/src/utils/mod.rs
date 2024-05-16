//! Common Smart Contracts utilities.

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc20_metadata", feature = "erc721_metadata"))] {
        pub mod metadata;
        pub use metadata::Metadata;
    }
}
