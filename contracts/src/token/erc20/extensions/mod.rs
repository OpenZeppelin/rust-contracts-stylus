//! Common extensions to the ERC-20 standard.
pub mod burnable;
pub mod capped;
pub mod flashmint;
pub mod metadata;
pub mod permit;

pub use burnable::IErc20Burnable;
pub use capped::Capped;
pub use flashmint::{Erc20Flashmint, IERC3156FlashLender};
pub use metadata::{Erc20Metadata, IErc20Metadata};
pub use permit::Erc20Permit;
