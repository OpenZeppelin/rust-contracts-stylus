#![cfg(feature = "e2e")]

use abi::Erc4626;
use alloy::{
    primitives::{uint, Address, U256},
    sol,
};
use e2e::{
    receipt, send, watch, Account, EventExt, Panic, PanicCode, ReceiptExt,
    Revert,
};
use eyre::Result;
use mock::{erc20, erc20::ERC20Mock};
use openzeppelin_stylus::utils::math::alloy::{Math, Rounding};

use crate::Erc4626Example::constructorCall;

const ERC4626_NAME: &str = "Erc4626 Token";
const ERC4626_SYMBOL: &str = "ETT";

mod abi;
mod mock;

sol!("src/constructor.sol");

macro_rules! total_supply {
    ($contract:expr) => {
        $contract.totalSupply().call().await?.totalSupply
    };
}

macro_rules! decimals_offset {
    () => {
        U256::ZERO
    };
}

macro_rules! calculate_shares {
    ($contract:expr, $assets:expr, $tokens:expr) => {{
        let total_supply = total_supply!($contract);
        $assets.mul_div(
            total_supply
                + U256::from(10)
                    .checked_pow(decimals_offset!())
                    .expect("should not overflow"),
            $tokens + U256::from(1),
            Rounding::Floor,
        )
    }};
}

macro_rules! calculate_assets {
    ($contract:expr, $shares:expr, $tokens:expr) => {{
        let total_supply = total_supply!($contract);
        $shares.mul_div(
            $tokens.checked_add(uint!(1_U256)).expect("should not overflow"),
            total_supply
                + U256::from(10)
                    .checked_pow(decimals_offset!())
                    .expect("should not overflow"),
            Rounding::Floor,
        )
    }};
}

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

#[e2e::test]
async fn total_assets_success(alice: Account) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(asset_address, &alice.wallet);

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let initial_total_assets = contract.totalAssets().call().await?.totalAssets;

    let assets = uint!(69_U256);
    let _ = watch!(erc20_alice.mint(contract_addr, assets))?;

    let total_assets = contract.totalAssets().call().await?.totalAssets;
    assert_eq!(total_assets, initial_total_assets + assets);

    Ok(())
}

#[e2e::test]
async fn total_assets_reverts_when_asset_is_not_erc20(
    alice: Account,
) -> Result<()> {
    let invalid_asset = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(invalid_asset))
        .deploy()
        .await?
        .address()?;

    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let err = contract
        .totalAssets()
        .call()
        .await
        .expect_err("should return `InvalidAsset`");

    assert!(err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset }));

    Ok(())
}

#[e2e::test]
async fn convert_to_shares_reverts_when_asset_is_not_erc20(
    alice: Account,
) -> Result<()> {
    let invalid_asset = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(invalid_asset))
        .deploy()
        .await?
        .address()?;

    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let err = contract
        .convertToShares(uint!(10_U256))
        .call()
        .await
        .expect_err("should return `InvalidAsset`");

    assert!(err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset }));

    Ok(())
}

#[e2e::test]
async fn convert_to_shares_reverts_when_result_overflows(
    alice: Account,
) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(asset_address, &alice.wallet);

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let _ = watch!(erc20_alice.mint(contract_addr, U256::MAX))?;

    let err = contract
        .convertToShares(U256::MAX)
        .call()
        .await
        .expect_err("should panics due to `Overflow`");

    assert!(err.panicked_with(PanicCode::ArithmeticOverflow));
    Ok(())
}

#[e2e::test]
async fn convert_to_shares_works(alice: Account) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(asset_address, &alice.wallet);

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let tokens = uint!(100_U256);
    let _ = watch!(erc20_alice.mint(contract_addr, tokens))?;
    let assets = uint!(69_U256);

    let expected_shares = calculate_shares!(contract, assets, tokens);
    let shares = contract.convertToShares(assets).call().await?.shares;

    assert_eq!(shares, expected_shares);

    Ok(())
}

#[e2e::test]
async fn convert_to_assets_reverts_when_asset_is_not_erc20(
    alice: Account,
) -> Result<()> {
    let invalid_asset = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(invalid_asset))
        .deploy()
        .await?
        .address()?;

    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let err = contract
        .convertToAssets(uint!(10_U256))
        .call()
        .await
        .expect_err("should return `InvalidAsset`");

    assert!(err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset }));

    Ok(())
}

