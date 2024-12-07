// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc20FlashMintExample {
    mapping(address => uint256) private _balances;
    mapping(address => mapping(address => uint256)) private _allowances;

    uint256 private _totalSupply;
    uint256 private _flash_fee_amount;
    address private _flash_fee_receiver_address;

    constructor(address flash_fee_receiver_address_, uint256 flash_fee_amount_) {
        _flash_fee_receiver_address = flash_fee_receiver_address_;
        _flash_fee_amount = flash_fee_amount_;
    }
}
