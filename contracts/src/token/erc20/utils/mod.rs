//! Utilities for the ERC-20 standard.
pub mod safe_erc20;

pub use safe_erc20::SafeErc20;
pub use token::*;
mod token {

    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]

    use alloc::vec;

    use stylus_sdk::stylus_proc::sol_interface;

    sol_interface! {
        /// Solidity Interface of the ERC-20 token.
        interface IErc20 {
            function balanceOf(address account) external view returns (uint256);
            function totalSupply() external view returns (uint256);
        }
    }
}
