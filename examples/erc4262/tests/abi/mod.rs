#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc4262 {
          function name() external view returns (string name);
        function symbol() external view returns (string symbol);
        function decimals() external view returns (uint8 decimals);
        function totalSupply() external view returns (uint256 totalSupply);
        function balanceOf(address account) external view returns (uint256 balance);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256 allowance);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
        error ERC20InvalidSender(address sender);
        error ERC20InvalidReceiver(address receiver);
        error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
        error ERC20InvalidSpender(address spender);
 
        error ERC4626ExceededMaxMint(address receiver, uint256 shares, uint256 max);
        error ERC4626ExceededMaxDeposit(address receiver, uint256 assets, uint256 max);
        error ERC4626ExceededMaxWithdraw(address owner, uint256 assets, uint256 max);
        error ERC4626ExceededMaxRedeem(address owner, uint256 shares, uint256 max);


        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 value);
        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }
);
