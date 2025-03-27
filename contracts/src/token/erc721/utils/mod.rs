//! Utilities for the ERC-721 standard.
pub use token::*;

mod token {
    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]
    use alloc::vec;

    stylus_sdk::prelude::sol_interface! {
        /// Interface of the ERC-721 token.
        interface IErc721 {
            function ownerOf(uint256 token_id) external view returns (address);
            function safeTransferFrom(address from, address to, uint256 token_id) external;
            function transferFrom(address from, address to, uint256 token_id) external;
        }
    }
}
