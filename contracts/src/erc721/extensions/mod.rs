//! Common extensions to the ERC-721 standard.

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc721_burnable"))] {
        pub mod burnable;
        pub use burnable::IERC721Burnable;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc721_metadata"))] {
        pub mod metadata;
        pub use metadata::ERC721Metadata;
    }
}
