//! Common extensions to the ERC-20 standard.

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc20_metadata"))] {
        pub mod metadata;
        pub use metadata::Metadata;
    }
}
cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc20_burnable"))] {
        pub mod burnable;
    }
}
