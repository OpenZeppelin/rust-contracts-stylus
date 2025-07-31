#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract UUPSProxyErc20Example {
        function mint(address account, uint256 value) external;

        // ERC20 errors that will be bubbled up to the caller.
        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
        error ERC20InvalidSender(address sender);
        error ERC20InvalidReceiver(address receiver);
        error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
        error ERC20InvalidSpender(address spender);

        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 value);
        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }

    #[sol(rpc)]
    contract Erc1967Example {
        function implementation() public view returns (address implementation);

        // ERC20 functions that we want to delegate to the implementation.
        function totalSupply() external view returns (uint256 totalSupply);
        function balanceOf(address account) external view returns (uint256 balance);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256 allowance);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        function mint(address account, uint256 value) external;

        event Upgraded(address indexed implementation);
    }
);
