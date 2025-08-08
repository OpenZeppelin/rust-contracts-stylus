#![cfg(feature = "e2e")]

use abi::SafeErc20;
use alloy::primitives::{uint, U256};
use e2e::{receipt, send, Account, EventExt, Revert};

mod abi;
mod mock;

#[e2e::test]
async fn safe_transfer_reverts_when_eoa_token(
    alice: Account,
    bob: Account,
    has_no_code: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();
    let has_no_code_addr = has_no_code.address();

    let value = uint!(1_U256);

    let err =
        send!(safe_erc20_alice.safeTransfer(has_no_code_addr, bob_addr, value))
            .expect_err("should not be able to invoke 'transfer' on EOA");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: has_no_code_addr
    }));

    Ok(())
}

#[e2e::test]
async fn returns_false_on_try_safe_transfer(
    alice: Account,
    bob: Account,
    has_no_code: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();
    let has_no_code_addr = has_no_code.address();

    let value = uint!(0_U256);

    let receipt = receipt!(safe_erc20_alice.trySafeTransfer(
        has_no_code_addr,
        bob_addr,
        value
    ))?;

    assert!(receipt.emits(SafeErc20::False {}));

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_reverts_when_eoa_token(
    alice: Account,
    bob: Account,
    has_no_code: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let has_no_code_addr = has_no_code.address();

    let value = uint!(1_U256);

    let err = send!(safe_erc20_alice.safeTransferFrom(
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
async fn returns_false_on_try_safe_transfer_from(
    alice: Account,
    bob: Account,
    has_no_code: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let has_no_code_addr = has_no_code.address();

    let value = uint!(0_U256);

    let receipt = receipt!(safe_erc20_alice.trySafeTransferFrom(
        has_no_code_addr,
        alice_addr,
        bob_addr,
        value
    ))?;

    assert!(receipt.emits(SafeErc20::False {}));

    Ok(())
}

#[e2e::test]
async fn safe_increase_allowance_reverts_when_eoa_token(
    alice: Account,
    bob: Account,
    has_no_code: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();
    let has_no_code_addr = has_no_code.address();

    let value = uint!(1_U256);

    let err = send!(safe_erc20_alice.safeIncreaseAllowance(
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
async fn safe_decrease_allowance_reverts_when_eoa_token(
    alice: Account,
    bob: Account,
    has_no_code: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();
    let has_no_code_addr = has_no_code.address();

    let requested_decrease = uint!(1_U256);

    let err = send!(safe_erc20_alice.safeDecreaseAllowance(
        has_no_code_addr,
        bob_addr,
        requested_decrease
    ))
    .expect_err("should not be able to invoke 'decreaseAllowance' on EOA");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedDecreaseAllowance {
        spender: bob_addr,
        currentAllowance: U256::ZERO,
        requestedDecrease: requested_decrease,
    }));

    Ok(())
}

#[e2e::test]
async fn force_approve_reverts_when_eoa_token(
    alice: Account,
    bob: Account,
    has_no_code: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();
    let has_no_code_addr = has_no_code.address();

    let err = send!(safe_erc20_alice.forceApprove(
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
