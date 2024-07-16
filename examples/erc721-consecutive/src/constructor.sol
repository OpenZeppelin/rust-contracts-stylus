// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc721Example {
    mapping(uint256 tokenId => address) private _owners;
    mapping(address owner => uint256) private _balances;
    mapping(uint256 tokenId => address) private _tokenApprovals;
    mapping(address owner => mapping(address operator => bool))
    private _operatorApprovals;
    
    mapping(uint256 => uint256) private _data;
    Checkpoint160[] private _checkpoints;
    bool _initialized;

    struct Checkpoint160 {
        uint96 _key;
        uint160 _value;
    }

    constructor() {
        _initialized = false;
    }
}
