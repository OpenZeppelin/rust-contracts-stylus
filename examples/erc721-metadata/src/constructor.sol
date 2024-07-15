// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc721MetadataExample {
    mapping(uint256 tokenId => address) private _owners;
    mapping(address owner => uint256) private _balances;
    mapping(uint256 tokenId => address) private _tokenApprovals;
    mapping(address owner => mapping(address operator => bool))
        private _operatorApprovals;

    string private _name;
    string private _symbol;
    string private _baseUri;

    mapping(uint256 => string) _tokenUris;

    constructor(string memory name_, string memory symbol_, string memory baseUri_) {
        _name = name_;
        _symbol = symbol_;
        _baseUri = baseUri_;
    }
}
