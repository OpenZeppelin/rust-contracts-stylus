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
    ($contract:expr, $assets:expr, $tokens:expr, $rounding:expr) => {{
        let total_supply = total_supply!($contract);
        $assets.mul_div(
            total_supply
                + U256::from(10)
                    .checked_pow(decimals_offset!())
                    .expect("should not overflow"),
            $tokens + U256::from(1),
            $rounding,
        )
    }};
}

macro_rules! calculate_assets {
    ($contract:expr, $shares:expr, $tokens:expr, $rounding:expr) => {{
        let total_supply = total_supply!($contract);
        $shares.mul_div(
            $tokens.checked_add(uint!(1_U256)).expect("should not overflow"),
            total_supply
                + U256::from(10)
                    .checked_pow(decimals_offset!())
                    .expect("should not overflow"),
            $rounding,
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

async fn deploy(
    account: &Account,
    initial_tokens: U256,
) -> Result<(Address, Address)> {
    let asset_addr = erc20::deploy(&account.wallet).await?;

    let contract_addr = account
        .as_deployer()
        .with_constructor(ctr(asset_addr))
        .deploy()
        .await?
        .address()?;

    // Mint initial tokens to the vault
    if initial_tokens > U256::ZERO {
        let asset = ERC20Mock::new(asset_addr, &account.wallet);
        _ = watch!(asset.mint(contract_addr, initial_tokens))?;
    }

    Ok((contract_addr, asset_addr))
}

mod constructor {
    use super::*;
    #[e2e::test]
    async fn success(alice: Account) -> Result<()> {
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
}

mod total_assets {
    use super::*;
    #[e2e::test]
    async fn reports_zero_total_assets_when_empty(
        alice: Account,
    ) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let total = contract.totalAssets().call().await?.totalAssets;
        assert_eq!(U256::ZERO, total);

        Ok(())
    }

    #[e2e::test]
    async fn reports_correct_total_assets_after_deposit(
        alice: Account,
    ) -> Result<()> {
        let initial_deposit = uint!(1000_U256);
        let (contract_addr, _) = deploy(&alice, initial_deposit).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let total = contract.totalAssets().call().await?.totalAssets;
        assert_eq!(initial_deposit, total);

        Ok(())
    }

    #[e2e::test]
    async fn updates_after_external_transfer(alice: Account) -> Result<()> {
        let initial_deposit = uint!(1000_U256);
        let additional_amount = uint!(500_U256);
        let (contract_addr, asset_addr) =
            deploy(&alice, initial_deposit).await?;

        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        // Transfer additional tokens directly to the vault
        _ = watch!(asset.mint(contract_addr, additional_amount))?;

        let total = contract.totalAssets().call().await?.totalAssets;
        assert_eq!(initial_deposit + additional_amount, total);

        Ok(())
    }

    #[e2e::test]
    async fn handles_max_uint256_balance(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::MAX).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let total = contract.totalAssets().call().await?.totalAssets;
        assert_eq!(U256::MAX, total);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_for_zero_address_asset(alice: Account) -> Result<()> {
        // Deploy with zero address as asset
        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(Address::ZERO))
            .deploy()
            .await?
            .address()?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .totalAssets()
            .call()
            .await
            .expect_err("should return `InvalidAsset`");

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: Address::ZERO })
        );

        Ok(())
    }

    #[e2e::test]
    async fn reverts_for_invalid_asset(alice: Account) -> Result<()> {
        // Deploy with zero address as asset
        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(alice.address()))
            .deploy()
            .await?
            .address()?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .totalAssets()
            .call()
            .await
            .expect_err("should return `InvalidAsset`");

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: alice.address() })
        );

        Ok(())
    }

    #[e2e::test]
    async fn reflects_balance_after_withdrawal(alice: Account) -> Result<()> {
        let initial_deposit = uint!(1000_U256);
        let withdrawal = uint!(400_U256);
        let (contract_addr, asset_addr) =
            deploy(&alice, initial_deposit).await?;

        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        let alice_addr = alice.address();

        // Simulate withdrawal by transferring tokens out
        _ = watch!(asset.regular_approve(
            contract_addr,
            alice_addr,
            withdrawal
        ))?;
        _ = watch!(asset.transferFrom(contract_addr, alice_addr, withdrawal))?;

        let total = contract.totalAssets().call().await?.totalAssets;
        assert_eq!(initial_deposit - withdrawal, total);

        Ok(())
    }
}

mod convert_to_shares {
    use super::*;

    #[e2e::test]
    async fn converts_zero_assets_to_zero_shares(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, uint!(1000_U256)).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let shares = contract.convertToShares(U256::ZERO).call().await?.shares;
        assert_eq!(U256::ZERO, shares);

