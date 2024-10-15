#![cfg(feature = "e2e")]

use abi::SafeErc20;
use alloy::primitives::U256;
use e2e::{send, Account, ReceiptExt, Revert};
use mock::erc20_return_false;

mod abi;
mod mock;

#[e2e::test]
async fn reverts_on_transfer(alice: Account, bob: Account) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_alice =
        SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();

    let erc20_address = erc20_return_false::deploy(&alice.wallet).await?;

    let err = send!(safe_erc20_alice.safeTransfer(
        erc20_address,
        bob_addr,
        U256::ZERO
    ))
    .expect_err("should not be able to succeed on 'transfer'");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: erc20_address
    }));

    Ok(())
}

#[e2e::test]
async fn reverts_on_transfer_from(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_alice =
        SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let erc20_address = erc20_return_false::deploy(&alice.wallet).await?;

    let err = send!(safe_erc20_alice.safeTransferFrom(
        erc20_address,
        alice_addr,
        bob_addr,
        U256::ZERO
    ))
    .expect_err("should not be able to succeed on 'transferFrom'");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: erc20_address
    }));

    Ok(())
}

#[e2e::test]
async fn reverts_on_increase_allowance(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_alice =
        SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();

    let erc20_address = erc20_return_false::deploy(&alice.wallet).await?;

    let err = send!(safe_erc20_alice.safeIncreaseAllowance(
        erc20_address,
        bob_addr,
        U256::ZERO
    ))
    .expect_err("should not be able to succeed on 'increaseAllowance'");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: erc20_address
    }));

    Ok(())
}

#[e2e::test]
async fn reverts_on_decrease_allowance(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_alice =
        SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();

    let erc20_address = erc20_return_false::deploy(&alice.wallet).await?;

    let err = send!(safe_erc20_alice.safeDecreaseAllowance(
        erc20_address,
        bob_addr,
        U256::ZERO
    ))
    .expect_err("should not be able to succeed on 'decreaseAllowance'");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: erc20_address
    }));

    Ok(())
}

#[e2e::test]
async fn reverts_on_force_approve(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_alice =
        SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();

    let erc20_address = erc20_return_false::deploy(&alice.wallet).await?;

    let err = send!(safe_erc20_alice.forceApprove(
        erc20_address,
        bob_addr,
        U256::ZERO
    ))
    .expect_err("should not be able to succeed on 'forceApprove'");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: erc20_address
    }));

    Ok(())
}
