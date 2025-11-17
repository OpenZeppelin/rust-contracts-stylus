#![cfg(feature = "e2e")]

use abi::Erc4626;
use alloy::primitives::{uint, Address, U256};
use e2e::{
    constructor, receipt, send, watch, Account, Constructor, EventExt, Revert,
    RustPanic,
};
use eyre::Result;
use mock::{
    erc20, erc20::ERC20Mock, erc20_failing_transfer,
    erc20_failing_transfer::ERC20FailingTransferMock,
};

const DECIMALS_OFFSET: u8 = 0;
/// The minimum decimal offset needed to induce overflow
const MIN_OVERFLOW_DECIMAL_OFFSET: u8 = 78;
const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "TTK";

mod abi;
mod mock;

fn ctr(asset: Address) -> Constructor {
    constructor!(
        asset,
        DECIMALS_OFFSET,
        TOKEN_NAME.to_string(),
        TOKEN_SYMBOL.to_string(),
    )
}

fn dec_offset_overflow_ctr(asset: Address) -> Constructor {
    constructor!(
        asset,
        MIN_OVERFLOW_DECIMAL_OFFSET,
        TOKEN_NAME.to_string(),
        TOKEN_SYMBOL.to_string(),
    )
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
        .contract_address;

    // Mint initial tokens to the vault
    if initial_tokens > U256::ZERO {
        let asset = ERC20Mock::new(asset_addr, &account.wallet);
        watch!(asset.mint(contract_addr, initial_tokens))?;
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
            .contract_address;

        let contract = Erc4626::new(contract_addr, &alice.wallet);

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
        watch!(asset.mint(contract_addr, additional_amount))?;

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
            .contract_address;
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
            .contract_address;
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
        watch!(asset.regular_approve(contract_addr, alice_addr, withdrawal))?;
        watch!(asset.transferFrom(contract_addr, alice_addr, withdrawal))?;

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

        let expected_shares = U256::ONE;
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
            .contract_address;

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
        let (contract_addr, _) = deploy(&alice, U256::MAX).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .convertToShares(U256::MAX)
            .call()
            .await
            .expect_err("should panics due to `Overflow`");

        assert!(err.panicked());
        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_decimals_offset_overflows_during_conversion(
        alice: Account,
    ) -> Result<()> {
        let asset = erc20::deploy(&alice.wallet).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(dec_offset_overflow_ctr(asset))
            .deploy()
            .await?
            .contract_address;

        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .convertToShares(uint!(10_U256))
            .call()
            .await
            .expect_err("should panic due to decimal offset overflow");

        assert!(err.panicked());

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
    async fn returns_more_assets_than_expected_when_no_shares_were_ever_minted(
        alice: Account,
    ) -> Result<()> {
        let tokens = uint!(100_U256);

        let (contract_addr, _) = deploy(&alice, tokens).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let shares = uint!(69_U256);
        let expected_assets = uint!(6969_U256);

        let assets = contract.convertToAssets(shares).call().await?.assets;

        assert_eq!(assets, expected_assets);

        Ok(())
    }

    #[e2e::test]
    async fn returns_assets_proportional_to_shares(
        alice: Account,
    ) -> Result<()> {
        let (contract_addr, asset_addr) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        let shares = uint!(10_U256);
        // conversion is 1:1 for empty vaults
        let expected_assets = shares;

        // Mint shares
        watch!(asset.mint(alice.address(), expected_assets))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            expected_assets
        ))?;
        watch!(contract.mint(shares, alice.address()))?;

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
            .contract_address;

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
    async fn reverts_when_decimals_offset_overflows_during_conversion(
        alice: Account,
    ) -> Result<()> {
        let asset = erc20::deploy(&alice.wallet).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(dec_offset_overflow_ctr(asset))
            .deploy()
            .await?
            .contract_address;

        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .convertToAssets(uint!(10_U256))
            .call()
            .await
            .expect_err("should panic due to decimal offset overflow");

        assert!(err.panicked());

        Ok(())
    }
}

mod max_deposit {
    use super::*;

    #[e2e::test]
    async fn returns_max_uint256_for_any_address(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, uint!(1000_U256)).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let max = contract.maxDeposit(alice.address()).call().await?.maxDeposit;
        assert_eq!(U256::MAX, max);

        let max = contract.maxDeposit(Address::ZERO).call().await?.maxDeposit;
        assert_eq!(U256::MAX, max);

        Ok(())
    }
}

mod preview_deposit {
    use super::*;

    #[e2e::test]
    async fn returns_zero_assets_for_zero_shares(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, uint!(1000_U256)).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let shares = contract.previewDeposit(U256::ZERO).call().await?.shares;
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
            contract.previewDeposit(assets_to_convert).call().await?.shares;

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
            contract.previewDeposit(assets_to_convert).call().await?.shares;

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

        let expected_shares = U256::ONE;
        let shares =
            contract.previewDeposit(assets_to_convert).call().await?.shares;

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
            .contract_address;

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
        let (contract_addr, _) = deploy(&alice, U256::MAX).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .previewDeposit(U256::MAX)
            .call()
            .await
            .expect_err("should panics due to `Overflow`");