#[e2e::test]
async fn convert_to_assets_works(alice: Account) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(asset_address, &alice.wallet);

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let tokens = uint!(100_U256);
    let _ = watch!(erc20_alice.mint(contract_addr, tokens))?;

    let shares = uint!(69_U256);
    let assets = contract.convertToAssets(shares).call().await?.assets;
    let expected_assets = calculate_assets!(contract, shares, tokens);

    assert_eq!(assets, expected_assets);
    Ok(())
}

#[e2e::test]
async fn max_deposit_success(alice: Account) -> eyre::Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;

    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let max_deposit =
        contract.maxDeposit(alice.address()).call().await?.maxDeposit;
    assert_eq!(max_deposit, U256::MAX);

    Ok(())
}

#[e2e::test]
async fn preview_deposit_reverts_when_asset_is_not_erc20(
    alice: Account,
) -> Result<()> {
    let invalid_asset = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(invalid_asset))
        .deploy()
        .await?
        .address()?;

    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let err = contract
        .previewDeposit(uint!(10_U256))
        .call()
        .await
        .expect_err("should return `InvalidAsset`");

    assert!(err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset }));

    Ok(())
}

#[e2e::test]
async fn preview_deposit_reverts_when_result_overflows(
    alice: Account,
) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(asset_address, &alice.wallet);

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let _ = watch!(erc20_alice.mint(contract_addr, U256::MAX))?;

    let err = contract
        .previewDeposit(U256::MAX)
        .call()
        .await
        .expect_err("should panics due to `Overflow`");

    assert!(err.panicked_with(PanicCode::ArithmeticOverflow));
    Ok(())
}

#[e2e::test]
async fn preview_deposit_works(alice: Account) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(asset_address, &alice.wallet);

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let tokens = uint!(100_U256);
    let _ = watch!(erc20_alice.mint(contract_addr, tokens))?;

    let assets = uint!(69_U256);

    let expected_deposit = calculate_shares!(contract, assets, tokens);
    let preview_deposit = contract.previewDeposit(assets).call().await?.deposit;

    assert_eq!(preview_deposit, expected_deposit);
    Ok(())
}

#[e2e::test]
async fn deposit_reverts_when_asset_is_not_erc20(
    alice: Account,
) -> eyre::Result<()> {
    let invalid_asset = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(invalid_asset))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let err = send!(contract.deposit(uint!(10_U256), alice.address()))
        .expect_err("should return `InvalidAsset`");

    assert!(err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset }));
    Ok(())
}

#[e2e::test]
async fn deposit_reverts_when_result_overflows(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(asset_address, &alice.wallet);

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let _ = watch!(erc20_alice.mint(contract_addr, U256::MAX))?;

    let err = send!(contract.deposit(U256::MAX, bob.address()))
        .expect_err("should panics due to `Overflow`");

    assert!(err.panicked_with(PanicCode::ArithmeticOverflow));
    Ok(())
}

#[e2e::test]
async fn deposit_works(alice: Account, bob: Account) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let alice_address = alice.address();
    let erc20_alice = ERC20Mock::new(asset_address, &alice.wallet);

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let tokens = uint!(100_U256);
    let _ = watch!(erc20_alice.mint(contract_addr, tokens))?;
    let _ = watch!(erc20_alice.mint(alice_address, tokens))?;

    let assets = uint!(69_U256);
    let expected_deposit = calculate_shares!(contract, assets, tokens);

    let initial_alice_token_balance =
        erc20_alice.balanceOf(alice_address).call().await?._0;

    let initial_vault_token_balance =
        erc20_alice.balanceOf(contract_addr).call().await?._0;

    let initial_bob_shares_balance =
        contract.balanceOf(bob.address()).call().await?.balance;

    let initial_supply = contract.totalSupply().call().await?.totalSupply;

    let _ = watch!(erc20_alice.approve(contract_addr, assets))?;

    let receipt = receipt!(contract.deposit(assets, bob.address()))?;

    assert!(receipt.emits(Erc4626::Deposit {
        sender: alice_address,
        owner: bob.address(),
        assets,
        shares: expected_deposit
    }));

    let bob_shares_balance =
        contract.balanceOf(bob.address()).call().await?.balance;
    assert_eq!(
        initial_bob_shares_balance + expected_deposit,
        bob_shares_balance
    );

    let supply = contract.totalSupply().call().await?.totalSupply;
    assert_eq!(initial_supply + expected_deposit, supply);

    let alice_token_balance =
        erc20_alice.balanceOf(alice_address).call().await?._0;
    assert_eq!(initial_alice_token_balance - assets, alice_token_balance);

    let vault_token_balance =
        erc20_alice.balanceOf(contract_addr).call().await?._0;
    assert_eq!(initial_vault_token_balance + assets, vault_token_balance);

    Ok(())
}

