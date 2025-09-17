//! Common extensions to the ERC-721 standard.
pub mod burnable;
pub mod consecutive;
pub mod enumerable;
pub mod metadata;
pub mod uri_storage;
pub mod wrapper;

pub use burnable::IErc721Burnable;
pub use consecutive::Erc721Consecutive;
pub use enumerable::{Erc721Enumerable, IErc721Enumerable};
pub use metadata::{Erc721Metadata, IErc721Metadata};
pub use uri_storage::{Erc721UriStorage, IErc721UriStorage};
pub use wrapper::{Erc721Wrapper, IErc721Wrapper};
