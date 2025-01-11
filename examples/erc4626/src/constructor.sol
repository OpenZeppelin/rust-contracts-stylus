
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc4626Example {
    mapping(address account => uint256) private _balances;
    mapping(address account => mapping(address spender => uint256)) private _allowances;
    uint256 private _totalSupply;

    address private _token_address;
    uint8  private  _underlyingDecimals;
    string private _name;
    string private _symbol;


    constructor(address  token_address_, string memory name_, string memory symbol_) {
        _underlyingDecimals = 18;
        _token_address = token_address_;
        _name = name_;
        _symbol = symbol_;
    }
}
