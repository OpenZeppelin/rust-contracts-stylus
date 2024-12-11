// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract Erc1155Example {
    mapping(address => mapping(uint256 => uint256)) private _balances;
    mapping(address => mapping(address => bool)) private _operatorApprovals;

    bool private _paused;

    constructor() {
        _paused = false;
    }
}
