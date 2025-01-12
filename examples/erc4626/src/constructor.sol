
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc4626Example {
    mapping(address account => uint256) private _balances;
    mapping(address account => mapping(address spender => uint256)) private _allowances;
    uint256 private _totalSupply;

    address private _asset;
    uint8  private  _underlyingDecimals;
    string private _name;
    string private _symbol;


    constructor(string memory name_, string memory symbol_, address asset_) {
        _underlyingDecimals = 18;
        _asset = asset_;
        _name = name_;
        _symbol = symbol_;
    }
}