        Ok(())
    }

    #[e2e::test]
    async fn returns_zero_shares_for_asset_amount_less_then_vault_assets(
        alice: Account,
    ) -> Result<()> {
        let initial_assets = uint!(1000_U256);
        let assets_to_convert = uint!(100_U256);
        let (contract_addr, _) = deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let shares =
            contract.convertToShares(assets_to_convert).call().await?.shares;

        assert_eq!(U256::ZERO, shares);

        Ok(())
    }

    #[e2e::test]
    async fn returns_shares_equal_to_deposit_when_vault_is_empty(
        alice: Account,
    ) -> Result<()> {
        let assets_to_convert = uint!(101_U256);
        let (contract_addr, _) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let shares =
            contract.convertToShares(assets_to_convert).call().await?.shares;

        assert_eq!(assets_to_convert, shares);

        Ok(())
    }

    #[e2e::test]
    async fn returns_shares_proportional_to_deposit_when_vault_has_assets(
        alice: Account,
    ) -> Result<()> {
        let initial_assets = uint!(100_U256);
        let assets_to_convert = uint!(101_U256);
        let (contract_addr, _) = deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let expected_shares = uint!(1_U256);
        let shares =
            contract.convertToShares(assets_to_convert).call().await?.shares;

        assert_eq!(expected_shares, shares);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_invalid_asset(alice: Account) -> Result<()> {
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

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset })
        );

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_result_overflows(alice: Account) -> Result<()> {
        let (contract_addr, _asset_addr) = deploy(&alice, U256::MAX).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .convertToShares(U256::MAX)
            .call()
            .await
            .expect_err("should panics due to `Overflow`");

        assert!(err.panicked_with(PanicCode::ArithmeticOverflow));
        Ok(())
    }
}

mod convert_to_assets {
    use super::*;

    #[e2e::test]
    async fn converts_zero_shares_to_zero_assets(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, uint!(1000_U256)).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let assets = contract.convertToAssets(U256::ZERO).call().await?.assets;
        assert_eq!(U256::ZERO, assets);

        Ok(())
    }

    #[e2e::test]
    async fn returns_zero_assets_when_no_shares_were_ever_minted(
        alice: Account,
    ) -> Result<()> {
        let tokens = uint!(100_U256);

        let (contract_addr, _asset_addr) = deploy(&alice, tokens).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let shares = uint!(69_U256);
        let expected_assets = uint!(6969_U256);

        let assets = contract.convertToAssets(shares).call().await?.assets;

        assert_eq!(assets, expected_assets);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_invalid_asset(alice: Account) -> Result<()> {
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

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset })
        );

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_result_overflows(alice: Account) -> Result<()> {
        let (contract_addr, _asset_addr) = deploy(&alice, U256::MAX).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .convertToShares(U256::MAX)
            .call()
            .await
            .expect_err("should panics due to `Overflow`");

        assert!(err.panicked_with(PanicCode::ArithmeticOverflow));
        Ok(())
    }
}

mod max_deposit {
    use super::*;
    #[e2e::test]
    async fn success(alice: Account) -> Result<()> {
        let (contract_addr, _asset_addr) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let max_deposit =
            contract.maxDeposit(alice.address()).call().await?.maxDeposit;
        assert_eq!(max_deposit, U256::MAX);

        Ok(())
    }
}

mod preview_deposit {
    use super::*;
    #[e2e::test]
    async fn reverts_when_invalid_asset(alice: Account) -> Result<()> {
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

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset })
        );

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_result_overflows(alice: Account) -> Result<()> {
        let (contract_addr, _asset_addr) = deploy(&alice, U256::MAX).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .previewDeposit(U256::MAX)
            .call()
            .await
            .expect_err("should panics due to `Overflow`");

        assert!(err.panicked_with(PanicCode::ArithmeticOverflow));
        Ok(())
    }

    #[e2e::test]
    async fn success(alice: Account) -> Result<()> {
        let tokens = uint!(100_U256);

        let (contract_addr, _asset_addr) = deploy(&alice, tokens).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let assets = uint!(69_U256);

        let expected_deposit =
            calculate_shares!(contract, assets, tokens, Rounding::Floor);
        let preview_deposit =
            contract.previewDeposit(assets).call().await?.deposit;

        assert_eq!(preview_deposit, expected_deposit);
        Ok(())
    }
}

