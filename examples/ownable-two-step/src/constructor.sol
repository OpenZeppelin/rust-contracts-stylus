// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Ownable2StepExample {
    mapping(address => uint256) _balances;
    mapping(address => mapping(address => uint256)) _allowances;
    uint256 _totalSupply;

    address private _owner;
    address private _pendingOwner;

    error OwnableInvalidOwner(address owner);

    event OwnershipTransferred(
        address indexed previousOwner,
        address indexed newOwner
    );

    constructor(address initialOwner) {
        if (initialOwner == address(0)) {
            revert OwnableInvalidOwner(address(0));
        }

        address oldOwner = _owner;
        _owner = initialOwner;
        emit OwnershipTransferred(oldOwner, initialOwner);
    }
}
