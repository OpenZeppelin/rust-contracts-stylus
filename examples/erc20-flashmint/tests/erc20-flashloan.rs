#![cfg(feature = "e2e")]

use std::{assert_eq, println};

use abi::Erc20FlashMint;
use alloy::{
    primitives::{address, uint, Address, U256},
    sol,
};
use e2e::{receipt, send, Account, ReceiptExt, Revert};
use eyre::Result;
// use stylus_sdk::contract::address;
use mock::borrower;

use crate::Erc20FlashMintExample::constructorCall;

mod abi;
mod mock;

sol!("src/constructor.sol");

const RECIVER_ADDDRESS: Address =
    address!("1000000000000000000000000000000000000000");
const FLASH_FEE_AMOUNT: U256 = uint!(1_000_U256);

impl Default for constructorCall {
    fn default() -> Self {
        ctr()
    }
}

fn ctr() -> constructorCall {
    Erc20FlashMintExample::constructorCall {
        flash_fee_receiver_address_: RECIVER_ADDDRESS,
        flash_fee_amount_: FLASH_FEE_AMOUNT,
    }
}

// ============================================================================
// Integration Tests: ERC-20 Token + Metadata Extension
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
    let Erc20FlashMint::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply().call().await?;
    assert_eq!(total_supply, U256::ZERO);
    Ok(())
}
#[e2e::test]
async fn flash_fee(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
    let mint_amount = uint!(10000000_U256);
    let supposed_fee = mint_amount.checked_mul(U256::from(1)).unwrap() / U256::from(100);
    let Erc20FlashMint::flashFeeReturn { fee } =
        contract.flashFee(contract_addr, mint_amount) .call().await?;
    assert_eq!(fee, supposed_fee);
    Ok(())
}

#[e2e::test]
async fn flash_fee_rejects_unsupported_token(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
    let invalid_token_address =
        address!("a6CB74633b3F981AB239ed5fe17E714184236b9C");

    let err = send!(contract.flashFee(invalid_token_address, uint!(1000_U256)))
        .expect_err("should fail with ERC3156UnsupportedToken");
    assert!(err.reverted_with(Erc20FlashMint::ERC3156UnsupportedToken {
        token: invalid_token_address
    }));
    Ok(())
}

#[e2e::test]
async fn max_flash_loan(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let Erc20FlashMint::maxFlashLoanReturn { maxLoan } =
    contract.maxFlashLoan(contract_addr).call().await?;
    assert_eq!(maxLoan, U256::MAX);
    let mint_amount = uint!(10000000_U256);
    let _ = receipt!(contract.mint(alice_addr, mint_amount))?;
    let Erc20FlashMint::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;
    assert_eq!(balance,mint_amount);

    let Erc20FlashMint::totalSupplyReturn { totalSupply } =
        contract.totalSupply().call().await?;

    let Erc20FlashMint::maxFlashLoanReturn { maxLoan } =
        contract.maxFlashLoan(contract_addr).call().await?;
    Ok(())
}

#[e2e::test]
async fn max_flash_loan_invalid_address(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
    let random_address = address!("a6CB74633b3F981AB239ed5fe17E714184236b9C");

    let Erc20FlashMint::maxFlashLoanReturn { maxLoan } =
        contract.maxFlashLoan(random_address).call().await?;
    assert_eq!(maxLoan, U256::MIN);
    Ok(())
}

#[e2e::test]
async fn can_deploy_mock_borrower(alice: Account) -> Result<()> {
    let borrower = borrower::deploy(&alice.wallet).await?;
    assert_eq!(borrower.is_zero(), false);
    Ok(())
}

#[e2e::test]
async fn flash_loan(alice: Account) -> Result<()> {
     let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let random_address =
address!("a6CB74633b3F981AB239ed5fe17E714184236b9C");

    let Erc20FlashMint::maxFlashLoanReturn { maxLoan } =
        contract.maxFlashLoan(random_address).call().await?;
    assert_eq!(maxLoan, U256::MIN);
    Ok(())
}
