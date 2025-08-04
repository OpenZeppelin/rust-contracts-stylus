#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract SafeErc20 {
        function safeTransfer(address token, address to, uint256 value) external;
        function safeTransferFrom(address token, address from, address to, uint256 value) external;
        function trySafeTransfer(address token, address to, uint256 value) external returns (bool);
        function trySafeTransferFrom(address token, address from, address to, uint256 value) external returns (bool);
        function safeIncreaseAllowance(address token, address spender, uint256 value) external;
        function safeDecreaseAllowance(address token, address spender, uint256 requestedDecrease) external;
        function forceApprove(address token, address spender, uint256 value) external;
        function transferAndCallRelaxed(address token, address to, uint256 value, bytes calldata data) external;
        function transferFromAndCallRelaxed(address token, address from, address to, uint256 value, bytes calldata data) external;
        function approveAndCallRelaxed(address token, address spender, uint256 value, bytes calldata data) external;

        error SafeErc20FailedOperation(address token);
        error SafeErc20FailedDecreaseAllowance(address spender, uint256 currentAllowance, uint256 requestedDecrease);

        // test-related events
        #[derive(Debug, PartialEq)]
        event True();
        #[derive(Debug, PartialEq)]
        event False();
    }

    contract Erc20 {
        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);

        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 value);
        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }

    contract Erc1363 {
        error ERC1363TransferFailed(address receiver, uint256 value);
        error ERC1363TransferFromFailed(address sender, address receiver, uint256 value);
        error ERC1363ApproveFailed(address spender, uint256 value);
        error ERC1363InvalidReceiver(address receiver);
        error ERC1363InvalidSpender(address spender);
    }

    contract Erc1363Receiver {
        #[derive(Debug, PartialEq)]
        event Received(address operator, address from, uint256 value, bytes data);

        error CustomError(bytes4);
    }

    contract Erc1363Spender {
        #[derive(Debug, PartialEq)]
        event Approved(address owner, uint256 value, bytes data);

        error CustomError(bytes4);
    }
);