#[e2e::test]
async fn max_mint_success(alice: Account) -> eyre::Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;

    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let max_mint = contract.maxMint(alice.address()).call().await?.maxMint;
    assert_eq!(max_mint, U256::MAX);

    Ok(())
}

#[e2e::test]
async fn preview_mint_reverts_when_asset_is_not_erc20(
    alice: Account,
) -> Result<()> {
    let invalid_asset = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(invalid_asset))
        .deploy()
        .await?
        .address()?;

    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let err = contract
        .previewMint(uint!(10_U256))
        .call()
        .await
        .expect_err("should return `InvalidAsset`");

    assert!(err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset }));

    Ok(())
}

#[e2e::test]
async fn mint_reverts_when_asset_is_not_erc20(
    alice: Account,
) -> eyre::Result<()> {
    let invalid_asset = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(invalid_asset))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let err = send!(contract.mint(uint!(10_U256), alice.address()))
        .expect_err("should return `InvalidAsset`");

    assert!(err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset }));
    Ok(())
}

#[e2e::test]
async fn max_withdraw_reverts_when_asset_is_not_erc20(
    alice: Account,
) -> Result<()> {
    let invalid_asset = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(invalid_asset))
        .deploy()
        .await?
        .address()?;

    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let err = contract
        .maxWithdraw(alice.address())
        .call()
        .await
        .expect_err("should return `InvalidAsset`");

    assert!(err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset }));

    Ok(())
}

#[e2e::test]
async fn preview_withdraw_reverts_when_asset_is_not_erc20(
    alice: Account,
) -> Result<()> {
    let invalid_asset = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(invalid_asset))
        .deploy()
        .await?
        .address()?;

    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let err = contract
        .previewWithdraw(uint!(10_U256))
        .call()
        .await
        .expect_err("should return `InvalidAsset`");

    assert!(err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset }));

    Ok(())
}

#[e2e::test]
async fn withdraw_reverts_when_exceeded_max_redeem(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let invalid_asset = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(invalid_asset))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let err = send!(contract.withdraw(
        uint!(10_U256),
        alice.address(),
        bob.address()
    ))
    .expect_err("should return `InvalidAsset`");

    assert!(err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset }));
    Ok(())
}

#[e2e::test]
async fn max_redeem_zero_balance_success(alice: Account) -> eyre::Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;

    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let max_redeem =
        contract.maxRedeem(alice.address()).call().await?.maxRedeem;
    assert_eq!(max_redeem, U256::ZERO);

    Ok(())
}

#[e2e::test]
async fn preview_redeem_reverts_when_asset_is_not_erc20(
    alice: Account,
) -> Result<()> {
    let invalid_asset = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(invalid_asset))
        .deploy()
        .await?
        .address()?;

    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let err = contract
        .previewRedeem(uint!(10_U256))
        .call()
        .await
        .expect_err("should return `InvalidAsset`");

    assert!(err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset }));

    Ok(())
}

#[e2e::test]
async fn redeem_reverts_when_exceeded_max_redeem_zero_balance(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let invalid_asset = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(invalid_asset))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);

    let shares = uint!(10_U256);
    let err = send!(contract.redeem(shares, bob.address(), alice.address()))
        .expect_err("should return `ERC4626ExceededMaxRedeem`");

    assert!(err.reverted_with(Erc4626::ERC4626ExceededMaxRedeem {
        owner: alice.address(),
        shares,
        max: U256::ZERO,
    }));
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
