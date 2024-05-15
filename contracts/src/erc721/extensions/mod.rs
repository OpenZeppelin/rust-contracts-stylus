//! Common extensions to the ERC-721 standard.

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc721_metadata"))] {
        pub mod metadata;
        pub use metadata::ERC721Metadata;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc721_uri_storage"))] {
        pub mod uri_storage;
        pub use uri_storage::ERC721UriStorage;
    }
}
