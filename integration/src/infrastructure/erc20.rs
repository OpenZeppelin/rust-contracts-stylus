use std::sync::Arc;

use ethers::prelude::*;

use crate::{infrastructure::*, token_impl};

abigen!(
    Erc20Token,
    r#"[
        function init(uint256) external
        function name() external view returns (string)
        function symbol() external view returns (string)
        function decimals() external view returns (uint8)
        function totalSupply() external view returns (uint256)
        function balanceOf(address account) external view returns (uint256)
        function transfer(address recipient, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
        function approve(address spender, uint256 amount) external returns (bool)
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool)
        
        function mint(address account, uint256 amount) external
        function burn(uint256 amount) external
        
        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed)
        error ERC20InvalidSender(address sender)
        error ERC20InvalidReceiver(address receiver)
        error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed)
        error ERC20InvalidSpender(address spender)
    ]"#
);

pub type Erc20 = Erc20Token<HttpMiddleware>;
token_impl!(Erc20, "ERC20_EXAMPLE_DEPLOYMENT_ADDRESS");
