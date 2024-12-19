// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract Erc20FlashMintExample {
    mapping(address => uint256) private _balances;
    mapping(address => mapping(address => uint256)) private _allowances;
    uint256 private _totalSupply;

    uint256 private _flashFeeAmount;
    address private _flashFeeReceiverAddress;

    constructor(address flashFeeReceiverAddress_, uint256 flashFeeAmount_) {
        _flashFeeReceiverAddress = flashFeeReceiverAddress_;
        _flashFeeAmount = flashFeeAmount_;
    }
}
