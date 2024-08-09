// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc20PermitExample {
    mapping(address account => uint256) private _balances;
    mapping(address account => mapping(address spender => uint256))
        private _allowances;
    uint256 private _totalSupply;
    mapping(address account => uint256) _nonces;

    constructor() {}
}
