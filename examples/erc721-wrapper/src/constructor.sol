// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc721WrapperExample {
    mapping(address account => uint256) private _balances;
    mapping(address account => mapping(address spender => uint256))
        private _allowances;
    uint256 private _totalSupply;

    address private _underlyingToken;

    constructor(address underlyingToken_) {
        _underlyingToken = underlyingToken_;
    }
}
