// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract UniswapV4HooksExample {
    mapping(bytes32 => uint256 count) public beforeSwapCount;
    mapping(bytes32 => uint256 count) public afterSwapCount;

    mapping(bytes32 => uint256 count) public beforeAddLiquidityCount;
    mapping(bytes32 => uint256 count) public beforeRemoveLiquidityCount;
}
