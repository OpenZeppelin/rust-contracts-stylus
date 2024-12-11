#![cfg(feature = "e2e")]

use abi::Erc20FlashMint;
use alloy::{
    primitives::{address, uint, Address, U256},
    sol,
};
use e2e::{receipt, send, Account, ReceiptExt, Revert};
use eyre::Result;
use mock::{borrower, borrower::ERC3156FlashBorrowerMock};

use crate::Erc20FlashMintExample::constructorCall;

mod abi;
mod mock;

sol!("src/constructor.sol");

const FEE_RECEIVER: Address =
    address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");
const FLASH_FEE_AMOUNT: U256 = uint!(100_U256);

impl Default for constructorCall {
    fn default() -> Self {
        ctr(FEE_RECEIVER, FLASH_FEE_AMOUNT)
    }
}

fn ctr(fee_receiver: Address, fee_amount: U256) -> constructorCall {
    Erc20FlashMintExample::constructorCall {
        flashFeeReceiverAddress_: fee_receiver,
        flashFeeAmount_: fee_amount,
    }
}

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);

    let max = contract.maxFlashLoan(contract_addr).call().await?.maxLoan;
    let fee = contract.flashFee(contract_addr, U256::from(1)).call().await?.fee;

    assert_eq!(max, U256::MAX);
    assert_eq!(fee, FLASH_FEE_AMOUNT);

    Ok(())
}
// #[e2e::test]
// async fn flash_fee(alice: Account) -> Result<()> {
//     let contract_addr = alice
//         .as_deployer()
//         .with_default_constructor::<constructorCall>()
//         .deploy()
//         .await?
//         .address()?;
//     let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
//     let mint_amount = uint!(10000000_U256);
//     let supposed_fee =
//         mint_amount.checked_mul(U256::from(1)).unwrap() / U256::from(100);
//     let Erc20FlashMint::flashFeeReturn { fee } =
//         contract.flashFee(contract_addr, mint_amount).call().await?;
//     assert_eq!(fee, supposed_fee);
//     Ok(())
// }

// #[e2e::test]
// async fn flash_fee_rejects_unsupported_token(alice: Account) -> Result<()> {
//     let contract_addr = alice
//         .as_deployer()
//         .with_default_constructor::<constructorCall>()
//         .deploy()
//         .await?
//         .address()?;
//     let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
//     let invalid_token_address =
//         address!("a6CB74633b3F981AB239ed5fe17E714184236b9C");

//     let err = send!(contract.flashFee(invalid_token_address,
// uint!(1000_U256)))         .expect_err("should fail with
// ERC3156UnsupportedToken");     assert!(err.
// reverted_with(Erc20FlashMint::ERC3156UnsupportedToken {         token:
// invalid_token_address     }));
//     Ok(())
// }

// #[e2e::test]
// async fn max_flash_loan(alice: Account) -> Result<()> {
//     let contract_addr = alice
//         .as_deployer()
//         .with_default_constructor::<constructorCall>()
//         .deploy()
//         .await?
//         .address()?;
//     let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
//     let alice_addr = alice.address();

//     let Erc20FlashMint::maxFlashLoanReturn { maxLoan } =
//         contract.maxFlashLoan(contract_addr).call().await?;
//     assert_eq!(maxLoan, U256::MAX);
//     let mint_amount = uint!(10000000_U256);
//     let _ = receipt!(contract.mint(alice_addr, mint_amount))?;
//     let Erc20FlashMint::balanceOfReturn { balance } =
//         contract.balanceOf(alice_addr).call().await?;
//     assert_eq!(balance, mint_amount);

//     let Erc20FlashMint::totalSupplyReturn { totalSupply } =
//         contract.totalSupply().call().await?;

//     let Erc20FlashMint::maxFlashLoanReturn { maxLoan } =
//         contract.maxFlashLoan(contract_addr).call().await?;
//     Ok(())
// }

// #[e2e::test]
// async fn max_flash_loan_invalid_address(alice: Account) -> Result<()> {
//     let contract_addr = alice
//         .as_deployer()
//         .with_default_constructor::<constructorCall>()
//         .deploy()
//         .await?
//         .address()?;
//     let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
//     let random_address =
// address!("a6CB74633b3F981AB239ed5fe17E714184236b9C");

//     let Erc20FlashMint::maxFlashLoanReturn { maxLoan } =
//         contract.maxFlashLoan(random_address).call().await?;
//     assert_eq!(maxLoan, U256::MIN);
//     Ok(())
// }

// #[e2e::test]
// async fn can_deploy_mock_borrower(alice: Account) -> Result<()> {
//     let borrower = borrower::deploy(&alice.wallet).await?;
//     assert_eq!(borrower.is_zero(), false);
//     Ok(())
// }

// #[e2e::test]
// async fn flash_loan(alice: Account) -> Result<()> {
//     let contract_addr = alice
//         .as_deployer()
//         .with_default_constructor::<constructorCall>()
//         .deploy()
//         .await?
//         .address()?;
//     let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
//     let alice_addr = alice.address();
//     let random_address =
// address!("a6CB74633b3F981AB239ed5fe17E714184236b9C");

//     let Erc20FlashMint::maxFlashLoanReturn { maxLoan } =
//         contract.maxFlashLoan(random_address).call().await?;
//     assert_eq!(maxLoan, U256::MIN);
//     Ok(())
// }
