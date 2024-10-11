// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract VestingWalletExample {
    uint256 private _released;
    mapping(address token => uint256) private _erc20Released;
    uint64 private immutable _start;
    uint64 private immutable _duration;
    
    address private _owner;

    constructor(address beneficiary, uint64 startTimestamp, uint64 durationSeconds) payable {
        _owner = beneficiary;
        _start = startTimestamp;
        _duration = durationSeconds;
    }
}
