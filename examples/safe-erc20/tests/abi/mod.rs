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
        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);

        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 value);
        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SafeErc20;

impl SafeErc20 {
    pub fn new(address: Address, wallet: &Wallet) -> Self {
        Self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Erc20;

impl SafeErc20 {
    pub fn safeTransfer(&self, token: Address, to: Address, value: U256) -> Result<()> {
        Ok(())
    }

    pub fn safeTransferFrom(&self, token: Address, from: Address, to: Address, value: U256) -> Result<()> {
        Ok(())
    }

    pub fn trySafeTransfer(&self, token: Address, to: Address, value: U256) -> Result<bool> {
        Ok(true)
    }

    pub fn trySafeTransferFrom(&self, token: Address, from: Address, to: Address, value: U256) -> Result<bool> {
        Ok(true)
    }

    pub fn safeIncreaseAllowance(&self, token: Address, spender: Address, value: U256) -> Result<()> {
        Ok(())
    }

    pub fn safeDecreaseAllowance(&self, token: Address, spender: Address, requestedDecrease: U256) -> Result<()> {
        Ok(())
    }

    pub fn forceApprove(&self, token: Address, spender: Address, value: U256) -> Result<()> {
        Ok(())
    }

    pub fn transferAndCallRelaxed(&self, token: Address, to: Address, value: U256, data: Vec<u8>) -> Result<()> {
        Ok(())
    }

    pub fn transferFromAndCallRelaxed(&self, token: Address, from: Address, to: Address, value: U256, data: Vec<u8>) -> Result<()> {
        Ok(())
    }

    pub fn approveAndCallRelaxed(&self, token: Address, to: Address, value: U256, data: Vec<u8>) -> Result<()> {
        Ok(())
    }
}
