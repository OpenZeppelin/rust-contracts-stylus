// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract OwnableExample {
    mapping(address => uint256) _balances;
    mapping(address => mapping(address => uint256)) _allowances;
    uint256 _total_supply;

    address private _owner;

    error OwnableInvalidOwner(address owner);
    event OwnershipTransferred(
        address indexed previousOwner,
        address indexed newOwner
    );

    constructor(address initialOwner) {
        if (initialOwner == address(0)) {
            revert OwnableInvalidOwner(address(0));
        }
        _transferOwnership(initialOwner);
    }

    function _transferOwnership(address newOwner) internal virtual {
        address oldOwner = _owner;
        _owner = newOwner;
        emit OwnershipTransferred(oldOwner, newOwner);
    }
}
