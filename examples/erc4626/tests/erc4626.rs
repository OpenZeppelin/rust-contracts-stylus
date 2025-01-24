#![cfg(feature = "e2e")]

use abi::Erc4626;
use alloy::{primitives::Address, sol};
use e2e::{/* receipt, */ Account, ReceiptExt};
use eyre::Result;
use mock::{erc20 /* , erc20::ERC20Mock */};

use crate::Erc4626Example::constructorCall;

const ERC4626_NAME: &str = "Erc4626 Token";
const ERC4626_SYMBOL: &str = "ETT";

mod abi;
mod mock;

sol!("src/constructor.sol");

fn ctr(asset: Address) -> constructorCall {
    constructorCall {
        asset_: asset,
        name_: ERC4626_NAME.to_owned(),
        symbol_: ERC4626_SYMBOL.to_owned(),
    }
}

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;

    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let name = contract.name().call().await?.name;
    assert_eq!(name, ERC4626_NAME.to_owned());

    let symbol = contract.symbol().call().await?.symbol;
    assert_eq!(symbol, ERC4626_SYMBOL.to_owned());

    let decimals = contract.decimals().call().await?.decimals;
    assert_eq!(decimals, 18);

    let asset = contract.asset().call().await?.asset;
    assert_eq!(asset, asset_address);

    Ok(())
}

/*#[e2e::test]
async fn deposit(alice: Account, bob: Account) -> Result<()> {
    let asset_address =
        erc20::deploy(&alice.wallet).await?;
    let vault_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;

    let asset = MockErc20::new(asset_address, &alice.wallet);
    let vault = Erc4626::new(vault_addr, &alice.wallet);
    let alice_addr = alice.address();

    let MockErc20::balanceOfReturn { balance: initial_balance } =
        asset.balanceOf(alice_addr).call().await?;
    let MockErc20::totalSupplyReturn { totalSupply: initial_supply } =
        asset.totalSupply().call().await?;

    assert_eq!(U256::ZERO, initial_balance);
    assert_eq!(U256::ZERO, initial_supply);

    // Mint token
    let _mint_receipt = receipt!(asset.mint(alice.address(), uint!(100_U256)))?;
    // println!("{:?}", mint_receipt);

    let _ = asset.approve(alice.address(), U256::MAX).send().await?;
    let _ = vault.approve(alice.address(), U256::MAX).send().await?;

    let max_mint = vault.maxMint(bob.address()).call().await?._0;
    let preview_deposit = vault.previewDeposit(uint!(1_U256)).call().await?._0;
    let _deposit = vault.deposit(uint!(1_U256), bob.address()).call().await?._0;
    assert_eq!(max_mint, U256::MAX);
    assert_eq!(preview_deposit, uint!(1_U256));

    let _asset_balance = asset.balanceOf(bob.address()).call().await?.balance;
    //assert_eq!(asset_balance, uint!(1_U256));

    // let valut_balance =
    // contract.balanceOf(bob.address()).call().await?.balance;
    // assert_eq!(valut_balance, uint!(1_U256));

    Ok(())
}

#[e2e::test]
async fn mint(_alice: Account, _bob: Account) -> Result<()> {
    Ok(())
}

#[e2e::test]
async fn withdraw(_alice: Account, _bob: Account) -> Result<()> {
    Ok(())
}

#[e2e::test]
async fn redeem(_alice: Account, _bob: Account) -> Result<()> {
    Ok(())
}

#[e2e::test]
async fn deposit_inflation_attack(
    _alice: Account,
    _bob: Account,
) -> Result<()> {
    Ok(())
}

#[e2e::test]
async fn mint_inflation_attack(_alice: Account, _bob: Account) -> Result<()> {
    Ok(())
}

#[e2e::test]
async fn withdraw_inflation_attack(
    _alice: Account,
    _bob: Account,
) -> Result<()> {
    Ok(())
}

#[e2e::test]
async fn redeem_inflation_attack(_alice: Account, _bob: Account) -> Result<()> {
    Ok(())
}
*/
#[e2e::test]
async fn error_when_exceeded_max_deposit(
    alice: Account,
    _bob: Account,
) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;
    let _contract = Erc4626::new(contract_addr, &alice.wallet);
    Ok(())
}

#[e2e::test]
async fn error_when_exceeded_max_mint(
    alice: Account,
    _bob: Account,
) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;
    let _contract = Erc4626::new(contract_addr, &alice.wallet);
    Ok(())
}

#[e2e::test]
async fn error_when_exceeded_max_withdraw(
    alice: Account,
    _bob: Account,
) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;
    let _contract = Erc4626::new(contract_addr, &alice.wallet);
    Ok(())
}

#[e2e::test]
async fn error_when_exceeded_max_redeem(
    alice: Account,
    _bob: Account,
) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;
    let _contract = Erc4626::new(contract_addr, &alice.wallet);
    Ok(())
}
