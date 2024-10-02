#![cfg(feature = "e2e")]

use alloy::primitives::{uint, U256};
use e2e::{send, Account, ReceiptExt, Revert};

use abi::SafeErc20;

mod abi;
mod mock;

#[e2e::test]
async fn reverts_on_transfer(
    alice: Account,
    bob: Account,
    has_no_code: Account,
) -> eyre::Result<()> {
    let safe_erc20_mock_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_mock_alice =
        SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
    let bob_addr = bob.address();
    let has_no_code_addr = has_no_code.address();

    let value = uint!(1_U256);

    let err = send!(safe_erc20_mock_alice.safeTransfer(
        has_no_code_addr,
        bob_addr,
        value
    ))
    .expect_err("should not be able to invoke 'transfer' on EOA");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: has_no_code_addr
    }));

    Ok(())
}

#[e2e::test]
async fn reverts_on_transfer_from(
    alice: Account,
    bob: Account,
    has_no_code: Account,
) -> eyre::Result<()> {
    let safe_erc20_mock_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_mock_alice =
        SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let has_no_code_addr = has_no_code.address();

    let value = uint!(1_U256);

    let err = send!(safe_erc20_mock_alice.safeTransferFrom(
        has_no_code_addr,
        alice_addr,
        bob_addr,
        value
    ))
    .expect_err("should not be able to invoke 'transferFrom' on EOA");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: has_no_code_addr
    }));

    Ok(())
}

#[e2e::test]
async fn reverts_on_increase_allowance(
    alice: Account,
    bob: Account,
    has_no_code: Account,
) -> eyre::Result<()> {
    let safe_erc20_mock_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_mock_alice =
        SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
    let bob_addr = bob.address();
    let has_no_code_addr = has_no_code.address();

    let value = uint!(1_U256);

    let err = send!(safe_erc20_mock_alice.safeIncreaseAllowance(
        has_no_code_addr,
        bob_addr,
        value
    ))
    .expect_err("should not be able to invoke 'increaseAllowance' on EOA");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: has_no_code_addr
    }));

    Ok(())
}

#[e2e::test]
async fn reverts_on_decrease_allowance(
    alice: Account,
    bob: Account,
    has_no_code: Account,
) -> eyre::Result<()> {
    let safe_erc20_mock_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_mock_alice =
        SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
    let bob_addr = bob.address();
    let has_no_code_addr = has_no_code.address();

    let requested_descrease = uint!(1_U256);

    let err = send!(safe_erc20_mock_alice.safeDecreaseAllowance(
        has_no_code_addr,
        bob_addr,
        requested_descrease
    ))
    .expect_err("should not be able to invoke 'decreaseAllowance' on EOA");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: has_no_code_addr
    }));

    Ok(())
}

#[e2e::test]
async fn reverts_on_force_approve(
    alice: Account,
    bob: Account,
    has_no_code: Account,
) -> eyre::Result<()> {
    let safe_erc20_mock_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_mock_alice =
        SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
    let bob_addr = bob.address();
    let has_no_code_addr = has_no_code.address();

    let err = send!(safe_erc20_mock_alice.forceApprove(
        has_no_code_addr,
        bob_addr,
        U256::ZERO
    ))
    .expect_err("should not be able to invoke 'forceApprove' on EOA");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: has_no_code_addr
    }));

    Ok(())
}
