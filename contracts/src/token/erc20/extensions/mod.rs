//! Common extensions to the ERC-20 standard.

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc20_metadata"))] {
        pub mod metadata;
        pub use metadata::{IErc20Metadata, Erc20Metadata};
    }
}
cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc20_burnable"))] {
        pub mod burnable;
        pub use burnable::IErc20Burnable;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(test, feature = "erc20_capped"))] {
        pub mod capped;
        pub use capped::Capped;
    }
}
