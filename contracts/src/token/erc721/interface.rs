//! Solidity Interface of the ERC-721 token.
pub use interface::*;

mod interface {
    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]
    use alloc::vec;

    stylus_sdk::prelude::sol_interface! {
        interface Erc721Interface {
            function balanceOf(address owner) external view returns (uint256 balance);
            function ownerOf(uint256 token_id) external view returns (address owner);
            function safeTransferFrom(address from, address to, uint256 token_id, bytes calldata data) external;
            function transferFrom(address from, address to, uint256 token_id) external;
            function approve(address to, uint256 token_id) external;
            function setApprovalForAll(address operator, bool approved) external;
            function getApproved(uint256 token_id) external view returns (address operator);
            function isApprovedForAll(address owner, address operator) external view returns (bool);
        }
    }
}
