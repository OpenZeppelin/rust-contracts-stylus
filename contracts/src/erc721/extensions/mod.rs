//! Common extensions to the ERC-721 standard.

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc721_burnable"))] {
        pub mod burnable;
        pub use burnable::IErc721Burnable;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc721_enumerable"))] {
        pub mod enumerable;
        pub use enumerable::Erc721Enumerable;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc721_metadata"))] {
        pub mod metadata;
        pub use metadata::Erc721Metadata;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc721_uri_storage"))] {
        pub mod uri_storage;
        pub use uri_storage::Erc721UriStorage;
    }
}
