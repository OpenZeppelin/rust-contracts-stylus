#![cfg(feature = "e2e")]

use std::assert_ne;

use alloy::primitives::uint;
use alloy_primitives::U256;
use e2e::{receipt, send, watch, Account, ReceiptExt, Revert};

use abi::SafeErc20;
use mock::{erc20, erc20::ERC20Mock};

mod abi;
mod mock;

#[e2e::test]
async fn safe_transfers(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = SafeErc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let erc20_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

    let _ = watch!(erc20_alice.mint(alice_addr, balance));

    let ERC20Mock::balanceOfReturn { _0: initial_alice_balance } =
        erc20_alice.balanceOf(alice_addr).call().await?;
    let ERC20Mock::balanceOfReturn { _0: initial_bob_balance } =
        erc20_alice.balanceOf(bob_addr).call().await?;
    assert_eq!(initial_alice_balance, balance);
    assert_eq!(initial_bob_balance, U256::ZERO);

    let _ =
        receipt!(contract_alice.safeTransfer(erc20_address, bob_addr, value))?;

    let ERC20Mock::balanceOfReturn { _0: alice_balance } =
        erc20_alice.balanceOf(alice_addr).call().await?;
    let ERC20Mock::balanceOfReturn { _0: bob_balance } =
        erc20_alice.balanceOf(bob_addr).call().await?;

    assert_eq!(initial_alice_balance - value, alice_balance);
    assert_eq!(initial_bob_balance + value, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_rejects_with_eoa_as_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = SafeErc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let value = uint!(1_U256);

    let err = send!(contract_alice.safeTransfer(alice_addr, bob_addr, value))
        .expect_err("should not be able to invoke 'transfer' on EOA");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: alice_addr
    }));
    assert_ne!(alice_addr, bob_addr);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_rejects_on_error(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = SafeErc20::new(contract_addr, &alice.wallet);
    let bob_addr = bob.address();

    let value = uint!(1_U256);

    let erc20_address = erc20::deploy(&alice.wallet).await?;

    let err =
        send!(contract_alice.safeTransfer(erc20_address, bob_addr, value))
            .expect_err(
                "transfer should not succeed when insufficient balance",
            );
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: erc20_address
    }));

    Ok(())
}

#[e2e::test]
async fn safe_transfers_from(
    alice: Account,
    bob: Account,
    charlie: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = SafeErc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let charlie_addr = charlie.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let erc20_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

    let _ = watch!(erc20_alice.mint(charlie_addr, balance));

    let ERC20Mock::balanceOfReturn { _0: initial_charlie_balance } =
        erc20_alice.balanceOf(charlie_addr).call().await?;
    let ERC20Mock::balanceOfReturn { _0: initial_bob_balance } =
        erc20_alice.balanceOf(bob_addr).call().await?;
    assert_eq!(initial_charlie_balance, balance);
    assert_eq!(initial_bob_balance, U256::ZERO);

    let erc20_charlie = ERC20Mock::new(erc20_address, &charlie.wallet);
    let _ = watch!(erc20_charlie.approve(alice_addr, value));

    let _ = receipt!(contract_alice.safeTransferFrom(
        erc20_address,
        charlie_addr,
        bob_addr,
        value
    ))?;

    let ERC20Mock::balanceOfReturn { _0: charlie_balance } =
        erc20_alice.balanceOf(charlie_addr).call().await?;
    let ERC20Mock::balanceOfReturn { _0: bob_balance } =
        erc20_alice.balanceOf(bob_addr).call().await?;

    assert_eq!(initial_charlie_balance - value, charlie_balance);
    assert_eq!(initial_bob_balance + value, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_rejects_with_eoa_as_token(
    alice: Account,
    bob: Account,
    charlie: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = SafeErc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let charlie_addr = charlie.address();

    let value = uint!(1_U256);

    let err = send!(contract_alice.safeTransferFrom(
        alice_addr,
        bob_addr,
        charlie_addr,
        value
    ))
    .expect_err("should not be able to invoke 'transferFrom' on EOA");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: alice_addr
    }));

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_rejects_on_error(
    alice: Account,
    bob: Account,
    charlie: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = SafeErc20::new(contract_addr, &alice.wallet);
    let bob_addr = bob.address();
    let charlie_addr = charlie.address();

    let value = uint!(1_U256);

    let erc20_address = erc20::deploy(&alice.wallet).await?;

    let err = send!(contract_alice.safeTransferFrom(
        erc20_address,
        charlie_addr,
        bob_addr,
        value
    ))
    .expect_err(
        "transferFrom should not succeed when not enough approved balance",
    );
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: erc20_address
    }));

    Ok(())
}
