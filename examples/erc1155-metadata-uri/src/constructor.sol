// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract Erc1155MetadataUriExample {
    mapping(address => mapping(uint256 => uint256)) private _balances;
    mapping(address => mapping(address => bool)) private _operatorApprovals;
    
    string private _uri;
    string private _baseUri;

    mapping(uint256 => string) _tokenUris;
    
    constructor(string memory uri_, string memory baseUri_) {
        _uri = uri_;
        _baseUri = baseUri_;
    }
}
