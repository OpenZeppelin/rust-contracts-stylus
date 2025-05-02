#![cfg(feature = "e2e")]

use abi::{Erc20, SafeErc20};
use alloy::primitives::uint;
use alloy_primitives::U256;
use e2e::{
    receipt, send, watch, Account, EventExt, Panic, PanicCode, ReceiptExt,
    Revert,
};
use mock::{erc20, erc20::ERC20Mock};

mod abi;
mod mock;

#[e2e::test]
async fn try_safe_transfer_succeeds(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let erc20_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

    watch!(erc20_alice.mint(safe_erc20_addr, balance))?;

    let initial_safe_erc20_balance =
        erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
    let initial_bob_balance =
        erc20_alice.balanceOf(bob_addr).call().await?._0;
    assert_eq!(initial_safe_erc20_balance, balance);
    assert_eq!(initial_bob_balance, U256::ZERO);

    let success = safe_erc20_alice.trySafeTransfer(
        erc20_address,
        bob_addr,
        value
    ).call().await?._0;
    assert!(success);

    let safe_erc20_balance =
        erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
    let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

    assert_eq!(initial_safe_erc20_balance - value, safe_erc20_balance);
    assert_eq!(initial_bob_balance + value, bob_balance);

    Ok(())
}

#[e2e::test]
async fn try_safe_transfer_fails(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();

    let value = uint!(1_U256);

    let erc20_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

    let initial_safe_erc20_balance =
        erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
    let initial_bob_balance =
        erc20_alice.balanceOf(bob_addr).call().await?._0;

    let success = safe_erc20_alice.trySafeTransfer(
        erc20_address,
        bob_addr,
        value
    ).call().await?._0;
    assert!(!success);

    let safe_erc20_balance =
        erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
    let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

    assert_eq!(initial_safe_erc20_balance, safe_erc20_balance);
    assert_eq!(initial_bob_balance, bob_balance);

    Ok(())
}

#[e2e::test]
async fn try_safe_transfer_from_succeeds(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let erc20_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

    watch!(erc20_alice.mint(alice_addr, balance))?;
    watch!(erc20_alice.approve(safe_erc20_addr, value))?;

    let initial_alice_balance =
        erc20_alice.balanceOf(alice_addr).call().await?._0;
    let initial_bob_balance =
        erc20_alice.balanceOf(bob_addr).call().await?._0;
    assert_eq!(initial_alice_balance, balance);
    assert_eq!(initial_bob_balance, U256::ZERO);

    let success = safe_erc20_alice.trySafeTransferFrom(
        erc20_address,
        alice_addr,
        bob_addr,
        value
    ).call().await?._0;
    assert!(success);

    let alice_balance = erc20_alice.balanceOf(alice_addr).call().await?._0;
    let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

    assert_eq!(initial_alice_balance - value, alice_balance);
    assert_eq!(initial_bob_balance + value, bob_balance);

    Ok(())
}

#[e2e::test]
async fn try_safe_transfer_from_fails(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let value = uint!(1_U256);

    let erc20_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

    watch!(erc20_alice.approve(safe_erc20_addr, value))?;

    let initial_alice_balance =
        erc20_alice.balanceOf(alice_addr).call().await?._0;
    let initial_bob_balance =
        erc20_alice.balanceOf(bob_addr).call().await?._0;

    let success = safe_erc20_alice.trySafeTransferFrom(
        erc20_address,
        alice_addr,
        bob_addr,
        value
    ).call().await?._0;
    assert!(!success);

    let alice_balance = erc20_alice.balanceOf(alice_addr).call().await?._0;
    let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);

    Ok(())
} 