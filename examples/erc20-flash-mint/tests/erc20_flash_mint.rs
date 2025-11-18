#![cfg(feature = "e2e")]

use abi::Erc20FlashMint;
use alloy::primitives::{address, uint, Address, U256};
use e2e::{receipt, send, watch, Account, EventExt, Revert, RustPanic};
use eyre::Result;
use mock::{borrower, borrower::ERC3156FlashBorrowerMock};
use stylus_sdk::alloy_sol_types::SolCall;

mod abi;
mod mock;

const FEE_RECEIVER: Address =
    address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");
const FLASH_FEE_VALUE: U256 = uint!(100_U256);

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
    watch!(contract.setFlashFeeReceiver(FEE_RECEIVER))?;
    watch!(contract.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let max = contract.maxFlashLoan(contract_addr).call().await?.maxLoan;
    let fee = contract.flashFee(contract_addr, U256::ONE).call().await?.fee;

    assert_eq!(max, U256::MAX);
    assert_eq!(fee, FLASH_FEE_VALUE);

    Ok(())
}

#[e2e::test]
async fn max_flash_loan(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
    watch!(contract.setFlashFeeReceiver(FEE_RECEIVER))?;
    watch!(contract.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let alice_addr = alice.address();
    let mint_amount = uint!(1_000_000_U256);
    watch!(contract.mint(alice_addr, mint_amount))?;

    let max_loan = contract.maxFlashLoan(contract_addr).call().await?.maxLoan;
    assert_eq!(U256::MAX - mint_amount, max_loan);

    Ok(())
}

#[e2e::test]
async fn max_flash_loan_return_zero_if_no_more_tokens_to_mint(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
    watch!(contract.setFlashFeeReceiver(FEE_RECEIVER))?;
    watch!(contract.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let alice_addr = alice.address();
    watch!(contract.mint(alice_addr, U256::MAX))?;

    let max_loan = contract.maxFlashLoan(contract_addr).call().await?.maxLoan;
    assert_eq!(U256::MIN, max_loan);

    Ok(())
}

#[e2e::test]
async fn max_flash_loan_returns_zero_on_invalid_address(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
    watch!(contract.setFlashFeeReceiver(FEE_RECEIVER))?;
    watch!(contract.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let alice_addr = alice.address();
    let mint_amount = uint!(1_000_000_U256);
    watch!(contract.mint(alice_addr, mint_amount))?;

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
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
    watch!(contract.setFlashFeeReceiver(FEE_RECEIVER))?;
    watch!(contract.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let amounts = &[U256::ZERO, U256::ONE, uint!(1000_U256), U256::MAX];
    for &amount in amounts {
        let fee = contract.flashFee(contract_addr, amount).call().await?.fee;
        assert_eq!(fee, FLASH_FEE_VALUE);
    }

    Ok(())
}

#[e2e::test]
async fn flash_fee_reverts_on_unsupported_token(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc20FlashMint::new(contract_addr, &alice.wallet);
    watch!(contract.setFlashFeeReceiver(FEE_RECEIVER))?;
    watch!(contract.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let unsupported_token = alice.address();

    let err = contract
        .flashFee(unsupported_token, U256::ONE)
        .call()
        .await
        .expect_err("should return `UnsupportedToken`");

    assert!(err.reverted_with(Erc20FlashMint::ERC3156UnsupportedToken {
        token: unsupported_token
    }));

    let err = contract
        .flashFee(Address::ZERO, U256::ONE)
        .call()
        .await
        .expect_err("should return `UnsupportedToken`");

    assert!(err.reverted_with(Erc20FlashMint::ERC3156UnsupportedToken {
        token: Address::ZERO
    }));

    Ok(())
}

#[e2e::test]
async fn flash_loan_with_fee(alice: Account) -> Result<()> {
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(Address::ZERO))?;
    _ = watch!(erc20.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    watch!(erc20.mint(borrower_addr, FLASH_FEE_VALUE))?;
    let loan_amount = uint!(1_000_000_U256);

    let borrower_balance = erc20.balanceOf(borrower_addr).call().await?.balance;
    let total_supply = erc20.totalSupply().call().await?.totalSupply;

    assert_eq!(FLASH_FEE_VALUE, borrower_balance);
    assert_eq!(FLASH_FEE_VALUE, total_supply);

    let receipt = receipt!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        loan_amount,
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc20FlashMint::Transfer {
        from: Address::ZERO,
        to: borrower_addr,
        value: loan_amount,
    }));
    assert!(receipt.emits(ERC3156FlashBorrowerMock::BalanceOf {
        token: erc20_addr,
        account: borrower_addr,
        value: loan_amount + FLASH_FEE_VALUE,
    }));
    assert!(receipt.emits(ERC3156FlashBorrowerMock::TotalSupply {
        token: erc20_addr,
        value: loan_amount + FLASH_FEE_VALUE,
    }));
    assert!(receipt.emits(Erc20FlashMint::Transfer {
        from: borrower_addr,
        to: Address::ZERO,
        value: loan_amount + FLASH_FEE_VALUE,
    }));

    let borrower_balance = erc20.balanceOf(borrower_addr).call().await?.balance;
    let total_supply = erc20.totalSupply().call().await?.totalSupply;

    assert_eq!(U256::ZERO, borrower_balance);
    assert_eq!(U256::ZERO, total_supply);

    Ok(())
}

#[e2e::test]
async fn flash_loan_with_fee_receiver(alice: Account) -> Result<()> {
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(FEE_RECEIVER))?;
    _ = watch!(erc20.setFlashFeeValue(U256::ZERO))?;

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    let loan_amount = uint!(1_000_000_U256);

    let borrower_balance = erc20.balanceOf(borrower_addr).call().await?.balance;
    let fee_receiver_balance =
        erc20.balanceOf(FEE_RECEIVER).call().await?.balance;
    let total_supply = erc20.totalSupply().call().await?.totalSupply;

    assert_eq!(U256::ZERO, borrower_balance);
    assert_eq!(U256::ZERO, fee_receiver_balance);
    assert_eq!(U256::ZERO, total_supply);

    let receipt = receipt!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        loan_amount,
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc20FlashMint::Transfer {
        from: Address::ZERO,
        to: borrower_addr,
        value: loan_amount,
    }));
    assert!(receipt.emits(ERC3156FlashBorrowerMock::BalanceOf {
        token: erc20_addr,
        account: borrower_addr,
        value: loan_amount,
    }));
    assert!(receipt.emits(ERC3156FlashBorrowerMock::TotalSupply {
        token: erc20_addr,
        value: loan_amount,
    }));
    assert!(receipt.emits(Erc20FlashMint::Transfer {
        from: borrower_addr,
        to: Address::ZERO,
        value: loan_amount,
    }));

    let borrower_balance = erc20.balanceOf(borrower_addr).call().await?.balance;
    let fee_receiver_balance =
        erc20.balanceOf(FEE_RECEIVER).call().await?.balance;
    let total_supply = erc20.totalSupply().call().await?.totalSupply;

    assert_eq!(U256::ZERO, borrower_balance);
    assert_eq!(U256::ZERO, fee_receiver_balance);
    assert_eq!(U256::ZERO, total_supply);

    Ok(())
}

