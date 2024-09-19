#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract SafeErc20Example {
        function safeTransfer(address token, address to, uint256 value) external;

        error SafeErc20FailedOperation(address token);
        error SafeErc20FailedDecreaseAllowance(address spender, uint256 currentAllowance, uint256 requestedDecrease);
    }
);
