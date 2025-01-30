// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc4626Example {
    // Erc20 Token Storage
    mapping(address account => uint256) private _balances;
    mapping(address account => mapping(address spender => uint256))
        private _allowances;
    uint256 private _totalSupply;

    // Erc20 Metadata Storage
    string private _name;
    string private _symbol;

    // Erc4626 Storage
    address private _asset;
    uint8 private _underlyingDecimals;

    constructor(string memory name_, string memory symbol_, address asset_, uint8 underlyingDecimals_) {
        _name = name_;
        _symbol = symbol_;
        _asset = asset_;
        _underlyingDecimals = underlyingDecimals_;
    }
}
