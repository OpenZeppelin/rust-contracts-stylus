
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc4626Example {
    address private  _asset;
    uint8  private  _underlyingDecimals;
    string private _name;
    string private _symbol;


    constructor(address asset_, string memory name_, string memory symbol_) {
        _underlyingDecimals = 18;
        _asset = asset_;
        _name = name_;
        _symbol = symbol_;
    }
}
