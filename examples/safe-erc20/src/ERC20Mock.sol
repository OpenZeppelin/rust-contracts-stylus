// SPDX-License-Identifier: MIT

pragma solidity ^0.8.21;

contract ERC20Mock {
    mapping(address => uint256) private _balances;

    function balanceOf(address account) public view returns (uint256) {
        return _balances[account];
    }

    function mint(address account, uint256 amount) public {
        _balances[account] += amount;
    }

    function transfer(address to, uint256 amount) public returns (bool) {
        _balances[msg.sender] -= amount;
        _balances[to] += amount;
        return true;
    }
}