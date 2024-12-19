#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc4262 {
        

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
