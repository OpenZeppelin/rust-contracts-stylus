use alloy::primitives::{Address, U256};
use alloy::providers::WalletProvider;
use alloy::sol;
use eyre::Result;

use crate::context::{build_context, Context};

sol!(
    #[sol(rpc)]
    contract Erc20 {
        function name() external view returns (string);
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
        function totalSupply() external view returns (uint256 totalSupply);
        function balanceOf(address account) external view returns (uint256 balance);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        function mint(address account, uint256 amount) external;
        function burn(uint256 amount) external;

        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
        error ERC20InvalidSender(address sender);
        error ERC20InvalidReceiver(address receiver);
        error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
        error ERC20InvalidSpender(address spender);
    }
);

#[tokio::test]
async fn mint() -> Result<()> {
    let ctx = build_context();
    let alice = &ctx.signers()[0];
    let alice_addr = alice.default_signer_address();
    let contract = Erc20::new(Address::random(), &alice);

    let Erc20::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr).call().await.unwrap();
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await.unwrap();

    let one = U256::from(1);
    let _ = contract.mint(alice_addr, one).send().await.unwrap();

    let Erc20::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await.unwrap();
    let Erc20::totalSupplyReturn { totalSupply } =
        contract.totalSupply().call().await.unwrap();

    assert_eq!(initial_balance + one, balance);
    assert_eq!(initial_supply + one, totalSupply);
    Ok(())
}

// TODO: add rest of the tests for erc20 base implementation
