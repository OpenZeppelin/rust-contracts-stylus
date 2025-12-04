//! Consolidated Solidity Interfaces for ERC-1155 tokens.
//!
//! This module contains both contract interfaces and ABI interfaces:
//! - **Contract interfaces**: defined with
//!   [`stylus_sdk::prelude::sol_interface`], which enables invoking contract
//!   functions directly on actual deployed contracts
//! - **ABI interfaces**: defined with [`alloy_sol_types::sol`], which enables
//!   constructing function call data to use with [`stylus_sdk::call::RawCall`]

pub use callable::*;

/// Contract interfaces defined with [`stylus_sdk::prelude::sol_interface`].
/// These enable invoking contract functions directly on actual deployed
/// contracts.
mod callable {
    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]

    use alloc::vec;

    use stylus_sdk::prelude::sol_interface;

    sol_interface! {
        /// ERC-1155 standard interface.
        interface Erc1155Interface {
            function balanceOf(address account, uint256 id) external view returns (uint256);
            function balanceOfBatch(address[] calldata accounts, uint256[] calldata ids) external view returns (uint256[] memory);
            function setApprovalForAll(address operator, bool approved) external;
            function isApprovedForAll(address account, address operator) external view returns (bool);
            function safeTransferFrom(address from, address to, uint256 id, uint256 value, bytes calldata data) external;
            function safeBatchTransferFrom(address from, address to, uint256[] calldata ids, uint256[] calldata values, bytes calldata data) external;
        }
    }

    sol_interface! {
        /// ERC-1155 token receiver Solidity interface.
        ///
        /// Check [`crate::token::erc1155::IErc1155Receiver`] trait for more details.
        interface Erc1155ReceiverInterface {
            /// See [`crate::token::erc1155::IErc1155Receiver::on_erc1155_received`].
            #[allow(missing_docs)]
            function onERC1155Received(
                address operator,
                address from,
                uint256 id,
                uint256 value,
                bytes calldata data
            ) external returns (bytes4);

            /// See [`crate::token::erc1155::IErc1155Receiver::on_erc1155_batch_received`].
            #[allow(missing_docs)]
            function onERC1155BatchReceived(
                address operator,
                address from,
                uint256[] calldata ids,
                uint256[] calldata values,
                bytes calldata data
            ) external returns (bytes4);
        }
    }
}
