#![cfg(feature = "e2e")]

use std::{assert_eq,  println};

use abi::Erc20Flashmint;
use alloy::{
    primitives::{address, uint, Address, U256},
    sol,
};
use e2e::{
    receipt, send, Account, ReceiptExt,
    Revert,
};
use eyre::Result;
// use stylus_sdk::contract::address;
use mock::borrower;

use crate::Erc20FlashmintExample::constructorCall;

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
    Erc20FlashmintExample::constructorCall {
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
    let contract = Erc20Flashmint::new(contract_addr, &alice.wallet);
    let Erc20Flashmint::totalSupplyReturn { totalSupply: total_supply } =
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
    let contract = Erc20Flashmint::new(contract_addr, &alice.wallet);

    let Erc20Flashmint::flashFeeReturn { fee } =
        contract.flashFee(contract_addr, uint!(1000_U256)).call().await?;
    assert_eq!(fee, FLASH_FEE_AMOUNT);
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
    let contract = Erc20Flashmint::new(contract_addr, &alice.wallet);
    let invalid_token_address =
        address!("a6CB74633b3F981AB239ed5fe17E714184236b9C");

    let err = send!(contract.flashFee(invalid_token_address, uint!(1000_U256)))
        .expect_err("should fail with ERC3156UnsupportedToken");
    assert!(err.reverted_with(Erc20Flashmint::ERC3156UnsupportedToken {
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
    let contract = Erc20Flashmint::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    // let Erc20Flashmint::maxFlashLoanReturn { maxLoan } =
    //     contract.maxFlashLoan(contract_addr).call().await?;
    // assert_eq!(maxLoan, U256::MAX);

    let _ = receipt!(contract.mint(alice_addr, uint!(1000_U256)))?;
    let Erc20Flashmint::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;
    println!("balance: {}", balance);

    // let Erc20Flashmint::maxFlashLoanReturn { maxLoan:bobMaxLoan } =
    //     contract.maxFlashLoan(contract_addr).call().await?;

    let Erc20Flashmint::totalSupplyReturn { totalSupply } =
        contract.totalSupply().call().await?;

    // let Erc20Flashmint::maxFlashLoanReturn { maxLoan: testMaxLoan } =
    //     contract.maxFlashLoan(contract_addr).call().await?;
    // let Erc20Flashmint::balanceOfReturn { balance } =
    // contract.balanceOf(bob.address()).call().await?;
    // let Erc20Flashmint::balanceOfReturn { balance: alice_balance } =
    // contract.balanceOf(alice.address()).call().await?;
    let Erc20Flashmint::maxFlashLoanReturn { maxLoan } =
        contract.maxFlashLoan(contract_addr).call().await?;
    println!("totalSupply: {}", totalSupply);
    println!("maxLoan: {}", maxLoan);
    // println!("bobMaxLoan: {}", bobMaxLoan);
    // println!("testMaxLoan: {}", testMaxLoan);
    // println!("balance: {}", balance);
    // println!("alice_balance: {}", alice_balance);
    // assert_eq!(bobMaxLoan, U256::MAX);
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
    let contract = Erc20Flashmint::new(contract_addr, &alice.wallet);
    let random_address = address!("a6CB74633b3F981AB239ed5fe17E714184236b9C");

    let Erc20Flashmint::maxFlashLoanReturn { maxLoan } =
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

// #[e2e::test]
// async fn flash_loan(alice: Account) -> Result<()> {
//      let contract_addr = alice
//         .as_deployer()
//         .with_default_constructor::<constructorCall>()
//         .deploy()
//         .await?
//         .address()?;
//     let contract = Erc20Flashmint::new(contract_addr, &alice.wallet);
//     let alice_addr = alice.address();
//     let random_address =
// address!("a6CB74633b3F981AB239ed5fe17E714184236b9C");

//     let Erc20Flashmint::maxFlashLoanReturn { maxLoan } =
//         contract.maxFlashLoan(random_address).call().await?;
//     assert_eq!(maxLoan, U256::MIN);
//     Ok(())
// }
