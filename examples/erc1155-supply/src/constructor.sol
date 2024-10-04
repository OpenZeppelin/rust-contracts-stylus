// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract Erc1155SupplyExample {
    mapping(address => mapping(uint256 => uint256)) private _balanceOf;
    mapping(address => mapping(address => bool)) private _isApprovedForAll;

    mapping(uint256 id => uint256) private _totalSupply;
    uint256 private _totalSupplyAll;

    constructor() {
        _totalSupplyAll = 0;
    }
}
