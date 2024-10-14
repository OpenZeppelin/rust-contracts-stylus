// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract VestingWalletExample {
    address private _owner;
    
    uint256 private _released;
    mapping(address token => uint256) private _erc20Released;
    uint64 private _start;
    uint64 private _duration;

    error OwnableInvalidOwner(address owner);
    event OwnershipTransferred(
        address indexed previousOwner,
        address indexed newOwner
    );

    constructor(address beneficiary, uint64 startTimestamp, uint64 durationSeconds) payable {
         if (beneficiary == address(0)) {
            revert OwnableInvalidOwner(address(0));
        }
        _transferOwnership(beneficiary);
        
        _start = startTimestamp;
        _duration = durationSeconds;
    }

    function _transferOwnership(address newOwner) internal virtual {
        address oldOwner = _owner;
        _owner = newOwner;
        emit OwnershipTransferred(oldOwner, newOwner);
    }
}
