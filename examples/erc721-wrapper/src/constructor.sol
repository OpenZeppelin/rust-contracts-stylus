// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc721WrapperExample {
    mapping(uint256 tokenId => address) private _owners;
    mapping(address owner => uint256) private _balances;
    mapping(uint256 tokenId => address) private _tokenApprovals;
    mapping(address owner => mapping(address operator => bool))
        private _operatorApprovals;

    address private _underlyingToken;

    constructor(address underlyingToken_) {
        _underlyingToken = underlyingToken_;
    }
}
