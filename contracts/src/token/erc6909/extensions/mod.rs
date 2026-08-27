//! Common extensions to the ERC-6909 standard.
pub mod content_uri;
pub mod token_supply;

pub use content_uri::{Erc6909ContentUri, IErc6909ContentUri};
pub use token_supply::{Erc6909TokenSupply, IErc6909TokenSupply};
