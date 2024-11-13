//! Common extensions to the ERC-20 standard.
pub mod burnable;
pub mod capped;
pub mod metadata;
pub mod permit;
pub mod vote;

pub use burnable::IErc20Burnable;
pub use capped::Capped;
pub use metadata::{Erc20Metadata, IErc20Metadata};
pub use permit::Erc20Permit;
//pub use vote::Erc20Vote;
