// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc721Example {
    mapping(uint256 tokenId => address) private _owners;
    mapping(address owner => uint256) private _balances;
    mapping(uint256 tokenId => address) private _tokenApprovals;
    mapping(address owner => mapping(address operator => bool))
    private _operatorApprovals;

    bool _paused;

    constructor() {
        _paused = false;
    }
}
