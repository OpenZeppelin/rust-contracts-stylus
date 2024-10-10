// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract VestingWalletExample {
    uint256 private _released;
    mapping(address token => uint256) private _erc20Released;
    uint64 private immutable _start;
    uint64 private immutable _duration;
    
    address private _owner;

    error OwnableInvalidOwner(address owner);
    event OwnershipTransferred(
        address indexed previousOwner,
        address indexed newOwner
    );

    constructor(address beneficiary, uint64 startTimestamp, uint64 durationSeconds) payable {
        _start = startTimestamp;
        _duration = durationSeconds;
        
        if (beneficiary == address(0)) {
            revert OwnableInvalidOwner(address(0));
        }
        _transferOwnership(beneficiary);
    }
    
    function _transferOwnership(address newOwner) internal virtual {
        address oldOwner = _owner;
        _owner = newOwner;
        emit OwnershipTransferred(oldOwner, newOwner);
    }
}
