//! Utilities for the ERC-20 standard.
pub mod safe_erc20;

pub use safe_erc20::{ISafeErc20, SafeErc20};
pub use token::*;
mod token {

    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]

    use alloc::vec;

    use alloy_sol_types::sol;
    use stylus_sdk::stylus_proc::sol_interface;

    sol_interface! {
        /// Solidity Interface of the ERC-20 token.
        interface IErc20 {
            function balanceOf(address account) external view returns (uint256);
            function totalSupply() external view returns (uint256);
        }
    }

    sol! {
        /// Solidity Interface of the ERC-20 Metadata token.
        interface IERC20Metadata {
            function name() external view returns (string memory);
            function symbol() external view returns (string memory);
            function decimals() external view returns (uint8);
        }
    }
}