        assert!(err.panicked());
        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_decimals_offset_overflows_during_conversion(
        alice: Account,
    ) -> Result<()> {
        let asset = erc20::deploy(&alice.wallet).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(dec_offset_overflow_ctr(asset))
            .deploy()
            .await?
            .contract_address;

        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .previewDeposit(uint!(10_U256))
            .call()
            .await
            .expect_err("should panic due to decimal offset overflow");

        assert!(err.panicked());

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
            .contract_address;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = send!(contract.deposit(uint!(10_U256), alice.address()))
            .expect_err("should return `InvalidAsset`");

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset })
        );
        Ok(())
    }

    #[e2e::test]
    async fn mints_zero_shares_for_zero_assets(alice: Account) -> Result<()> {
        let (contract_addr, asset_addr) =
            deploy(&alice, uint!(1000_U256)).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let erc20_alice = ERC20Mock::new(asset_addr, &alice.wallet);
        let alice_address = alice.address();

        watch!(erc20_alice.mint(alice_address, uint!(1000_U256)))?;

        let initial_alice_balance =
            erc20_alice.balanceOf(alice_address).call().await?._0;
        let initial_alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;

        let receipt = receipt!(contract.deposit(U256::ZERO, alice.address()))?;
        assert!(receipt.emits(Erc4626::Deposit {
            sender: alice_address,
            owner: alice_address,
            assets: U256::ZERO,
            shares: U256::ZERO,
        }));

        let alice_balance =
            erc20_alice.balanceOf(alice_address).call().await?._0;
        assert_eq!(initial_alice_balance, alice_balance);

        let alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;
        assert_eq!(initial_alice_shares, alice_shares);

        Ok(())
    }

    #[e2e::test]
    async fn mints_zero_shares_for_asset_amount_less_then_vault_assets(
        alice: Account,
    ) -> Result<()> {
        let initial_assets = uint!(1000_U256);
        let assets_to_convert = uint!(100_U256);
        let (contract_addr, asset_addr) =
            deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let erc20_alice = ERC20Mock::new(asset_addr, &alice.wallet);
        let alice_address = alice.address();

        watch!(erc20_alice.mint(alice_address, assets_to_convert))?;

        let initial_alice_balance =
            erc20_alice.balanceOf(alice_address).call().await?._0;
        let initial_alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;

        watch!(erc20_alice.regular_approve(
            alice_address,
            contract_addr,
            assets_to_convert
        ))?;

        let receipt =
            receipt!(contract.deposit(assets_to_convert, alice.address()))?;

        assert!(receipt.emits(Erc4626::Deposit {
            sender: alice_address,
            owner: alice_address,
            assets: assets_to_convert,
            shares: U256::ZERO,
        }));

        let alice_balance =
            erc20_alice.balanceOf(alice_address).call().await?._0;
        assert_eq!(initial_alice_balance - assets_to_convert, alice_balance);

        let alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;
        assert_eq!(initial_alice_shares, alice_shares);

        Ok(())
    }

    #[e2e::test]
    async fn mints_shares_equal_to_deposit_when_vault_is_empty(
        alice: Account,
    ) -> Result<()> {
        let assets_to_convert = uint!(101_U256);
        let (contract_addr, asset_addr) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let erc20_alice = ERC20Mock::new(asset_addr, &alice.wallet);
        let alice_address = alice.address();

        watch!(erc20_alice.mint(alice_address, assets_to_convert))?;

        let initial_alice_balance =
            erc20_alice.balanceOf(alice_address).call().await?._0;
        let initial_alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;

        watch!(erc20_alice.regular_approve(
            alice_address,
            contract_addr,
            assets_to_convert
        ))?;

        let receipt =
            receipt!(contract.deposit(assets_to_convert, alice.address()))?;

        assert!(receipt.emits(Erc4626::Deposit {
            sender: alice_address,
            owner: alice_address,
            assets: assets_to_convert,
            shares: assets_to_convert,
        }));

        let alice_balance =
            erc20_alice.balanceOf(alice_address).call().await?._0;
        assert_eq!(initial_alice_balance - assets_to_convert, alice_balance);

        let alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;
        assert_eq!(initial_alice_shares + assets_to_convert, alice_shares);

        Ok(())
    }

    #[e2e::test]
    async fn mints_shares_proportional_to_deposit_when_vault_has_assets(
        alice: Account,
    ) -> Result<()> {
        let initial_assets = uint!(100_U256);
        let assets_to_convert = uint!(101_U256);
        let (contract_addr, asset_addr) =
            deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let erc20_alice = ERC20Mock::new(asset_addr, &alice.wallet);
        let alice_address = alice.address();

        watch!(erc20_alice.mint(alice_address, assets_to_convert))?;
        watch!(erc20_alice.regular_approve(
            alice_address,
            contract_addr,
            assets_to_convert
        ))?;

        let initial_alice_balance =
            erc20_alice.balanceOf(alice_address).call().await?._0;
        let initial_alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;

        let receipt =
            receipt!(contract.deposit(assets_to_convert, alice.address()))?;

        let expected_shares = U256::ONE;

        assert!(receipt.emits(Erc4626::Deposit {
            sender: alice_address,
            owner: alice_address,
            assets: assets_to_convert,
            shares: expected_shares,
        }));

        let alice_balance =
            erc20_alice.balanceOf(alice_address).call().await?._0;
        assert_eq!(initial_alice_balance - assets_to_convert, alice_balance);

        let alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;
        assert_eq!(initial_alice_shares + expected_shares, alice_shares);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_no_approval_on_assets(alice: Account) -> Result<()> {
        let assets_to_convert = uint!(101_U256);
        let (contract_addr, asset_addr) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let erc20_alice = ERC20Mock::new(asset_addr, &alice.wallet);
        let alice_address = alice.address();

        watch!(erc20_alice.mint(alice_address, assets_to_convert))?;

        let err = send!(contract.deposit(assets_to_convert, alice_address))
            .expect_err("should return `SafeErc20FailedOperation`");

        assert!(err.reverted_with(Erc4626::SafeErc20FailedOperation {
            token: asset_addr
        }));

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_result_overflows(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::MAX).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = send!(contract.deposit(U256::MAX, bob.address()))
            .expect_err("should panics due to `Overflow`");

        assert!(err.panicked());
        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_decimals_offset_overflows_during_conversion(
        alice: Account,
    ) -> Result<()> {
        let asset = erc20::deploy(&alice.wallet).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(dec_offset_overflow_ctr(asset))
            .deploy()
            .await?
            .contract_address;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset, &alice.wallet);

        let assets = uint!(10_U256);

        watch!(asset.mint(alice.address(), assets))?;
        watch!(asset.regular_approve(alice.address(), contract_addr, assets))?;

        let err = send!(contract.deposit(assets, alice.address()))
            .expect_err("should panic due to decimal offset overflow");

        assert!(err.panicked());

        Ok(())
    }
}
mod max_mint {
    use super::*;

    #[e2e::test]
    async fn returns_max_uint256_for_any_address(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, uint!(1000_U256)).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let max = contract.maxMint(alice.address()).call().await?.maxMint;
        assert_eq!(U256::MAX, max);

        let max = contract.maxMint(Address::ZERO).call().await?.maxMint;
        assert_eq!(U256::MAX, max);

        Ok(())
    }
}

mod preview_mint {
    use super::*;

    #[e2e::test]
    async fn returns_zero_shares_to_zero_assets(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, uint!(1000_U256)).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let assets = contract.previewMint(U256::ZERO).call().await?.assets;
        assert_eq!(U256::ZERO, assets);

