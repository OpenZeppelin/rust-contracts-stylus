#![cfg(feature = "e2e")]

use abi::Erc20FlashMint;
use alloy::{
    primitives::{address, uint, Address, U256},
    sol,
};
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt, Revert};
use eyre::Result;
use mock::borrower;

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
    let mint_amount = uint!(1_000_000_U256);
    let _ = watch!(contract.mint(alice_addr, mint_amount))?;

    let max_loan = contract.maxFlashLoan(contract_addr).call().await?.maxLoan;
    assert_eq!(U256::MAX - mint_amount, max_loan);

    Ok(())
}

#[e2e::test]
async fn max_flash_loan_return_zero_if_no_more_tokens_to_mint(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let _ = watch!(contract.mint(alice_addr, U256::MAX))?;

    let max_loan = contract.maxFlashLoan(contract_addr).call().await?.maxLoan;
    assert_eq!(U256::MIN, max_loan);

    Ok(())
}

#[e2e::test]
async fn max_flash_loan_returns_zero_on_invalid_address(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let mint_amount = uint!(1_000_000_U256);
    let _ = watch!(contract.mint(alice_addr, mint_amount))?;

    // non-token address
    let max_loan = contract.maxFlashLoan(alice_addr).call().await?.maxLoan;
    assert_eq!(U256::MIN, max_loan);

    // works for zero address too
    let max_loan = contract.maxFlashLoan(Address::ZERO).call().await?.maxLoan;
    assert_eq!(U256::MIN, max_loan);

    Ok(())
}

// NOTE: this behavior is assumed for our implementation, but other
// implementations may have different behavior (e.g. return fee as a percentage
// of the passed amount).
#[e2e::test]
async fn flash_fee_returns_same_value_regardless_of_amount(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);

    let amounts = &[U256::ZERO, U256::from(1), U256::from(1000), U256::MAX];
    for &amount in amounts {
        let fee = contract.flashFee(contract_addr, amount).call().await?.fee;
        assert_eq!(fee, FLASH_FEE_AMOUNT);
    }

    Ok(())
}

#[e2e::test]
async fn flash_fee_reverts_on_unsupported_token(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);

    let unsupported_token = alice.address();

    let err = contract
        .flashFee(unsupported_token, U256::from(1))
        .call()
        .await
        .expect_err("should return `UnsupportedToken`");

    assert!(err.reverted_with(Erc20FlashMint::ERC3156UnsupportedToken {
        token: unsupported_token
    }));

    let err = contract
        .flashFee(Address::ZERO, U256::from(1))
        .call()
        .await
        .expect_err("should return `UnsupportedToken`");

    assert!(err.reverted_with(Erc20FlashMint::ERC3156UnsupportedToken {
        token: Address::ZERO
    }));

    Ok(())
}

#[e2e::test]
async fn flash_loan_reverts_when_loan_amount_greater_than_max_loan(
    alice: Account,
) -> Result<()> {
    let erc20_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    let max_loan = U256::from(1);

    let _ = watch!(erc20.mint(borrower_addr, U256::MAX - max_loan))?;

    let err = send!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        U256::from(2),
        vec![].into()
    ))
    .expect_err("should revert with `ERC3156ExceededMaxLoan`");

    assert!(err.reverted_with(Erc20FlashMint::ERC3156ExceededMaxLoan {
        maxLoan: max_loan
    }));

    Ok(())
}

#[e2e::test]
async fn flash_loan_reverts_with_exceeded_max_with_unsupported_token(
    alice: Account,
) -> Result<()> {
    let erc20_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    let invalid_token = alice.address();

    let err = send!(erc20.flashLoan(
        borrower_addr,
        invalid_token,
        U256::from(1),
        vec![].into()
    ))
    .expect_err("should revert with `ERC3156ExceededMaxLoan`");

    assert!(err.reverted_with(Erc20FlashMint::ERC3156ExceededMaxLoan {
        maxLoan: U256::ZERO
    }));

    Ok(())
}

#[e2e::test]
async fn flash_loan_reverts_with_unsupported_token_with_zero_loan_amount_and_unsupported_token(
    alice: Account,
) -> Result<()> {
    let erc20_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    let invalid_token = alice.address();

    let err = send!(erc20.flashLoan(
        borrower_addr,
        invalid_token,
        U256::ZERO,
        vec![].into()
    ))
    .expect_err("should revert with `ERC3156UnsupportedToken`");

    assert!(err.reverted_with(Erc20FlashMint::ERC3156UnsupportedToken {
        token: invalid_token
    }));

    Ok(())
}

#[e2e::test]
async fn flash_loan_reverts_when_invalid_receiver(
    alice: Account,
) -> Result<()> {
    let erc20_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    let _ = watch!(erc20.mint(borrower_addr, FLASH_FEE_AMOUNT))?;

    let invalid_receivers = &[alice.address(), Address::ZERO];

    for &invalid_receiver in invalid_receivers {
        let err = send!(erc20.flashLoan(
            invalid_receiver,
            erc20_addr,
            U256::from(1),
            vec![].into()
        ))
        .expect_err("should revert with `ERC3156InvalidReceiver`");

        assert!(
            err.reverted_with(Erc20FlashMint::ERC3156InvalidReceiver {
                receiver: invalid_receiver
            }),
            "receiver: {invalid_receiver}"
        );
    }

    //     let err = send!(erc20.flashLoan(
    //         borrower_addr,
    //         erc20_addr,
    //         U256::from(1),
    //         vec![].into()
    //     ))
    //     .expect_err("should revert with `ERC3156InvalidReceiver`");

    //     assert!(
    //         err.reverted_with(Erc20FlashMint::ERC3156InvalidReceiver {
    //             receiver: borrower_addr
    //         }),
    //         "receiver: {borrower_addr}"
    //     );

    Ok(())
}

#[e2e::test]
async fn flash_loan_with_fee_and_fee_receiver(alice: Account) -> Result<()> {
    let erc20_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    let _ = watch!(erc20.mint(borrower_addr, FLASH_FEE_AMOUNT))?;

    let balance = erc20.balanceOf(borrower_addr).call().await?.balance;
    let total_supply = erc20.totalSupply().call().await?.totalSupply;

    assert_eq!(FLASH_FEE_AMOUNT, balance);
    assert_eq!(FLASH_FEE_AMOUNT, total_supply);

    let loan_amount = uint!(1_000_000_U256);

    let receipt = receipt!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        loan_amount,
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc20FlashMint::Transfer {
        from: erc20_addr,
        to: borrower_addr,
        value: loan_amount,
    }));

    let balance_of = erc20.balanceOf(borrower_addr).call().await?.balance;
    assert_eq!(balance_of, loan_amount + FLASH_FEE_AMOUNT);

    let total_supply = erc20.totalSupply().call().await?.totalSupply;
    assert_eq!(total_supply, loan_amount + FLASH_FEE_AMOUNT);

    assert!(receipt.emits(Erc20FlashMint::Approval {
        owner: borrower_addr,
        spender: erc20_addr,
        value: loan_amount + FLASH_FEE_AMOUNT,
    }));

    let balance = erc20.balanceOf(borrower_addr).call().await?.balance;
    assert_eq!(U256::ZERO, balance);

    Ok(())
}
