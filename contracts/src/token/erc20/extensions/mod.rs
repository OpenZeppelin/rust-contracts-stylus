//! Common extensions to the ERC-20 standard.
pub mod burnable;
pub mod capped;
pub mod flash_mint;
pub mod metadata;
pub mod permit;
pub mod wrapper;

pub use burnable::IErc20Burnable;
pub use capped::Capped;
pub use flash_mint::{Erc20FlashMint, IErc3156FlashLender};
pub use metadata::{Erc20Metadata, IErc20Metadata};
pub use permit::Erc20Permit;
pub use wrapper::{IERC20Wrapper,Erc20Wrapper};
