#![cfg(feature = "e2e")]

use alloy::{eips::BlockId, providers::Provider, sol};
use alloy_primitives::{Address, U256};
use e2e::{watch, Account, ReceiptExt, Revert};

use crate::{
    abi::VestingWallet,
    mock::{erc20, erc20::ERC20Mock},
    VestingWalletExample::constructorCall,
};

mod abi;
mod mock;

sol!("src/constructor.sol");

const DURATION: u64 = 365 * 86400; // 1 year

fn ctr(
    beneficiary: Address,
    start_timestamp: u64,
    duration_seconds: u64,
) -> constructorCall {
    constructorCall {
        beneficiary,
        startTimestamp: start_timestamp,
        durationSeconds: duration_seconds,
    }
}

async fn block_timestamp(account: &Account) -> eyre::Result<u64> {
    let timestamp = account
        .wallet
        .get_block(
            BlockId::latest(),
            alloy::rpc::types::BlockTransactionsKind::Full,
        )
        .await?
        .expect("latest block should exist")
        .header
        .timestamp;

    Ok(timestamp)
}

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let start_timestamp = block_timestamp(&alice).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address(), start_timestamp, DURATION))
        .deploy()
        .await?
        .address()?;
    let contract = VestingWallet::new(contract_addr, &alice.wallet);

    let owner = contract.owner().call().await?.owner;
    let start = contract.start().call().await?.start;
    let duration = contract.duration().call().await?.duration;
    let end = contract.end().call().await?.end;

    assert_eq!(alice.address(), owner);
    assert_eq!(U256::from(start_timestamp), start);
    assert_eq!(U256::from(DURATION), duration);
    assert_eq!(U256::from(start_timestamp + DURATION), end);

    Ok(())
}

#[e2e::test]
async fn rejects_zero_address_for_beneficiary(
    alice: Account,
) -> eyre::Result<()> {
    let start = block_timestamp(&alice).await?;
    let err = alice
        .as_deployer()
        .with_constructor(ctr(Address::ZERO, start, DURATION))
        .deploy()
        .await
        .expect_err("should not deploy due to `OwnableInvalidOwner`");

    assert!(err.reverted_with(VestingWallet::OwnableInvalidOwner {
        owner: Address::ZERO
    }));

    Ok(())
}

mod ether_vesting {
    use super::*;

    #[e2e::test]
    async fn check_vesting_schedule(alice: Account) -> eyre::Result<()> {
        let balance = 1000_u64;
        let start = block_timestamp(&alice).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(alice.address(), start, DURATION))
            .deploy()
            .await?
            .address()?;
        let contract = VestingWallet::new(contract_addr, &alice.wallet);

        let _ = watch!(contract.receiveEther().value(U256::from(balance)))?;

        for i in 0..64 {
            let timestamp = i * DURATION / 60 + start;
            let expected_amount = U256::from(std::cmp::min(
                balance,
                balance * (timestamp - start) / DURATION,
            ));

            let vested_amount =
                contract.vestedAmount_0(timestamp).call().await?.vestedAmount;
            assert_eq!(
                expected_amount, vested_amount,
                "\n---\ni: {i}\nstart: {start}\ntimestamp: {timestamp}\n---\n"
            );

            // let releasable =
            // contract.releasable_0().call().await?.releasable;
            // assert_eq!(
            //     expected_amount, releasable,
            //     "\n---\ni: {i}\nstart: {start}\ntimestamp:
            // {timestamp}\n---\n" );
        }

        Ok(())
    }
}

mod erc20_vesting {
    use super::*;

    #[e2e::test]
    async fn check_vesting_schedule(alice: Account) -> eyre::Result<()> {
        let balance = 1000_u64;
        let start = block_timestamp(&alice).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(alice.address(), start, DURATION))
            .deploy()
            .await?
            .address()?;
        let contract = VestingWallet::new(contract_addr, &alice.wallet);

        let erc20_address = erc20::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);
        let _ = watch!(erc20_alice.mint(contract_addr, U256::from(balance)))?;

        for i in 0..64 {
            let timestamp = i * DURATION / 60 + start;
            let expected_amount = U256::from(std::cmp::min(
                balance,
                balance * (timestamp - start) / DURATION,
            ));

            let vested_amount = contract
                .vestedAmount_1(erc20_address, timestamp)
                .call()
                .await?
                .vestedAmount;
            assert_eq!(
                expected_amount, vested_amount,
                "\n---\ni: {i}\nstart: {start}\ntimestamp: {timestamp}\n---\n"
            );

            // TODO: can't assert until block::timestamp can be manipulated
            // let VestingWallet::releasable_1Return { releasable } =
            //     contract.releasable_1(erc20_address).call().await?;
            // assert_eq!(expected_amount, releasable);
        }

        Ok(())
    }
}