        Ok(())
    }

    #[e2e::test]
    async fn returns_more_assets_than_expected_when_no_shares_were_ever_minted(
        alice: Account,
    ) -> Result<()> {
        let tokens = uint!(100_U256);

        let (contract_addr, _) = deploy(&alice, tokens).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let shares = uint!(69_U256);
        let expected_assets = uint!(6969_U256);

        let assets = contract.previewMint(shares).call().await?.assets;

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
            .contract_address;

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

    #[e2e::test]
    async fn reverts_when_overflows(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::ONE).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .previewMint(U256::MAX)
            .call()
            .await
            .expect_err("should return `Overflow`");

        assert!(err.panicked());
        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_decimals_offset_overflows_during_conversion(
        alice: Account,
    ) -> Result<()> {
        let asset = erc20::deploy(&alice.wallet).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(dec_offset_overflow_ctr(asset))
            .deploy()
            .await?
            .contract_address;

        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .previewMint(uint!(10_U256))
            .call()
            .await
            .expect_err("should panic due to decimal offset overflow");

        assert!(err.panicked());

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
            .contract_address;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = send!(contract.mint(uint!(10_U256), alice.address()))
            .expect_err("should return `InvalidAsset`");

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset })
        );
        Ok(())
    }

    #[e2e::test]
    async fn creates_zero_shares_for_zero_assets(alice: Account) -> Result<()> {
        let (contract_addr, asset_addr) =
            deploy(&alice, uint!(1000_U256)).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        let alice_address = alice.address();
        let shares = U256::ZERO;
        let alice_assets = U256::ZERO;
        let receipt = receipt!(contract.mint(shares, alice_address))?;

        assert!(receipt.emits(Erc4626::Deposit {
            sender: alice_address,
            owner: alice_address,
            assets: alice_assets,
            shares,
        }));

        let alice_balance =
            contract.balanceOf(alice_address).call().await?.balance;
        assert_eq!(alice_balance, shares);

        let alice_assets_balance =
            asset.balanceOf(alice_address).call().await?._0;
        assert_eq!(alice_assets_balance, alice_assets);

        Ok(())
    }

    #[e2e::test]
    async fn requires_more_assets_than_expected_when_no_shares_were_ever_minted(
        alice: Account,
    ) -> Result<()> {
        let tokens = uint!(100_U256);

        let (contract_addr, asset_addr) = deploy(&alice, tokens).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        let alice_address = alice.address();
        let shares = uint!(69_U256);
        let assets = uint!(6969_U256);

        watch!(asset.mint(alice.address(), assets))?;
        watch!(asset.regular_approve(alice_address, contract_addr, assets))?;

        let initial_alice_assets =
            asset.balanceOf(alice_address).call().await?._0;
        let initial_alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;

        let receipt = receipt!(contract.mint(shares, alice_address))?;

        assert!(receipt.emits(Erc4626::Deposit {
            sender: alice_address,
            owner: alice_address,
            assets,
            shares,
        }));

        let alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;
        assert_eq!(alice_shares, shares + initial_alice_shares);

        let alice_assets = asset.balanceOf(alice_address).call().await?._0;
        assert_eq!(alice_assets, initial_alice_assets - assets);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_no_approval_on_assets(alice: Account) -> Result<()> {
        let tokens = uint!(100_U256);

        let (contract_addr, asset_addr) = deploy(&alice, tokens).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        let alice_address = alice.address();
        let shares = uint!(69_U256);
        let assets = uint!(6969_U256);

        watch!(asset.mint(alice.address(), assets))?;

        let err = send!(contract.mint(shares, alice_address))
            .expect_err("should return `SafeErc20FailedOperation`");

        assert!(err.reverted_with(Erc4626::SafeErc20FailedOperation {
            token: asset_addr
        }));

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_overflows(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::ONE).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = send!(contract.mint(U256::MAX, alice.address()))
            .expect_err("should return `Overflow`");

        assert!(err.panicked());
        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_decimals_offset_overflows_during_conversion(
        alice: Account,
    ) -> Result<()> {
        let asset = erc20::deploy(&alice.wallet).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(dec_offset_overflow_ctr(asset))
            .deploy()
            .await?
            .contract_address;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset, &alice.wallet);

        let shares = uint!(10_U256);
        let assets = shares; // expected 1:1

        watch!(asset.mint(alice.address(), assets))?;
        watch!(asset.regular_approve(alice.address(), contract_addr, assets))?;

        let err = send!(contract.mint(shares, alice.address()))
            .expect_err("should panic due to decimal offset overflow");

        assert!(err.panicked());

        Ok(())
    }
}

mod max_withdraw {
    use super::*;

    #[e2e::test]
    async fn returns_zero_for_vault_with_no_shares(
        alice: Account,
    ) -> Result<()> {
        let initial_assets = uint!(1000_U256);
        let (contract_addr, _) = deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let max =
            contract.maxWithdraw(alice.address()).call().await?.maxWithdraw;
        assert_eq!(U256::ZERO, max);

        Ok(())
    }

    #[e2e::test]
    async fn returns_zero_when_vault_is_empty(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let max =
            contract.maxWithdraw(alice.address()).call().await?.maxWithdraw;
        assert_eq!(U256::ZERO, max);

        Ok(())
    }

