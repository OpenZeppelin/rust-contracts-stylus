#![cfg(feature = "e2e")]

use alloy::{
    primitives::{Address, U256},
    sol,
};
use alloy_primitives::uint;
use e2e::{receipt, watch, Account, EventExt, ReceiptExt, Revert};

use crate::{abi::Erc721, Erc721ConsecutiveExample::constructorCall};

mod abi;

sol!("src/constructor.sol");

const FIRST_CONSECUTIVE_ID: u128 = 0;
const MAX_BATCH_SIZE: u128 = 5000;

fn random_token_id() -> U256 {
    let num: u32 = rand::random();
    U256::from(num)
}

fn ctr(receivers: Vec<Address>, amounts: Vec<u128>) -> constructorCall {
    constructorCall {
        receivers,
        amounts,
        firstConsecutiveId: FIRST_CONSECUTIVE_ID,
        maxBatchSize: MAX_BATCH_SIZE,
    }
}

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let alice_addr = alice.address();
    let receivers = vec![alice_addr];
    let amounts = vec![10_u128];
    let receipt = alice
        .as_deployer()
        .with_constructor(ctr(receivers, amounts))
        .deploy()
        .await?;
    let contract = Erc721::new(receipt.address()?, &alice.wallet);

    let balance = contract.balanceOf(alice_addr).call().await?.balance;
    assert_eq!(balance, uint!(10_U256));
    Ok(())
}

#[e2e::test]
async fn mints(alice: Account) -> eyre::Result<()> {
    let batch_size = 10_u128;
    let receivers = vec![alice.address()];
    let amounts = vec![batch_size];
    let receipt = alice
        .as_deployer()
        .with_constructor(ctr(receivers, amounts))
        .deploy()
        .await?;
    let contract = Erc721::new(receipt.address()?, &alice.wallet);

    assert!(receipt.emits(Erc721::ConsecutiveTransfer {
        fromTokenId: U256::from(FIRST_CONSECUTIVE_ID),
        toTokenId: uint!(9_U256),
        fromAddress: Address::ZERO,
        toAddress: alice.address(),
    }));

    let Erc721::balanceOfReturn { balance: balance1 } =
        contract.balanceOf(alice.address()).call().await?;
    assert_eq!(balance1, U256::from(batch_size));

    let token_id = random_token_id();
    watch!(contract.mint(alice.address(), token_id))?;

    let Erc721::balanceOfReturn { balance: balance2 } =
        contract.balanceOf(alice.address()).call().await?;

    assert_eq!(balance2, balance1 + uint!(1_U256));
    Ok(())
}

#[e2e::test]
async fn error_when_to_is_zero(alice: Account) -> eyre::Result<()> {
    let receivers = vec![Address::ZERO];
    let amounts = vec![10_u128];
    let err = alice
        .as_deployer()
        .with_constructor(ctr(receivers, amounts))
        .deploy()
        .await
        .expect_err("should not mint consecutive");

    assert!(err.reverted_with(Erc721::ERC721InvalidReceiver {
        receiver: Address::ZERO
    }));
    Ok(())
}

#[e2e::test]
async fn error_when_exceed_batch_size(alice: Account) -> eyre::Result<()> {
    let receivers = vec![alice.address()];
    let amounts = vec![MAX_BATCH_SIZE + 1];
    let err = alice
        .as_deployer()
        .with_constructor(ctr(receivers, amounts))
        .deploy()
        .await
        .expect_err("should not mint consecutive");

    assert!(err.reverted_with(Erc721::ERC721ExceededMaxBatchMint {
        batchSize: U256::from(MAX_BATCH_SIZE + 1),
        maxBatch: U256::from(MAX_BATCH_SIZE),
    }));
    Ok(())
}

#[e2e::test]
async fn transfers_from(alice: Account, bob: Account) -> eyre::Result<()> {
    let receivers = vec![alice.address(), bob.address()];
    let amounts = vec![1000_u128, 1000_u128];
    // Deploy and mint batches of 1000 tokens to Alice and Bob.
    let receipt = alice
        .as_deployer()
        .with_constructor(ctr(receivers, amounts))
        .deploy()
        .await?;
    let contract = Erc721::new(receipt.address()?, &alice.wallet);

    let first_consecutive_token_id = U256::from(FIRST_CONSECUTIVE_ID);

    // Transfer first consecutive token from Alice to Bob.
    watch!(contract.transferFrom(
        alice.address(),
        bob.address(),
        first_consecutive_token_id
    ))?;

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(first_consecutive_token_id).call().await?;
    assert_eq!(ownerOf, bob.address());

    // Check that balances changed.
    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice.address()).call().await?;
    assert_eq!(alice_balance, uint!(1000_U256) - uint!(1_U256));
    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob.address()).call().await?;
    assert_eq!(bob_balance, uint!(1000_U256) + uint!(1_U256));

    // Test non-consecutive mint.
    let token_id = random_token_id();
    watch!(contract.mint(alice.address(), token_id))?;
    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice.address()).call().await?;
    assert_eq!(alice_balance, uint!(1000_U256));

    // Test transfer of the token that wasn't minted consecutive.
    watch!(contract.transferFrom(alice.address(), bob.address(), token_id))?;
    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice.address()).call().await?;
    assert_eq!(alice_balance, uint!(1000_U256) - uint!(1_U256));
    Ok(())
}

#[e2e::test]
async fn burns(alice: Account) -> eyre::Result<()> {
    let receivers = vec![alice.address()];
    let amounts = vec![1000_u128];
    // Mint batch of 1000 tokens to Alice.
    let receipt = alice
        .as_deployer()
        .with_constructor(ctr(receivers, amounts))
        .deploy()
        .await?;
    let contract = Erc721::new(receipt.address()?, &alice.wallet);

    let first_consecutive_token_id = U256::from(FIRST_CONSECUTIVE_ID);

    // Check consecutive token burn.
    let receipt = receipt!(contract.burn(first_consecutive_token_id))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice.address(),
        to: Address::ZERO,
        tokenId: first_consecutive_token_id,
    }));

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice.address()).call().await?;
    assert_eq!(alice_balance, uint!(1000_U256) - uint!(1_U256));

    let err = contract
        .ownerOf(first_consecutive_token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(err.reverted_with(Erc721::ERC721NonexistentToken {
        tokenId: first_consecutive_token_id
    }));

    // Check non-consecutive token burn.
    let non_consecutive_token_id = random_token_id();
    watch!(contract.mint(alice.address(), non_consecutive_token_id))?;
    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(non_consecutive_token_id).call().await?;
    assert_eq!(ownerOf, alice.address());
    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice.address()).call().await?;
    assert_eq!(alice_balance, uint!(1000_U256));

    let receipt = receipt!(contract.burn(non_consecutive_token_id))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice.address(),
        to: Address::ZERO,
        tokenId: non_consecutive_token_id,
    }));

    let err = contract
        .ownerOf(non_consecutive_token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(err.reverted_with(Erc721::ERC721NonexistentToken {
        tokenId: non_consecutive_token_id
    }));
    Ok(())
}

// No need to test for `IErc165` impl, as this is already tested in
// ../../examples/erc721 e2e tests
