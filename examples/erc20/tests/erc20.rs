#![cfg(feature = "e2e")]

use alloy::{
    primitives::{Address, U256},
    sol,
    sol_types::SolConstructor,
};
use e2e::user::User;
use eyre::Result;

sol!("src/constructor.sol");

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

async fn deploy(rpc_url: &str, private_key: &str) -> eyre::Result<Address> {
    let name = env!("CARGO_PKG_NAME").replace('-', "_");
    let pkg_dir = env!("CARGO_MANIFEST_DIR");
    let args = Erc20Example::constructorCall {
        name_: "Test Token".to_owned(),
        symbol_: "TTK".to_owned(),
        cap_: U256::from(1),
    };
    let args = alloy::hex::encode(args.abi_encode());
    let contract_addr =
        e2e::deploy::deploy(&name, pkg_dir, rpc_url, private_key, Some(args))
            .await?;

    Ok(contract_addr)
}

#[e2e::test]
async fn mint(alice: User) -> Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc20::new(contract_addr, &alice.signer);

    let Erc20::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice.address()).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await?;

    let one = U256::from(1);
    let _ = contract.mint(alice.address(), one).send().await?;

    let Erc20::balanceOfReturn { balance } =
        contract.balanceOf(alice.address()).call().await?;
    let Erc20::totalSupplyReturn { totalSupply } =
        contract.totalSupply().call().await?;

    assert_eq!(initial_balance + one, balance);
    assert_eq!(initial_supply + one, totalSupply);
    Ok(())
}

// TODO: add rest of the tests for erc20 base implementation
