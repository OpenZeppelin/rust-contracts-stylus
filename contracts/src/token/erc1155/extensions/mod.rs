//! Common extensions to the ERC-1155 standard.
pub mod burnable;
pub mod metadata_uri;

pub use burnable::IErc1155Burnable;
pub use metadata_uri::{Erc1155MetadataUri, IErc1155MetadataUri};
