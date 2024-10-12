// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract VestingWalletExample {
    address private _owner;
    
    uint256 private _released;
    mapping(address token => uint256) private _erc20Released;
    uint64 private _start;
    uint64 private _duration;

    constructor(address beneficiary, uint64 startTimestamp, uint64 durationSeconds) {
        _owner = beneficiary;
        _start = startTimestamp;
        _duration = durationSeconds;
    }
}
