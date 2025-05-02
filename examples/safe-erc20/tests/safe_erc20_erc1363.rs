#![cfg(feature = "e2e")]

use abi::{Erc20, SafeErc20};
use alloy::primitives::uint;
use alloy_primitives::U256;
use e2e::{
    receipt, send, watch, Account, EventExt, Panic, PanicCode, ReceiptExt,
    Revert,
};
use mock::{erc1363, erc1363::ERC1363Mock};

mod abi;
mod mock;

#[e2e::test]
async fn transfer_and_call_relaxed_works(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);
    let data = vec![1, 2, 3, 4];

    let erc1363_address = erc1363::deploy(&alice.wallet).await?;
    let erc1363_alice = ERC1363Mock::new(erc1363_address, &alice.wallet);

    watch!(erc1363_alice.mint(safe_erc20_addr, balance))?;

    let initial_safe_erc20_balance =
        erc1363_alice.balanceOf(safe_erc20_addr).call().await?._0;
    let initial_bob_balance =
        erc1363_alice.balanceOf(bob_addr).call().await?._0;
    assert_eq!(initial_safe_erc20_balance, balance);
    assert_eq!(initial_bob_balance, U256::ZERO);

    let receipt = receipt!(safe_erc20_alice.transferAndCallRelaxed(
        erc1363_address,
        bob_addr,
        value,
        data.clone()
    ))?;

    assert!(receipt.emits(Erc20::Transfer {
        from: safe_erc20_addr,
        to: bob_addr,
        value
    }));

    let safe_erc20_balance =
        erc1363_alice.balanceOf(safe_erc20_addr).call().await?._0;
    let bob_balance = erc1363_alice.balanceOf(bob_addr).call().await?._0;

    assert_eq!(initial_safe_erc20_balance - value, safe_erc20_balance);
    assert_eq!(initial_bob_balance + value, bob_balance);

    Ok(())
}

#[e2e::test]
async fn transfer_from_and_call_relaxed_works(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);
    let data = vec![1, 2, 3, 4];

    let erc1363_address = erc1363::deploy(&alice.wallet).await?;
    let erc1363_alice = ERC1363Mock::new(erc1363_address, &alice.wallet);

    watch!(erc1363_alice.mint(alice_addr, balance))?;
    watch!(erc1363_alice.approve(safe_erc20_addr, value))?;

    let initial_alice_balance =
        erc1363_alice.balanceOf(alice_addr).call().await?._0;
    let initial_bob_balance =
        erc1363_alice.balanceOf(bob_addr).call().await?._0;
    assert_eq!(initial_alice_balance, balance);
    assert_eq!(initial_bob_balance, U256::ZERO);

    let receipt = receipt!(safe_erc20_alice.transferFromAndCallRelaxed(
        erc1363_address,
        alice_addr,
        bob_addr,
        value,
        data.clone()
    ))?;

    assert!(receipt.emits(Erc20::Transfer {
        from: alice_addr,
        to: bob_addr,
        value
    }));

    let alice_balance = erc1363_alice.balanceOf(alice_addr).call().await?._0;
    let bob_balance = erc1363_alice.balanceOf(bob_addr).call().await?._0;

    assert_eq!(initial_alice_balance - value, alice_balance);
    assert_eq!(initial_bob_balance + value, bob_balance);

    Ok(())
}

#[e2e::test]
async fn approve_and_call_relaxed_works(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
    let bob_addr = bob.address();

    let value = uint!(1_U256);
    let data = vec![1, 2, 3, 4];

    let erc1363_address = erc1363::deploy(&alice.wallet).await?;
    let erc1363_alice = ERC1363Mock::new(erc1363_address, &alice.wallet);

    let receipt = receipt!(safe_erc20_alice.approveAndCallRelaxed(
        erc1363_address,
        bob_addr,
        value,
        data.clone()
    ))?;

    assert!(receipt.emits(Erc20::Approval {
        owner: safe_erc20_addr,
        spender: bob_addr,
        value
    }));

    let bob_allowance =
        erc1363_alice.allowance(safe_erc20_addr, bob_addr).call().await?._0;
    assert_eq!(bob_allowance, value);

    Ok(())
} 