//! Common extensions to the ERC-721 standard.

pub mod burnable;
pub use burnable::IErc721Burnable;
pub mod enumerable;
pub use enumerable::{Erc721Enumerable, IErc721Enumerable};
pub mod metadata;
pub use metadata::{Erc721Metadata, IErc721Metadata};
pub mod uri_storage;
pub use uri_storage::Erc721UriStorage;