    #[e2e::test]
    async fn returns_convertible_assets_for_sole_share_owner(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
        let initial_assets = uint!(100_U256);
        let (contract_addr, asset_addr) =
            deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        let shares_to_mint = uint!(10_U256);
        let assets_to_deposit = uint!(1010_U256);

        // Mint some shares to alice
        watch!(asset.mint(alice.address(), assets_to_deposit))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            assets_to_deposit
        ))?;
        watch!(contract.mint(shares_to_mint, alice.address()))?;

        let max =
            contract.maxWithdraw(alice.address()).call().await?.maxWithdraw;
        assert_eq!(assets_to_deposit, max);

        let max = contract.maxWithdraw(bob.address()).call().await?.maxWithdraw;
        assert_eq!(U256::ZERO, max);

        Ok(())
    }

    #[e2e::test]
    async fn returns_convertible_assets_for_sole_share_owner_when_vault_was_empty(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
        let (contract_addr, asset_addr) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        let shares_to_mint = uint!(10_U256);
        // conversion is 1:1 for empty vaults
        let assets_to_deposit = shares_to_mint;

        // Mint some shares to alice
        watch!(asset.mint(alice.address(), assets_to_deposit))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            assets_to_deposit
        ))?;
        watch!(contract.mint(shares_to_mint, alice.address()))?;

        let max =
            contract.maxWithdraw(alice.address()).call().await?.maxWithdraw;
        assert_eq!(assets_to_deposit, max);

        let max = contract.maxWithdraw(bob.address()).call().await?.maxWithdraw;
        assert_eq!(U256::ZERO, max);

        Ok(())
    }

    #[e2e::test]
    async fn returns_convertible_assets_to_multiple_share_owners(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
        let (contract_addr, asset_addr) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let contract_bob = Erc4626::new(contract_addr, &bob.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        let shares_to_mint = uint!(10_U256);
        // conversion is 1:1 for empty vaults
        let assets_to_deposit = shares_to_mint;
        let assets_to_deposit_bob = uint!(100_U256);

        // Mint some shares to alice
        watch!(asset.mint(alice.address(), assets_to_deposit))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            assets_to_deposit
        ))?;
        watch!(contract.mint(shares_to_mint, alice.address()))?;

        // Mint some shares to bob
        watch!(asset.mint(bob.address(), assets_to_deposit_bob))?;
        watch!(asset.regular_approve(
            bob.address(),
            contract_addr,
            assets_to_deposit_bob
        ))?;
        watch!(contract_bob.mint(shares_to_mint, bob.address()))?;

        let max =
            contract.maxWithdraw(alice.address()).call().await?.maxWithdraw;
        assert_eq!(assets_to_deposit, max);

        let max = contract.maxWithdraw(bob.address()).call().await?.maxWithdraw;
        assert_eq!(assets_to_deposit, max);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_for_invalid_asset(alice: Account) -> Result<()> {
        let invalid_asset = alice.address();
        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(invalid_asset))
            .deploy()
            .await?
            .contract_address;
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

    // Cannot test when denominator overflows, as amount of shares is always >=
    // amount of assets
    #[e2e::test]
    async fn reverts_when_multiplier_overflows_during_conversion(
        alice: Account,
    ) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::MAX).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .maxWithdraw(alice.address())
            .call()
            .await
            .expect_err("should panic due to overflow");

        assert!(err.panicked());

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_decimals_offset_overflows_during_conversion(
        alice: Account,
    ) -> Result<()> {
        let asset = erc20::deploy(&alice.wallet).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(dec_offset_overflow_ctr(asset))
            .deploy()
            .await?
            .contract_address;

        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .maxWithdraw(alice.address())
            .call()
            .await
            .expect_err("should panic due to decimal offset overflow");

        assert!(err.panicked());

        Ok(())
    }
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
            .contract_address;

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

    #[e2e::test]
    async fn returns_zero_assets_for_zero_shares(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, uint!(1000_U256)).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let shares = contract.previewWithdraw(U256::ZERO).call().await?.shares;
        assert_eq!(U256::ZERO, shares);

        Ok(())
    }

    #[e2e::test]
    async fn returns_one_share_for_asset_amount_less_then_vault_assets(
        alice: Account,
    ) -> Result<()> {
        let initial_assets = uint!(1000_U256);
        let assets_to_convert = uint!(100_U256);
        let (contract_addr, _) = deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let shares =
            contract.previewWithdraw(assets_to_convert).call().await?.shares;

        assert_eq!(U256::ONE, shares);

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
            contract.previewWithdraw(assets_to_convert).call().await?.shares;

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

        let expected_shares = U256::ONE;
        let shares =
            contract.previewWithdraw(assets_to_convert).call().await?.shares;

        assert_eq!(expected_shares, shares);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_result_overflows(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::MAX).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .previewDeposit(U256::MAX)
            .call()
            .await
            .expect_err("should panics due to `Overflow`");

        assert!(err.panicked());
        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_decimals_offset_overflows_during_conversion(
        alice: Account,
    ) -> Result<()> {
        let asset = erc20::deploy(&alice.wallet).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(dec_offset_overflow_ctr(asset))
            .deploy()
            .await?
            .contract_address;

        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .previewWithdraw(uint!(10_U256))
            .call()
            .await
            .expect_err("should panic due to decimal offset overflow");

        assert!(err.panicked());

        Ok(())
    }
}

mod withdraw {
    use super::*;

