
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc4626Example {
    address private _asset;

    mapping(address account => uint256) private _balances;
    mapping(address account => mapping(address spender => uint256))
        private _allowances;
    uint256 private _totalSupply;

    uint8 private _underlyingDecimals;


    constructor(address asset_) {
        _asset = asset_;
        _underlyingDecimals = 18;
    }
}
