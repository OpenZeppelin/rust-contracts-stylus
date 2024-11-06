#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract SafeErc20 {
        function safeTransfer(address token, address to, uint256 value) external;
        function safeTransferFrom(address token, address from, address to, uint256 value) external;
        function safeIncreaseAllowance(address token, address spender, uint256 value) external;
        function safeDecreaseAllowance(address token, address spender, uint256 requestedDecrease) external;
        function forceApprove(address token, address spender, uint256 value) external;

        error SafeErc20FailedOperation(address token);
        error SafeErc20FailedDecreaseAllowance(address spender, uint256 currentAllowance, uint256 requestedDecrease);
    }

    contract Erc20 {
        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 value);
        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }
);