    #[e2e::test]
    async fn reverts_when_exceeds_max_withdraw(alice: Account) -> Result<()> {
        let initial_assets = uint!(100_U256);
        let shares_to_mint = uint!(10_U256);
        let assets_to_deposit = uint!(1010_U256);
        let assets_to_withdraw = uint!(1011_U256); // More than deposited

        let (contract_addr, asset_addr) =
            deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        // Mint shares
        watch!(asset.mint(alice.address(), assets_to_deposit))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            assets_to_deposit
        ))?;
        watch!(contract.mint(shares_to_mint, alice.address()))?;

        let max_withdraw =
            contract.maxWithdraw(alice.address()).call().await?.maxWithdraw;

        let err = send!(contract.withdraw(
            assets_to_withdraw,
            alice.address(),
            alice.address()
        ))
        .expect_err("should fail due to exceeding max withdraw");

        assert!(err.reverted_with(Erc4626::ERC4626ExceededMaxWithdraw {
            owner: alice.address(),
            assets: assets_to_withdraw,
            max: max_withdraw
        }));
        assert_eq!(assets_to_deposit, max_withdraw);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_withdrawing_from_empty_vault(
        alice: Account,
    ) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = send!(contract.withdraw(
            uint!(100_U256),
            alice.address(),
            alice.address()
        ))
        .expect_err("should fail due to empty vault");

        assert!(err.reverted_with(Erc4626::ERC4626ExceededMaxWithdraw {
            owner: alice.address(),
            assets: uint!(100_U256),
            max: U256::ZERO
        }));

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_caller_lacks_allowance(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
        let initial_assets = uint!(100_U256);
        let shares_to_mint = uint!(10_U256);
        let assets_to_deposit = uint!(1010_U256);
        let shares_to_redeem = U256::ONE;
        let assets_to_withdraw = uint!(101_U256);

        let (contract_addr, asset_addr) =
            deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let bob_contract = Erc4626::new(contract_addr, &bob.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        // Mint shares to alice
        watch!(asset.mint(alice.address(), assets_to_deposit))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            assets_to_deposit
        ))?;
        watch!(contract.mint(shares_to_mint, alice.address()))?;

        // Bob tries to withdraw without allowance
        let err = send!(bob_contract.withdraw(
            assets_to_withdraw,
            bob.address(),
            alice.address()
        ))
        .expect_err("should fail due to insufficient allowance");

        assert!(err.reverted_with(Erc4626::ERC20InsufficientAllowance {
            spender: bob.address(),
            allowance: U256::ZERO,
            needed: shares_to_redeem
        }));

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_withdrawing_from_zero_address(
        alice: Account,
    ) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, uint!(1000_U256)).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = send!(contract.withdraw(
            U256::ZERO,
            alice.address(),
            Address::ZERO
        ))
        .expect_err("should fail due to invalid approver");

        assert!(err.reverted_with(Erc4626::ERC20InvalidApprover {
            approver: Address::ZERO
        }));

        let err =
            send!(contract.withdraw(U256::ONE, alice.address(), Address::ZERO))
                .expect_err("should fail due to exceeding max withdraw");

        assert!(err.reverted_with(Erc4626::ERC4626ExceededMaxWithdraw {
            owner: Address::ZERO,
            assets: U256::ONE,
            max: U256::ZERO
        }));

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_transfer_fails(alice: Account) -> Result<()> {
        let shares_to_mint = uint!(10_U256);
        let assets_to_deposit = shares_to_mint;
        let assets_to_withdraw = U256::ONE;

        // Deploy failing ERC20
        let failing = erc20_failing_transfer::deploy(&alice.wallet).await?;
        let failing_asset =
            ERC20FailingTransferMock::new(failing, &alice.wallet);

        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(failing))
            .deploy()
            .await?
            .contract_address;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        // Setup failing asset
        watch!(failing_asset.mint(alice.address(), assets_to_deposit))?;
        watch!(failing_asset.regular_approve(
            alice.address(),
            contract_addr,
            assets_to_deposit
        ))?;
        watch!(contract.mint(shares_to_mint, alice.address()))?;

        let err = send!(contract.withdraw(
            assets_to_withdraw,
            alice.address(),
            alice.address()
        ))
        .expect_err("should fail due to failed transfer");

        assert!(err.reverted_with(Erc4626::SafeErc20FailedOperation {
            token: failing
        }));

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_calculation_overflows(alice: Account) -> Result<()> {
        let (contract_addr, asset_addr) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        // Mint maximum shares
        watch!(asset.mint(alice.address(), U256::MAX))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            U256::MAX
        ))?;
        watch!(contract.mint(U256::MAX, alice.address()))?;

        let err = send!(contract.withdraw(
            U256::MAX,
            alice.address(),
            alice.address()
        ))
        .expect_err("should panic due to overflow");

        assert!(err.panicked());

        Ok(())
    }

    #[e2e::test]
    async fn succeeds_with_no_initial_assets(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
        let shares_to_mint = uint!(10_U256);
        let assets_to_deposit = shares_to_mint;
        let assets_to_withdraw = uint!(5_U256);

        let (contract_addr, asset_addr) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        // Initial state check
        let initial_max_withdraw =
            contract.maxWithdraw(alice.address()).call().await?.maxWithdraw;
        assert_eq!(U256::ZERO, initial_max_withdraw);

        // Mint shares
        watch!(asset.mint(alice.address(), assets_to_deposit))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            assets_to_deposit
        ))?;
        watch!(contract.mint(shares_to_mint, alice.address()))?;

        let alice_balance = asset.balanceOf(alice.address()).call().await?._0;
        let bob_balance = asset.balanceOf(bob.address()).call().await?._0;
        assert_eq!(U256::ZERO, alice_balance);
        assert_eq!(U256::ZERO, bob_balance);

        // Perform withdrawal
        let receipt = receipt!(contract.withdraw(
            assets_to_withdraw,
            alice.address(),
            alice.address()
        ))?;

        // Verify event
        assert!(receipt.emits(Erc4626::Withdraw {
            sender: alice.address(),
            receiver: alice.address(),
            owner: alice.address(),
            assets: assets_to_withdraw,
            shares: assets_to_withdraw, // 1:1 ratio expected
        }));

        // Verify updated state
        let final_max_withdraw =
            contract.maxWithdraw(alice.address()).call().await?.maxWithdraw;
        let final_max_redeem =
            contract.maxRedeem(alice.address()).call().await?.maxRedeem;
        assert_eq!(assets_to_deposit - assets_to_withdraw, final_max_withdraw);
        assert_eq!(shares_to_mint - assets_to_withdraw, final_max_redeem);

        // Perform withdrawal to a different recipient
        let receipt = receipt!(contract.withdraw(
            assets_to_withdraw,
            bob.address(),
            alice.address()
        ))?;

        // Verify event
        assert!(receipt.emits(Erc4626::Withdraw {
            sender: alice.address(),
            receiver: bob.address(),
            owner: alice.address(),
            assets: assets_to_withdraw,
            shares: assets_to_withdraw, // 1:1 ratio expected
        }));

        // Verify final state
        let final_max_withdraw =
            contract.maxWithdraw(alice.address()).call().await?.maxWithdraw;
        let final_max_redeem =
            contract.maxRedeem(alice.address()).call().await?.maxRedeem;
        assert_eq!(U256::ZERO, final_max_withdraw);
        assert_eq!(U256::ZERO, final_max_redeem);

        let alice_balance = asset.balanceOf(alice.address()).call().await?._0;
        let bob_balance = asset.balanceOf(bob.address()).call().await?._0;
        assert_eq!(assets_to_withdraw, alice_balance);
        assert_eq!(assets_to_withdraw, bob_balance);

        Ok(())
    }

    #[e2e::test]
    async fn succeeds_with_initial_assets(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
        let initial_assets = uint!(100_U256);
        let shares_to_mint = uint!(10_U256);
        let assets_to_deposit = uint!(1010_U256);
        let shares_to_redeem = U256::ONE;
        let assets_to_withdraw = uint!(101_U256);

        let (contract_addr, asset_addr) =
            deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        // Initial state check
        let initial_total_assets =
            contract.totalAssets().call().await?.totalAssets;
        assert_eq!(initial_assets, initial_total_assets);

        // Mint shares
        watch!(asset.mint(alice.address(), assets_to_deposit))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            assets_to_deposit
        ))?;
        watch!(contract.mint(shares_to_mint, alice.address()))?;

        let alice_balance = asset.balanceOf(alice.address()).call().await?._0;
        let bob_balance = asset.balanceOf(bob.address()).call().await?._0;
        assert_eq!(U256::ZERO, alice_balance);
        assert_eq!(U256::ZERO, bob_balance);

        let pre_withdraw_assets =
            contract.totalAssets().call().await?.totalAssets;

        // Perform withdrawal
        let receipt = receipt!(contract.withdraw(
            assets_to_withdraw,
            alice.address(),
            alice.address()
        ))?;

        // Verify event
        assert!(receipt.emits(Erc4626::Withdraw {
            sender: alice.address(),
            receiver: alice.address(),
            owner: alice.address(),
            assets: assets_to_withdraw,
            shares: shares_to_redeem,
        }));

        let post_withdraw_assets =
            contract.totalAssets().call().await?.totalAssets;
        assert_eq!(
            pre_withdraw_assets - assets_to_withdraw,
            post_withdraw_assets
        );

        let pre_withdraw_assets = post_withdraw_assets;

        // Perform withdrawal to different recipient
        let receipt = receipt!(contract.withdraw(
            assets_to_withdraw,
            bob.address(),
            alice.address()
        ))?;

        // Verify event
        assert!(receipt.emits(Erc4626::Withdraw {
            sender: alice.address(),
            receiver: bob.address(),
            owner: alice.address(),
            assets: assets_to_withdraw,
            shares: shares_to_redeem,
        }));

        // Verify the final state
        let post_withdraw_assets =
            contract.totalAssets().call().await?.totalAssets;
        assert_eq!(
            pre_withdraw_assets - assets_to_withdraw,
            post_withdraw_assets
        );

        let final_max_withdraw =
            contract.maxWithdraw(alice.address()).call().await?.maxWithdraw;
        let final_max_redeem =
            contract.maxRedeem(alice.address()).call().await?.maxRedeem;
        assert_eq!(
            assets_to_deposit - assets_to_withdraw - assets_to_withdraw,
            final_max_withdraw
        );
        assert_eq!(
            shares_to_mint - shares_to_redeem - shares_to_redeem,
            final_max_redeem
        );

        let alice_balance = asset.balanceOf(alice.address()).call().await?._0;
        let bob_balance = asset.balanceOf(bob.address()).call().await?._0;
        assert_eq!(assets_to_withdraw, alice_balance);
        assert_eq!(assets_to_withdraw, bob_balance);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_for_invalid_asset(alice: Account) -> Result<()> {
        let invalid_asset = alice.address();
        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(invalid_asset))
            .deploy()
            .await?
            .contract_address;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = send!(contract.withdraw(
            uint!(100_U256),
            alice.address(),
            alice.address()
        ))
        .expect_err("should return `InvalidAsset`");

        assert!(
            err.reverted_with(Erc4626::InvalidAsset { asset: invalid_asset })
        );

        Ok(())
    }

    #[e2e::test]
    async fn succeeds_with_multiple_holders_no_initial_assets(
        alice: Account,
        bob: Account,
        charlie: Account,
    ) -> Result<()> {
        let (contract_addr, asset_addr) = deploy(&alice, U256::ZERO).await?;
        let contract_alice = Erc4626::new(contract_addr, &alice.wallet);
        let contract_bob = Erc4626::new(contract_addr, &bob.wallet);
        let contract_charlie = Erc4626::new(contract_addr, &charlie.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        // Mint and approve for all users
        for user in [&alice, &bob, &charlie] {
            watch!(asset.mint(user.address(), uint!(1000_U256)))?;
            watch!(asset.regular_approve(
                user.address(),
                contract_addr,
                uint!(1000_U256)
            ))?;
        }

        // Each user deposits different amounts
        watch!(contract_alice.mint(uint!(100_U256), alice.address()))?;
        watch!(contract_bob.mint(uint!(200_U256), bob.address()))?;
        watch!(contract_charlie.mint(uint!(300_U256), charlie.address()))?;

        // Each user withdraws different percentages
        watch!(contract_alice.withdraw(
            uint!(50_U256),
            alice.address(),
            alice.address()
        ))?; // 50% for alice
        watch!(contract_bob.withdraw(
            uint!(100_U256),
            bob.address(),
            bob.address()
        ))?; // 50% for bob
        watch!(contract_charlie.withdraw(
            uint!(300_U256),
            charlie.address(),
            charlie.address()
        ))?; // 100% for charlie

        // Verify final balances
        assert_eq!(
            uint!(50_U256),
            contract_alice
                .maxWithdraw(alice.address())
                .call()
                .await?
                .maxWithdraw
        );
        assert_eq!(
            uint!(100_U256),
            contract_alice.maxWithdraw(bob.address()).call().await?.maxWithdraw
        );
        assert_eq!(
            U256::ZERO,
            contract_alice
                .maxWithdraw(charlie.address())
                .call()
                .await?
                .maxWithdraw
        );

        Ok(())
    }

    #[e2e::test]
    async fn succeeds_with_multiple_holders_with_initial_assets(
        alice: Account,
        bob: Account,
        charlie: Account,
    ) -> Result<()> {
        let initial_assets = uint!(100_U256);
        let (contract_addr, asset_addr) =
            deploy(&alice, initial_assets).await?;
        let contract_alice = Erc4626::new(contract_addr, &alice.wallet);
        let contract_bob = Erc4626::new(contract_addr, &bob.wallet);
        let contract_charlie = Erc4626::new(contract_addr, &charlie.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        // Record initial total assets
        let initial_total =
            contract_alice.totalAssets().call().await?.totalAssets;

        // Mint and approve for all users
        for user in [&alice, &bob, &charlie] {
            watch!(asset.mint(user.address(), uint!(10000_U256)))?;
            watch!(asset.regular_approve(
                user.address(),
                contract_addr,
                uint!(10000_U256)
            ))?;
        }

        // Each user deposits different amounts
        watch!(contract_alice.mint(uint!(10_U256), alice.address()))?;
        watch!(contract_bob.mint(uint!(20_U256), bob.address()))?;
        watch!(contract_charlie.mint(uint!(30_U256), charlie.address()))?;

        // Verify share distribution considers initial assets
        let alice_assets_before = contract_alice
            .maxWithdraw(alice.address())
            .call()
            .await?
            .maxWithdraw;
        let bob_assets_before =
            contract_alice.maxWithdraw(bob.address()).call().await?.maxWithdraw;
        let charlie_assets_before = contract_alice
            .maxWithdraw(charlie.address())
            .call()
            .await?
            .maxWithdraw;

        // Each user withdraws
        watch!(contract_alice.withdraw(
            alice_assets_before,
            alice.address(),
            alice.address()
        ))?; // 100%
        watch!(contract_bob.withdraw(
            uint!(1010_U256),
            bob.address(),
            bob.address()
        ))?; // 50%
        watch!(contract_charlie.withdraw(
            charlie_assets_before,
            charlie.address(),
            charlie.address()
        ))?; // 100%

        // Verify proportional distribution of initial assets was maintained
        let remaining_bob =
            contract_alice.maxWithdraw(bob.address()).call().await?.maxWithdraw;
        assert_eq!(bob_assets_before - uint!(1010_U256), remaining_bob);

        // Verify total assets consistency
        let final_total =
            contract_alice.totalAssets().call().await?.totalAssets;
        let expected_remaining = initial_total + uint!(6060_U256)
            - alice_assets_before
            - uint!(1010_U256)
            - charlie_assets_before;
        assert_eq!(expected_remaining, final_total);

        Ok(())
    }

    #[e2e::test]
    async fn maintains_share_price_ratio(alice: Account) -> Result<()> {
        let initial_assets = uint!(100_U256);
        let (contract_addr, asset_addr) =
            deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        let shares = uint!(10_U256);
        let assets = uint!(2000_U256);

        // Setup deposits
        watch!(asset.mint(alice.address(), assets))?;
        watch!(asset.regular_approve(alice.address(), contract_addr, assets))?;
        watch!(contract.mint(shares, alice.address()))?;

        // Record initial conversion rate
        let initial_rate =
            contract.convertToAssets(U256::ONE).call().await?.assets;

        // Perform partial withdrawal
        watch!(contract.withdraw(
            uint!(500_U256),
            alice.address(),
            alice.address()
        ))?;

        // Verify conversion rate remains the same
        let final_rate =
            contract.convertToAssets(U256::ONE).call().await?.assets;
        assert_eq!(initial_rate, final_rate);

        Ok(())
    }

    #[e2e::test]
    async fn maintains_state_consistency_after_failed_withdrawal(
        alice: Account,
    ) -> Result<()> {
        let initial_assets = uint!(100_U256);
        let shares_to_mint = uint!(10_U256);
        let assets_to_deposit = uint!(1010_U256);
        let excessive_assets_to_withdraw = uint!(1011_U256);

        let (contract_addr, asset_addr) =
            deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        // Setup initial state
        watch!(asset.mint(alice.address(), assets_to_deposit))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            assets_to_deposit
        ))?;
        watch!(contract.mint(shares_to_mint, alice.address()))?;

        // Record state before failed withdrawal
        let pre_total_assets = contract.totalAssets().call().await?.totalAssets;
        let pre_max_withdraw =
            contract.maxWithdraw(alice.address()).call().await?.maxWithdraw;
        let pre_max_redeem =
            contract.maxRedeem(alice.address()).call().await?.maxRedeem;

        // Attempt excessive withdrawal
        send!(contract.withdraw(
            excessive_assets_to_withdraw,
            alice.address(),
            alice.address()
        ))
        .expect_err("should fail due to exceeding max withdraw");

        // Verify state remains unchanged
        let post_total_assets =
            contract.totalAssets().call().await?.totalAssets;
        let post_max_withdraw =
            contract.maxWithdraw(alice.address()).call().await?.maxWithdraw;
        let post_max_redeem =
            contract.maxRedeem(alice.address()).call().await?.maxRedeem;

        assert_eq!(pre_total_assets, post_total_assets);
        assert_eq!(pre_max_withdraw, post_max_withdraw);
        assert_eq!(pre_max_redeem, post_max_redeem);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_decimals_offset_overflows_during_conversion(
        alice: Account,
    ) -> Result<()> {
        let asset = erc20::deploy(&alice.wallet).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(dec_offset_overflow_ctr(asset))
            .deploy()
            .await?
            .contract_address;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = send!(contract.withdraw(
            uint!(10_U256),
            alice.address(),
            alice.address()
        ))
        .expect_err("should panic due to decimal offset overflow");

        assert!(err.panicked());

        Ok(())
    }
}

