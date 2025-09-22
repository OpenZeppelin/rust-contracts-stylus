#![cfg(feature = "e2e")]

use abi::SafeErc20;
use alloy::primitives::{uint, U256};
use e2e::{receipt, send, watch, Account, EventExt, Revert};
use mock::{erc20_return_false, erc20_return_false::ERC20ReturnFalseMock};

mod abi;
mod mock;

#[e2e::test]
async fn reverts_on_transfer(alice: Account, bob: Account) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
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
async fn returns_false_on_try_safe_transfer(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();

    let balance = uint!(10_U256);

    let erc20_address = erc20_return_false::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20ReturnFalseMock::new(erc20_address, &alice.wallet);

    watch!(erc20_alice.mint(safe_erc20_addr, balance))?;

    let safe_erc20_balance =
        erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
    let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

    assert_eq!(safe_erc20_balance, balance);
    assert_eq!(bob_balance, U256::ZERO);

    let receipt = receipt!(safe_erc20_alice.trySafeTransfer(
        erc20_address,
        bob_addr,
        balance
    ))?;

    assert!(receipt.emits(SafeErc20::False {}));

    let safe_erc20_balance =
        erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
    let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

    assert_eq!(safe_erc20_balance, balance);
    assert_eq!(bob_balance, U256::ZERO);

    Ok(())
}

#[e2e::test]
async fn reverts_on_transfer_from(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
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
async fn returns_false_on_try_safe_transfer_from(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);

    let erc20_address = erc20_return_false::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20ReturnFalseMock::new(erc20_address, &alice.wallet);

    watch!(erc20_alice.mint(alice_addr, balance))?;
    watch!(erc20_alice.approve(safe_erc20_addr, balance))?;

    let alice_balance = erc20_alice.balanceOf(alice_addr).call().await?._0;
    let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;
    assert_eq!(alice_balance, balance);
    assert_eq!(bob_balance, U256::ZERO);

    let receipt = receipt!(safe_erc20_alice.trySafeTransferFrom(
        erc20_address,
        alice_addr,
        bob_addr,
        balance
    ))?;

    assert!(receipt.emits(SafeErc20::False {}));

    let alice_balance = erc20_alice.balanceOf(alice_addr).call().await?._0;
    let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;
    assert_eq!(alice_balance, balance);
    assert_eq!(bob_balance, U256::ZERO);

    Ok(())
}

#[e2e::test]
async fn reverts_on_increase_allowance(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
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
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
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
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
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