#[e2e::test]
async fn flash_loan_with_fee_and_fee_receiver(alice: Account) -> Result<()> {
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(FEE_RECEIVER))?;
    _ = watch!(erc20.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    watch!(erc20.mint(borrower_addr, FLASH_FEE_VALUE))?;
    let loan_amount = uint!(1_000_000_U256);

    let borrower_balance = erc20.balanceOf(borrower_addr).call().await?.balance;
    let fee_receiver_balance =
        erc20.balanceOf(FEE_RECEIVER).call().await?.balance;
    let total_supply = erc20.totalSupply().call().await?.totalSupply;

    assert_eq!(FLASH_FEE_VALUE, borrower_balance);
    assert_eq!(U256::ZERO, fee_receiver_balance);
    assert_eq!(FLASH_FEE_VALUE, total_supply);

    let receipt = receipt!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        loan_amount,
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc20FlashMint::Transfer {
        from: Address::ZERO,
        to: borrower_addr,
        value: loan_amount,
    }));
    assert!(receipt.emits(ERC3156FlashBorrowerMock::BalanceOf {
        token: erc20_addr,
        account: borrower_addr,
        value: loan_amount + FLASH_FEE_VALUE,
    }));
    assert!(receipt.emits(ERC3156FlashBorrowerMock::TotalSupply {
        token: erc20_addr,
        value: loan_amount + FLASH_FEE_VALUE,
    }));
    assert!(receipt.emits(Erc20FlashMint::Transfer {
        from: borrower_addr,
        to: Address::ZERO,
        value: loan_amount,
    }));
    assert!(receipt.emits(Erc20FlashMint::Transfer {
        from: borrower_addr,
        to: FEE_RECEIVER,
        value: FLASH_FEE_VALUE,
    }));

    let borrower_balance = erc20.balanceOf(borrower_addr).call().await?.balance;
    let fee_receiver_balance =
        erc20.balanceOf(FEE_RECEIVER).call().await?.balance;
    let total_supply = erc20.totalSupply().call().await?.totalSupply;

    assert_eq!(U256::ZERO, borrower_balance);
    assert_eq!(FLASH_FEE_VALUE, fee_receiver_balance);
    assert_eq!(FLASH_FEE_VALUE, total_supply);

    Ok(())
}