mod max_redeem {
    use super::*;

    #[e2e::test]
    async fn returns_zero_for_vault_with_no_shares(
        alice: Account,
    ) -> Result<()> {
        let initial_assets = uint!(1000_U256);
        let (contract_addr, _) = deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let max = contract.maxRedeem(alice.address()).call().await?.maxRedeem;
        assert_eq!(U256::ZERO, max);

        Ok(())
    }

    #[e2e::test]
    async fn returns_zero_when_vault_is_empty(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let max = contract.maxRedeem(alice.address()).call().await?.maxRedeem;
        assert_eq!(U256::ZERO, max);

        Ok(())
    }

    #[e2e::test]
    async fn returns_full_share_balance_for_owner(
        alice: Account,
    ) -> Result<()> {
        let initial_assets = uint!(100_U256);
        let (contract_addr, asset_addr) =
            deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        let assets_to_deposit = uint!(6969_U256);
        let shares_to_mint = uint!(69_U256);

        // Mint some shares to alice
        watch!(asset.mint(alice.address(), assets_to_deposit))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            assets_to_deposit
        ))?;
        watch!(contract.mint(shares_to_mint, alice.address()))?;

        let max = contract.maxRedeem(alice.address()).call().await?.maxRedeem;
        assert_eq!(shares_to_mint, max);

        Ok(())
    }

    #[e2e::test]
    async fn returns_balance_after_partial_transfer(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
        let initial_assets = uint!(100_U256);
        let (contract_addr, asset_addr) =
            deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        let assets_to_deposit = uint!(8080_U256);
        let shares_to_mint = uint!(80_U256);
        let transfer_amount = uint!(40_U256);

        // Mint shares to alice
        watch!(asset.mint(alice.address(), assets_to_deposit))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            assets_to_deposit
        ))?;
        watch!(contract.mint(shares_to_mint, alice.address()))?;

        // Transfer some shares to bob
        watch!(contract.transfer(bob.address(), transfer_amount))?;

        let alice_max =
            contract.maxRedeem(alice.address()).call().await?.maxRedeem;
        let bob_max = contract.maxRedeem(bob.address()).call().await?.maxRedeem;

        assert_eq!(shares_to_mint - transfer_amount, alice_max);
        assert_eq!(transfer_amount, bob_max);

        Ok(())
    }

    #[e2e::test]
    async fn returns_updated_balance_after_mint(alice: Account) -> Result<()> {
        let initial_assets = uint!(100_U256);
        let (contract_addr, asset_addr) =
            deploy(&alice, initial_assets).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);

        let first_mint = uint!(10_U256);
        let second_mint = uint!(50_U256);
        let assets_for_first_mint = uint!(1010_U256);
        let assets_for_second_mint = uint!(5050_U256);

        // First mint
        watch!(asset.mint(alice.address(), assets_for_first_mint))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            assets_for_first_mint
        ))?;
        watch!(contract.mint(first_mint, alice.address()))?;

        let max_after_first =
            contract.maxRedeem(alice.address()).call().await?.maxRedeem;
        assert_eq!(first_mint, max_after_first);

        // Second mint
        watch!(asset.mint(alice.address(), assets_for_second_mint))?;
        watch!(asset.regular_approve(
            alice.address(),
            contract_addr,
            assets_for_second_mint
        ))?;
        watch!(contract.mint(second_mint, alice.address()))?;

        let max_after_second =
            contract.maxRedeem(alice.address()).call().await?.maxRedeem;
        assert_eq!(first_mint + second_mint, max_after_second);

        Ok(())
    }
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
            .contract_address;

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

    #[e2e::test]
    async fn returns_zero_shares_to_zero_assets(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, uint!(1000_U256)).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let assets = contract.previewRedeem(U256::ZERO).call().await?.assets;
        assert_eq!(U256::ZERO, assets);

        Ok(())
    }

    #[e2e::test]
    async fn returns_more_assets_than_expected_when_no_shares_were_ever_minted(
        alice: Account,
    ) -> Result<()> {
        let tokens = uint!(100_U256);

        let (contract_addr, _) = deploy(&alice, tokens).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let shares = uint!(69_U256);
        let expected_assets = uint!(6969_U256);

        let assets = contract.previewRedeem(shares).call().await?.assets;

        assert_eq!(assets, expected_assets);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_overflows(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::ONE).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .previewRedeem(U256::MAX)
            .call()
            .await
            .expect_err("should return `Overflow`");

        assert!(err.panicked());
        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_decimals_offset_overflows_during_conversion(
        alice: Account,
    ) -> Result<()> {
        let asset = erc20::deploy(&alice.wallet).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(dec_offset_overflow_ctr(asset))
            .deploy()
            .await?
            .contract_address;

        let contract = Erc4626::new(contract_addr, &alice.wallet);

        let err = contract
            .previewRedeem(uint!(10_U256))
            .call()
            .await
            .expect_err("should panic due to decimal offset overflow");

        assert!(err.panicked());

        Ok(())
    }
}

