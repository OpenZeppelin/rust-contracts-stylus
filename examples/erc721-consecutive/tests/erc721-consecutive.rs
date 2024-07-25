#![cfg(feature = "e2e")]

use alloy::{
    primitives::{fixed_bytes, Address, Bytes, U256},
    sol,
    sol_types::SolConstructor,
};
use alloy_primitives::uint;
use e2e::{receipt, send, watch, Account, EventExt, Revert};
use erc721_consecutive_example::Params;
use openzeppelin_stylus::token::erc721::extensions::consecutive::Erc721ConsecutiveParams;

use crate::abi::Erc721;

mod abi;

sol!("src/constructor.sol");

fn random_token_id() -> U256 {
    let num: u32 = rand::random();
    U256::from(num)
}

async fn deploy(
    rpc_url: &str,
    private_key: &str,
    receivers: Vec<Address>,
    amounts: Vec<u128>,
) -> eyre::Result<Address> {
    let args = Erc721ConsecutiveExample::constructorCall { receivers, amounts };
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(rpc_url, private_key, Some(args)).await
}

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let alice_addr = alice.address();
    let receivers = vec![alice_addr];
    let amounts = vec![10_u128];
    let contract_addr =
        deploy(alice.url(), &alice.pk(), receivers, amounts).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let balance = contract.balanceOf(alice_addr).call().await?.balance;
    assert_eq!(balance, uint!(10_U256));
    Ok(())
}

#[e2e::test]
async fn mints(alice: Account) -> eyre::Result<()> {
    let batch_size = 10_u128;
    let receivers = vec![alice.address()];
    let amounts = vec![batch_size];
    let contract_addr =
        deploy(alice.url(), &alice.pk(), receivers, amounts).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    // TODO#q: check for ConsecutiveTransfer transfer event after deploy.
    // assert!(receipt.emits(Erc721::ConsecutiveTransfer {
    //     fromTokenId: from_token_id,
    //     toTokenId: to_token_id,
    //     fromAddress: Address::ZERO,
    //     toAddress: alice.address(),
    // }));

    let Erc721::balanceOfReturn { balance: balance1 } =
        contract.balanceOf(alice.address()).call().await?;
    assert_eq!(balance1, U256::from(batch_size));

    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let Erc721::balanceOfReturn { balance: balance2 } =
        contract.balanceOf(alice.address()).call().await?;

    assert_eq!(balance2, balance1 + uint!(1_U256));
    Ok(())
}

// TODO#q: try to check for these errors during construction.
//  Seems koba should be improved and return transaction receipt.

// #[e2e::test]
// async fn error_when_to_is_zero(alice: Account) -> eyre::Result<()> {
//     let contract_addr =
//         deploy(alice.url(), &alice.pk()).await?;
//     let contract = Erc721::new(contract_addr, &alice.wallet);
//
//     let err = send!(contract.init(vec![Address::ZERO], vec![uint!(10_U256)]))
//         .expect_err("should not mint consecutive");
//
//     assert!(err.reverted_with(Erc721::ERC721InvalidReceiver {
//         receiver: Address::ZERO
//     }));
//     Ok(())
// }

// #[e2e::test]
// async fn error_when_exceed_batch_size(alice: Account) -> eyre::Result<()> {
//     let contract_addr =
//         deploy(alice.url(), &alice.pk()).await?;
//     let contract = Erc721::new(contract_addr, &alice.wallet);
//
//     let batch_size = U256::from(Params::MAX_BATCH_SIZE + uint!(1_U96));
//     let err = send!(contract.init(vec![alice.address()], vec![batch_size]))
//         .expect_err("should not mint consecutive");
//
//     assert!(err.reverted_with(Erc721::ERC721ExceededMaxBatchMint {
//         batchSize: U256::from(batch_size),
//         maxBatch: U256::from(Params::MAX_BATCH_SIZE),
//     }));
//     Ok(())
// }

#[e2e::test]
async fn transfers_from(alice: Account, bob: Account) -> eyre::Result<()> {
    let receivers = vec![alice.address(), bob.address()];
    let amounts = vec![1000_u128, 1000_u128];
    // Deploy and mint batches of 1000 tokens to Alice and Bob
    let contract_addr =
        deploy(alice.url(), &alice.pk(), receivers, amounts).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let first_consecutive_token_id = U256::from(Params::FIRST_CONSECUTIVE_ID);

    // Transfer first consecutive token from Alice to Bob
    let _ = watch!(contract.transferFrom(
        alice.address(),
        bob.address(),
        first_consecutive_token_id
    ))?;

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(first_consecutive_token_id).call().await?;
    assert_eq!(ownerOf, bob.address());

    // Check that balances changed
    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice.address()).call().await?;
    assert_eq!(alice_balance, uint!(1000_U256) - uint!(1_U256));
    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob.address()).call().await?;
    assert_eq!(bob_balance, uint!(1000_U256) + uint!(1_U256));

    // Test non-consecutive mint
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice.address(), token_id))?;
    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice.address()).call().await?;
    assert_eq!(alice_balance, uint!(1000_U256));

    // Test transfer of the token that wasn't minted consecutive
    let _ = watch!(contract.transferFrom(
        alice.address(),
        bob.address(),
        token_id
    ))?;
    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice.address()).call().await?;
    assert_eq!(alice_balance, uint!(1000_U256) - uint!(1_U256));
    Ok(())
}

#[e2e::test]
async fn burns(alice: Account) -> eyre::Result<()> {
    let receivers = vec![alice.address()];
    let amounts = vec![1000_u128];
    // Mint batch of 1000 tokens to Alice
    let contract_addr =
        deploy(alice.url(), &alice.pk(), receivers, amounts).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let first_consecutive_token_id = U256::from(Params::FIRST_CONSECUTIVE_ID);

    // Check consecutive token burn
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

    // Check non-consecutive token burn
    let non_consecutive_token_id = random_token_id();
    let _ = watch!(contract.mint(alice.address(), non_consecutive_token_id))?;
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
