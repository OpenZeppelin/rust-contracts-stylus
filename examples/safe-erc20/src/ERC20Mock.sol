// SPDX-License-Identifier: MIT

pragma solidity ^0.8.21;

import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.0.2/contracts/token/ERC20/ERC20.sol";

contract ERC20Mock is ERC20 {
    constructor() ERC20("MyToken", "MTK") {}

    function balanceOf(address account) public override view returns (uint256) {
        return super.balanceOf(account);
    }

    function mint(address account, uint256 amount) public {
        super._mint(account, amount);
    }

    function transfer(address to, uint256 amount) public override returns (bool) {
        return super.transfer(to, amount);
    }
}