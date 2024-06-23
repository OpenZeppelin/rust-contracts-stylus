//! Common extensions to the ERC-20 standard.

pub mod metadata;
pub use metadata::{Erc20Metadata, IErc20Metadata};
pub mod burnable;
pub use burnable::IErc20Burnable;
pub mod capped;
pub use capped::Capped;
