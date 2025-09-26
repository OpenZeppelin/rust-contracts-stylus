//! Consolidated Solidity Interfaces for ERC-721 tokens.
//!
//! This module contains both callable and non-callable interfaces:
//! - **Callable interfaces**: defined with `stylus_proc::sol_interface`, which
//!   enables invoking contract functions directly
//! - **Non-callable interfaces**: defined with `alloy_sol_types::sol`, which
//!   enables constructing function call data to use with `RawCall`

pub use callable::*;

/// Callable interfaces defined with `stylus_proc::sol_interface`.
/// These enable invoking contract functions directly on the interface.
mod callable {
    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]

    use alloc::vec;

    use stylus_sdk::prelude::sol_interface;

    sol_interface! {
        /// ERC-721 standard interface.
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

    sol_interface! {
        /// ERC-721 token receiver Solidity interface.
        ///
        /// Check [`crate::token::erc721::IErc721Receiver`] trait for more details.
        interface IErc721ReceiverInterface {
            /// See [`crate::token::erc721::IErc721Receiver::on_erc721_received`].
            #[allow(missing_docs)]
            function onERC721Received(
                address operator,
                address from,
                uint256 token_id,
                bytes calldata data
            ) external returns (bytes4);
        }
    }
}
