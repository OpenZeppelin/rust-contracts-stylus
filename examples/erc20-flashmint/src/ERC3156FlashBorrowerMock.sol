// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.1.0/contracts/interfaces/IERC3156FlashBorrower.sol";

contract ERC3156FlashBorrowerMock is IERC3156FlashBorrower {
    bytes32 internal constant _RETURN_VALUE = keccak256("ERC3156FlashBorrower.onFlashLoan");

    bool immutable _enableApprove;
    bool immutable _enableReturn;

    event BalanceOf(address token, address account, uint256 value);
    event TotalSupply(address token, uint256 value);

    constructor(bool enableReturn, bool enableApprove) {
        _enableApprove = enableApprove;
        _enableReturn = enableReturn;
    }

    function onFlashLoan(
        address /* initiator */,
        address token,
        uint256 amount,
        uint256 fee,
        bytes calldata data
    ) public returns (bytes32) {
        require(msg.sender == token, "Invalid token address");

        emit BalanceOf(token, address(this), IERC20(token).balanceOf(address(this)));
        emit TotalSupply(token, IERC20(token).totalSupply());

        if (data.length > 0) {
            // WARNING: This code is for testing purposes only! Do not use in production.
            Address.functionCall(token, data);
        }

        if (_enableApprove) {
            IERC20(token).approve(token, amount + fee);
        }

        return _enableReturn ? _RETURN_VALUE : bytes32(0);
    }
}
