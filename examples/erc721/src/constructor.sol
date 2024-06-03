// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc721Example {
    mapping(uint256 tokenId => address) private _owners;
    mapping(address owner => uint256) private _balances;
    mapping(uint256 tokenId => address) private _tokenApprovals;
    mapping(address owner => mapping(address operator => bool))
        private _operatorApprovals;

    mapping(address owner => mapping(uint256 index => uint256))
        private _ownedTokens;
    mapping(uint256 tokenId => uint256) private _ownedTokensIndex;
    uint256[] private _allTokens;
    mapping(uint256 tokenId => uint256) private _allTokensIndex;

    string private _name;
    string private _symbol;

    bool _paused;

    mapping(uint256 => string) _tokenUris;

    constructor(string memory name_, string memory symbol_) {
        _name = name_;
        _symbol = symbol_;
        _paused = false;
    }
}
