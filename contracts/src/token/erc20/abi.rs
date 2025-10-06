//! Consolidated Solidity Interfaces for ERC-20 tokens.
//!
//! This module contains both contract interfaces and ABI interfaces:
//! - **Contract interfaces**: defined with
//!   [`stylus_sdk::prelude::sol_interface`], which enables invoking contract
//!   functions directly on actual deployed contracts
//! - **ABI interfaces**: defined with [`sol`], which enables constructing
//!   function call data to use with [`stylus_sdk::call::RawCall`]

#![allow(missing_docs)]
#![cfg_attr(coverage_nightly, coverage(off))]

use alloy_sol_types::sol;
pub use callable::*;

sol! {
    /// Interface of the ERC-20 token (ABI version).
    /// Complete ERC-20 standard as defined in EIP-20.
    interface Erc20Abi {
        function totalSupply() external view returns (uint256);
        function balanceOf(address account) external view returns (uint256);
        function transfer(address to, uint256 value) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 value) external returns (bool);
        function transferFrom(address from, address to, uint256 value) external returns (bool);
        function mint(address to, uint256 value) external;
    }
}

/// Contract interfaces defined with [`stylus_sdk::prelude::sol_interface`].
/// These enable invoking contract functions directly on actual deployed
/// contracts.
mod callable {
    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]

    use alloc::vec;

    use stylus_sdk::prelude::sol_interface;

    sol_interface! {
        /// ERC-20 standard interface.
        interface Erc20Interface {
            function totalSupply() external view returns (uint256);
            function balanceOf(address account) external view returns (uint256);
            function transfer(address to, uint256 value) external returns (bool);
            function allowance(address owner, address spender) external view returns (uint256);
            function approve(address spender, uint256 value) external returns (bool);
            function transferFrom(address from, address to, uint256 value) external returns (bool);
        }
    }

    sol_interface! {
        /// ERC-20 Metadata extension interface.
        interface Erc20MetadataInterface {
            function name() external view returns (string);
            function symbol() external view returns (string);
            function decimals() external view returns (uint8);
        }
    }

    sol_interface! {
        /// Interface of the ERC-3156 Flash Borrower, as defined in [ERC-3156].
        ///
        /// [ERC-3156]: https://eips.ethereum.org/EIPS/eip-3156
        interface Erc3156FlashBorrowerInterface {
            /// Receives a flash loan.
            ///
            /// To indicate successful handling of the flash loan, this function should return
            /// the `keccak256` hash of "ERC3156FlashBorrower.onFlashLoan".
            ///
            /// # Arguments
            ///
            /// * `initiator` - The initiator of the flash loan.
            /// * `token` - The token to be flash loaned.
            /// * `amount` - The amount of tokens lent.
            /// * `fee` - The additional amount of tokens to repay.
            /// * `data` - Arbitrary data structure, intended to contain user-defined parameters.
            #[allow(missing_docs)]
            function onFlashLoan(
                address initiator,
                address token,
                uint256 amount,
                uint256 fee,
                bytes calldata data
            ) external returns (bytes32);
        }
    }

    sol_interface! {
        /// Interface of the ERC-1363 Payable Token, as defined in [ERC-1363].
        ///
        /// [ERC-1363]: https://eips.ethereum.org/EIPS/eip-1363
        interface Erc1363Interface {
            function transferAndCall(address to, uint256 value, bytes calldata data) external returns (bool);
            function transferFromAndCall(address from, address to, uint256 value, bytes calldata data) external returns (bool);
            function approveAndCall(address spender, uint256 value, bytes calldata data) external returns (bool);
        }
    }
}
