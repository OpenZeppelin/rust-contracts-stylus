// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract ERC721Example {
    mapping(uint256 tokenId => address) private _owners;
    mapping(address owner => uint256) private _balances;
    mapping(uint256 tokenId => address) private _tokenApprovals;
    mapping(address owner => mapping(address operator => bool))
        private _operatorApprovals;

    string private _name;
    string private _symbol;

    mapping(uint256 => string) _token_uris;

    bool _paused;

    constructor(string memory name_, string memory symbol_) {
        _name = name_;
        _symbol = symbol_;
        _paused = false;
    }
}
