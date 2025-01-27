// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc20WrapperExample {

    
    mapping(address account => uint256) private _balances;
    mapping(address account => mapping(address spender => uint256))
        private _allowances;
    uint256 private _totalSupply;
    string private _name;
    string private _symbol;


    address private _underlyingToken;


    constructor(string memory name_, string memory symbol_, address underlyingToken_) {
        _name = name_;
        _symbol = symbol_;
        _underlyingToken = underlyingToken_;
    }
}