mod redeem {
    use super::*;

    #[e2e::test]
    async fn reverts_when_exceeded_max_redeem_zero_balance(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::ZERO).await?;
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

    #[e2e::test]
    async fn zero_shares_for_zero_assets(alice: Account) -> Result<()> {
        let (contract_addr, asset_addr) =
            deploy(&alice, uint!(1000_U256)).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);
        let alice_address = alice.address();

        let initial_alice_assets =
            asset.balanceOf(alice_address).call().await?._0;
        let initial_alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;

        let receipt = receipt!(contract.redeem(
            U256::ZERO,
            alice_address,
            alice_address
        ))?;

        assert!(receipt.emits(Erc4626::Withdraw {
            sender: alice_address,
            receiver: alice_address,
            owner: alice_address,
            assets: U256::ZERO,
            shares: U256::ZERO,
        }));

        let alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;
        assert_eq!(initial_alice_shares, alice_shares);

        let alice_assets = asset.balanceOf(alice_address).call().await?._0;
        assert_eq!(initial_alice_assets, alice_assets);

        Ok(())
    }

    #[e2e::test]
    async fn full_share_balance_for_owner(alice: Account) -> Result<()> {
        let tokens = uint!(100_U256);

        let (contract_addr, asset_addr) = deploy(&alice, tokens).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);
        let alice_address = alice.address();

        let assets = uint!(6969_U256);
        let shares = uint!(69_U256);

        // Mint some shares to alice
        watch!(asset.mint(alice.address(), assets))?;
        watch!(asset.regular_approve(alice.address(), contract_addr, assets))?;
        watch!(contract.mint(shares, alice.address()))?;

        let initial_alice_assets =
            asset.balanceOf(alice_address).call().await?._0;
        let initial_alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;

        let receipt =
            receipt!(contract.redeem(shares, alice_address, alice_address))?;

        assert!(receipt.emits(Erc4626::Withdraw {
            sender: alice_address,
            receiver: alice_address,
            owner: alice_address,
            assets,
            shares
        }));

        let alice_shares =
            contract.balanceOf(alice_address).call().await?.balance;
        assert_eq!(initial_alice_shares - shares, alice_shares);

        let alice_assets = asset.balanceOf(alice_address).call().await?._0;
        assert_eq!(initial_alice_assets + assets, alice_assets);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_insufficient_allowance(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
        let tokens = uint!(100_U256);

        let (contract_addr, asset_addr) = deploy(&alice, tokens).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);
        let contract_bob = Erc4626::new(contract_addr, &bob.wallet);
        let alice_address = alice.address();

        let assets = uint!(6969_U256);
        let shares = uint!(69_U256);

        // Mint some shares to alice
        watch!(asset.mint(alice.address(), assets))?;
        watch!(asset.regular_approve(alice.address(), contract_addr, assets))?;
        watch!(contract.mint(shares, alice.address()))?;

        let err =
            send!(contract_bob.redeem(shares, alice_address, alice_address))
                .expect_err("should return `ERC20InsufficientAllowance`");

        assert!(err.reverted_with(Erc4626::ERC20InsufficientAllowance {
            spender: bob.address(),
            allowance: U256::ZERO,
            needed: shares,
        }));

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_exceeded_max_redeem(alice: Account) -> Result<()> {
        let tokens = uint!(100_U256);

        let (contract_addr, asset_addr) = deploy(&alice, tokens).await?;
        let contract = Erc4626::new(contract_addr, &alice.wallet);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);
        let alice_address = alice.address();

        let assets = uint!(6969_U256);
        let shares = uint!(69_U256);

        // Mint some shares to alice
        watch!(asset.mint(alice.address(), assets))?;
        watch!(asset.regular_approve(alice.address(), contract_addr, assets))?;
        watch!(contract.mint(shares, alice.address()))?;

        let err = send!(contract.redeem(
            shares + U256::ONE,
            alice_address,
            alice_address
        ))
        .expect_err("should return `ERC4626ExceededMaxRedeem`");

        assert!(err.reverted_with(Erc4626::ERC4626ExceededMaxRedeem {
            owner: alice.address(),
            shares: shares + U256::ONE,
            max: shares,
        }));

        Ok(())
    }
}
