//! Common extensions to the ERC-20 standard.
pub mod burnable;
pub mod capped;
pub mod erc4626;
pub mod flash_mint;
pub mod metadata;
pub mod permit;

pub use burnable::IErc20Burnable;
pub use capped::Capped;
pub use erc4626::{Erc4626, IErc4626};
pub use flash_mint::{Erc20FlashMint, IErc3156FlashLender};
pub use metadata::{Erc20Metadata, IErc20Metadata};
pub use permit::Erc20Permit;