mod deposit {
    use super::*;
    #[e2e::test]
    async fn reverts_when_invalid_asset(alice: Account) -> Result<()> {
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

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset })
        );
        Ok(())
    }

    // TODO: deposit ExceededMaxDeposit E2E test

    // TODO: deposit InvalidReceiver E2E test

    // TODO: deposit SafeErc20FailedOperation E2E test

    #[e2e::test]
    async fn reverts_when_result_overflows(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
        let (contract_addr, _asset_addr) = deploy(&alice, U256::MAX).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = send!(contract.deposit(U256::MAX, bob.address()))
            .expect_err("should panics due to `Overflow`");

        assert!(err.panicked_with(PanicCode::ArithmeticOverflow));
        Ok(())
    }

    #[e2e::test]
    async fn success(alice: Account, bob: Account) -> Result<()> {
        let alice_address = alice.address();
        let tokens = uint!(100_U256);

        let (contract_addr, asset_addr) = deploy(&alice, tokens).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let erc20_alice = ERC20Mock::new(asset_addr, &alice.wallet);

        let _ = watch!(erc20_alice.mint(alice_address, tokens))?;

        let assets = uint!(69_U256);
        let expected_deposit =
            calculate_shares!(contract, assets, tokens, Rounding::Floor);

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
}
mod max_mint {
    use super::*;
    #[e2e::test]
    async fn success(alice: Account) -> Result<()> {
        let (contract_addr, _asset_addr) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let max_mint = contract.maxMint(alice.address()).call().await?.maxMint;
        assert_eq!(max_mint, U256::MAX);

        Ok(())
    }
}

mod preview_mint {
    use super::*;
    #[e2e::test]
    async fn reverts_when_invalid_asset(alice: Account) -> Result<()> {
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

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset })
        );

        Ok(())
    }

    // TODO: preview_mint overflows E2E test

    #[e2e::test]
    async fn success(alice: Account) -> Result<()> {
        let tokens = uint!(100_U256);

        let (contract_addr, _asset_addr) = deploy(&alice, tokens).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let shares = uint!(69_U256);
        let expected_mint =
            calculate_assets!(contract, shares, tokens, Rounding::Ceil);

        let mint = contract.previewMint(shares).call().await?.mint;

        assert_eq!(mint, expected_mint);
        Ok(())
    }
}

mod mint {
    use super::*;
    #[e2e::test]
    async fn reverts_when_invalid_asset(alice: Account) -> Result<()> {
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

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset })
        );
        Ok(())
    }

    // TODO: mint ExceededMaxMint E2E test

    // TODO: mint InvalidReceiver E2E test

    // TODO: mint SafeErc20FailedOperation E2E test

    // TODO: mint ERC20InsufficientBalance E2E test

    // TODO: mint overflows E2E test

    // TODO: mint success E2E test
}

mod max_withdraw {
    use super::*;
    #[e2e::test]
    async fn reverts_when_invalid_asset(alice: Account) -> Result<()> {
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

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset })
        );

        Ok(())
    }

    // TODO: max_withdraw overflows E2E test

    // TODO: max_withdraw success E2E test
}

mod preview_withdraw {
    use super::*;
    #[e2e::test]
    async fn reverts_when_invalid_asset(alice: Account) -> Result<()> {
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

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset })
        );

        Ok(())
    }

    // TODO: preview_withdraw overflows E2E test

    // TODO: preview_withdraw success E2E test
}

mod withdraw {
    use super::*;
    #[e2e::test]
    async fn reverts_when_invalid_asset(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
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

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset })
        );
        Ok(())
    }

    // TODO: withdraw ExceededMaxWithdraw E2E test

    // TODO: withdraw InsufficientAllowance E2E test

    // TODO: withdraw InvalidSender E2E test

    // TODO: withdraw InsufficientBalance E2E test

    // TODO: withdraw SafeErc20FailedOperation E2E test

    // TODO: withdraw overflows E2E test

    // TODO: withdraw success E2E test
}

mod max_redeem {
    use super::*;
    #[e2e::test]
    async fn zero_balance_success(alice: Account) -> Result<()> {
        let (contract_addr, _asset_addr) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let max_redeem =
            contract.maxRedeem(alice.address()).call().await?.maxRedeem;
        assert_eq!(max_redeem, U256::ZERO);

        Ok(())
    }

    // TODO: max_redeem balance higher than U256::ZERO E2E test
}

mod preview_redeem {
    use super::*;
    #[e2e::test]
    async fn reverts_when_invalid_asset(alice: Account) -> Result<()> {
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

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset })
        );

        Ok(())
    }

    // TODO: preview_redeem overflows E2E test

    // TODO: preview_redeem success E2E test
}

mod redeem {
    use super::*;
    #[e2e::test]
    async fn reverts_when_exceeded_max_redeem_zero_balance(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
        let (contract_addr, _asset_addr) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let shares = uint!(10_U256);
        let err =
            send!(contract.redeem(shares, bob.address(), alice.address()))
                .expect_err("should return `ERC4626ExceededMaxRedeem`");

        assert!(err.reverted_with(Erc4626::ERC4626ExceededMaxRedeem {
            owner: alice.address(),
            shares,
            max: U256::ZERO,
        }));
        Ok(())
    }

    // TODO: redeem InvaidAsset E2E test

    // TODO: redeem ExceededMaxRedeem E2E test

    // TODO: redeem InsufficientAllowance E2E test

    // TODO: redeem InvalidSender E2E test

    // TODO: redeem InsufficientBalance E2E test

    // TODO: redeem SafeErc20FailedOperation E2E test

    // TODO: redeem overflows E2E test

    // TODO: redeem success E2E test
}