#[e2e::test]
async fn flash_loan_reverts_when_loan_amount_greater_than_max_loan(
    alice: Account,
) -> Result<()> {
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(FEE_RECEIVER))?;
    _ = watch!(erc20.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    let max_loan = U256::ONE;
    let loan_amount = uint!(2_U256);

    watch!(erc20.mint(borrower_addr, U256::MAX - max_loan))?;

    let err = send!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        loan_amount,
        vec![].into(),
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
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(FEE_RECEIVER))?;
    _ = watch!(erc20.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    let invalid_token = alice.address();
    let loan_amount = U256::ONE;

    let err = send!(erc20.flashLoan(
        borrower_addr,
        invalid_token,
        loan_amount,
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
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(FEE_RECEIVER))?;
    _ = watch!(erc20.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    let invalid_token = alice.address();
    let loan_amount = U256::ZERO;

    let err = send!(erc20.flashLoan(
        borrower_addr,
        invalid_token,
        loan_amount,
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
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(FEE_RECEIVER))?;
    _ = watch!(erc20.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    watch!(erc20.mint(borrower_addr, FLASH_FEE_VALUE))?;
    let loan_amount = U256::ONE;

    let invalid_receivers = &[alice.address(), Address::ZERO];

    for &invalid_receiver in invalid_receivers {
        let err = send!(erc20.flashLoan(
            invalid_receiver,
            erc20_addr,
            loan_amount,
            vec![].into()
        ))
        .expect_err("should revert with `ERC3156InvalidReceiver`");

        assert!(err.reverted_with(Erc20FlashMint::ERC3156InvalidReceiver {
            receiver: invalid_receiver
        }),);
    }

    Ok(())
}

#[e2e::test]
async fn flash_loan_reverts_when_receiver_callback_reverts(
    alice: Account,
) -> Result<()> {
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(FEE_RECEIVER))?;
    _ = watch!(erc20.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    watch!(erc20.mint(borrower_addr, FLASH_FEE_VALUE))?;
    let loan_amount = U256::ONE;

    let err = send!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        loan_amount,
        vec![1, 2].into()
    ))
    .expect_err("should revert with `ERC3156InvalidReceiver`");

    assert!(err.reverted_with(Erc20FlashMint::ERC3156InvalidReceiver {
        receiver: borrower_addr
    }),);

    Ok(())
}

#[e2e::test]
async fn flash_loan_reverts_when_receiver_returns_invalid_callback_value(
    alice: Account,
) -> Result<()> {
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(FEE_RECEIVER))?;
    _ = watch!(erc20.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let borrower_addr = borrower::deploy(&alice.wallet, false, true).await?;
    watch!(erc20.mint(borrower_addr, FLASH_FEE_VALUE))?;
    let loan_amount = U256::ONE;

    let err = send!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        loan_amount,
        vec![].into()
    ))
    .expect_err("should revert with `ERC3156InvalidReceiver`");

    assert!(err.reverted_with(Erc20FlashMint::ERC3156InvalidReceiver {
        receiver: borrower_addr
    }),);

    Ok(())
}

