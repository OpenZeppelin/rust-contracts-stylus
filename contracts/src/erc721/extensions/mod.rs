//! Common extensions to the ERC-721 standard.

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc20_metadata"))] {
        pub mod metadata;
        pub use metadata::ERC721Metadata;
    }
}
