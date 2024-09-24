#![cfg(feature = "e2e")]

use alloy::primitives::uint;
use alloy_primitives::U256;
use e2e::{receipt, send, watch, Account, ReceiptExt, Revert};

use abi::SafeErc20;
use mock::{erc20, erc20::ERC20Mock};

mod abi;
mod mock;

#[e2e::test]
async fn safe_transfers(alice: Account, bob: Account) -> eyre::Result<()> {
    let safe_erc20_mock_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_mock_alice =
        SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let erc20mock_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(erc20mock_address, &alice.wallet);

    let _ = watch!(erc20_alice.mint(safe_erc20_mock_addr, balance));

    let ERC20Mock::balanceOfReturn { _0: initial_safe_erc20_mock_balance } =
        erc20_alice.balanceOf(safe_erc20_mock_addr).call().await?;
    let ERC20Mock::balanceOfReturn { _0: initial_bob_balance } =
        erc20_alice.balanceOf(bob_addr).call().await?;
    assert_eq!(initial_safe_erc20_mock_balance, balance);
    assert_eq!(initial_bob_balance, U256::ZERO);

    let _ = receipt!(safe_erc20_mock_alice.safeTransfer(
        erc20mock_address,
        bob_addr,
        value
    ))?;

    let ERC20Mock::balanceOfReturn { _0: safe_erc20_mock_balance } =
        erc20_alice.balanceOf(safe_erc20_mock_addr).call().await?;
    let ERC20Mock::balanceOfReturn { _0: bob_balance } =
        erc20_alice.balanceOf(bob_addr).call().await?;

    assert_eq!(
        initial_safe_erc20_mock_balance - value,
        safe_erc20_mock_balance
    );
    assert_eq!(initial_bob_balance + value, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_rejects_with_eoa_as_token(
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
async fn safe_transfer_rejects_on_error(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_mock_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_mock_alice =
        SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
    let bob_addr = bob.address();

    let value = uint!(1_U256);

    let erc20_address = erc20::deploy(&alice.wallet).await?;

    let err = send!(safe_erc20_mock_alice.safeTransfer(
        erc20_address,
        bob_addr,
        value
    ))
    .expect_err("transfer should not succeed when insufficient balance");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: erc20_address
    }));

    Ok(())
}

#[e2e::test]
async fn safe_transfers_from(alice: Account, bob: Account) -> eyre::Result<()> {
    let safe_erc20_mock_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_mock_alice =
        SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let erc20_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

    let _ = watch!(erc20_alice.mint(alice_addr, balance));
    let _ = watch!(erc20_alice.approve(safe_erc20_mock_addr, value));

    let ERC20Mock::balanceOfReturn { _0: initial_alice_balance } =
        erc20_alice.balanceOf(alice_addr).call().await?;
    let ERC20Mock::balanceOfReturn { _0: initial_bob_balance } =
        erc20_alice.balanceOf(bob_addr).call().await?;
    assert_eq!(initial_alice_balance, balance);
    assert_eq!(initial_bob_balance, U256::ZERO);

    let _ = receipt!(safe_erc20_mock_alice.safeTransferFrom(
        erc20_address,
        alice_addr,
        bob_addr,
        value
    ))?;

    let ERC20Mock::balanceOfReturn { _0: alice_balance } =
        erc20_alice.balanceOf(alice_addr).call().await?;
    let ERC20Mock::balanceOfReturn { _0: bob_balance } =
        erc20_alice.balanceOf(bob_addr).call().await?;

    assert_eq!(initial_alice_balance - value, alice_balance);
    assert_eq!(initial_bob_balance + value, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_rejects_with_eoa_as_token(
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
async fn safe_transfer_from_rejects_on_error(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_mock_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_mock_alice =
        SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let erc20_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);
    let _ = watch!(erc20_alice.mint(alice_addr, balance));

    let err = send!(safe_erc20_mock_alice.safeTransferFrom(
        erc20_address,
        alice_addr,
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