#[e2e::test]
async fn flash_loan_reverts_when_receiver_doesnt_approve_allowance(
    alice: Account,
) -> Result<()> {
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(FEE_RECEIVER))?;
    _ = watch!(erc20.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let borrower_addr = borrower::deploy(&alice.wallet, true, false).await?;
    watch!(erc20.mint(borrower_addr, FLASH_FEE_VALUE))?;
    let loan_amount = U256::ONE;

    let err = send!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        loan_amount,
        vec![].into()
    ))
    .expect_err("should revert with `ERC20InsufficientAllowance`");

    assert!(err.reverted_with(Erc20FlashMint::ERC20InsufficientAllowance {
        spender: erc20_addr,
        allowance: U256::ZERO,
        needed: loan_amount + FLASH_FEE_VALUE
    }),);

    Ok(())
}

#[e2e::test]
async fn flash_loan_reverts_when_allowance_overflows(
    alice: Account,
) -> Result<()> {
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(FEE_RECEIVER))?;
    _ = watch!(erc20.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let borrower_addr = borrower::deploy(&alice.wallet, true, false).await?;
    let loan_amount = U256::MAX;

    let err = send!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        loan_amount,
        vec![].into()
    ))
    .expect_err("should panic due to allowance overflow");

    assert!(err.panicked());

    Ok(())
}

#[e2e::test]
async fn flash_loan_reverts_when_receiver_doesnt_have_enough_tokens(
    alice: Account,
) -> Result<()> {
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(FEE_RECEIVER))?;
    _ = watch!(erc20.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    let loan_amount = U256::ONE;

    // test when not enough to cover fees
    let err = send!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        loan_amount,
        vec![].into(),
    ))
    .expect_err("should revert with `ERC20InsufficientBalance`");

    assert!(err.reverted_with(Erc20FlashMint::ERC20InsufficientBalance {
        sender: borrower_addr,
        balance: U256::ZERO,
        needed: FLASH_FEE_VALUE
    }));

    // test when not enough to return the loaned tokens
    let call = Erc20FlashMint::transferCall {
        recipient: alice.address(),
        amount: loan_amount,
    };

    let err = send!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        loan_amount,
        call.abi_encode().into(),
    ))
    .expect_err("should revert with `ERC20InsufficientBalance`");

    assert!(err.reverted_with(Erc20FlashMint::ERC20InsufficientBalance {
        sender: borrower_addr,
        balance: U256::ZERO,
        needed: loan_amount,
    }));

    Ok(())
}

#[e2e::test]
async fn flash_loan_reverts_when_receiver_doesnt_have_enough_tokens_and_fee_is_zero(
    alice: Account,
) -> Result<()> {
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(FEE_RECEIVER))?;
    _ = watch!(erc20.setFlashFeeValue(U256::ZERO))?;

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    let loan_amount = U256::ONE;

    let call = Erc20FlashMint::transferCall {
        recipient: alice.address(),
        amount: loan_amount,
    };

    let err = send!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        loan_amount,
        call.abi_encode().into(),
    ))
    .expect_err("should revert with `ERC20InsufficientBalance`");

    assert!(err.reverted_with(Erc20FlashMint::ERC20InsufficientBalance {
        sender: borrower_addr,
        balance: U256::ZERO,
        needed: loan_amount,
    }));

    Ok(())
}

#[e2e::test]
async fn flash_loan_reverts_when_receiver_doesnt_have_enough_tokens_and_fee_receiver_is_zero(
    alice: Account,
) -> Result<()> {
    let erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let erc20 = Erc20FlashMint::new(erc20_addr, &alice.wallet);
    _ = watch!(erc20.setFlashFeeReceiver(Address::ZERO))?;
    _ = watch!(erc20.setFlashFeeValue(FLASH_FEE_VALUE))?;

    let borrower_addr = borrower::deploy(&alice.wallet, true, true).await?;
    let loan_amount = U256::ONE;

    let err = send!(erc20.flashLoan(
        borrower_addr,
        erc20_addr,
        loan_amount,
        vec![].into(),
    ))
    .expect_err("should revert with `ERC20InsufficientBalance`");

    assert!(err.reverted_with(Erc20FlashMint::ERC20InsufficientBalance {
        sender: borrower_addr,
        balance: loan_amount,
        needed: loan_amount + FLASH_FEE_VALUE
    }));

    Ok(())
}
