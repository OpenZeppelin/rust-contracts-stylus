#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc20Wrapper {
        function totalSupply() external view returns (uint256 totalSupply);
        function balanceOf(address account) external view returns (uint256 balance);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256 allowance);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        #[derive(Debug)]
        function decimals() external view returns (uint8 decimals);
        #[derive(Debug)]
        function underlying() external view returns (address underlying);
        #[derive(Debug)]
        function depositFor(address account, uint256 value) external  returns (bool);
        #[derive(Debug)]
        function withdrawTo(address account, uint256 value) external  returns (bool);

        error ERC20InvalidUnderlying(address token);
        error ERC20InvalidSender(address sender);
        error ERC20InvalidReceiver(address receiver);

    }

    contract Erc20 {
        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 value);
        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed spender, uint256 value);

        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
    }

    #[sol(rpc)]
    contract SafeErc20 {
        error SafeErc20FailedOperation(address token);
    }
);
