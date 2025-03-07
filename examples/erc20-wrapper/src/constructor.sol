// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc20WrapperExample {
    // Erc20 Token Storage
    mapping(address account => uint256) private _balances;
    mapping(address account => mapping(address spender => uint256))
        private _allowances;
    uint256 private _totalSupply;

    // Erc20 Wrapper Storage
    address private _underlying;
    uint8 private _decimals;

    error ERC20InvalidUnderlying(address token);

    constructor(address underlyingToken_, uint8 decimals_) {
        if (underlyingToken_ == address(this)) {
            revert ERC20InvalidUnderlying(address(this));
        }
        _underlying = underlyingToken_;
        _decimals = decimals_;
    }
}
