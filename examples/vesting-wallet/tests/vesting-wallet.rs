#![cfg(feature = "e2e")]

use alloy::sol;
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

// Epoch timestamp: 1st January 2025 00::00::00
const BLOCK_TIMESTAMP: u64 = 1_735_689_600;
const START: u64 = BLOCK_TIMESTAMP + 3600; // 1 hour
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

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address(), START, DURATION))
        .deploy()
        .await?
        .address()?;
    let contract = VestingWallet::new(contract_addr, &alice.wallet);

    let VestingWallet::startReturn { start } = contract.start().call().await?;
    let VestingWallet::durationReturn { duration } =
        contract.duration().call().await?;
    let VestingWallet::endReturn { end } = contract.end().call().await?;

    assert_eq!(U256::from(START), start);
    assert_eq!(U256::from(DURATION), duration);
    assert_eq!(end, U256::from(START + DURATION));

    Ok(())
}

#[e2e::test]
async fn rejects_zero_address_for_beneficiary(
    alice: Account,
) -> eyre::Result<()> {
    let err = alice
        .as_deployer()
        .with_constructor(ctr(Address::ZERO, START, DURATION))
        .deploy()
        .await
        .expect_err("should not deploy due to `OwnableInvalidOwner`");

    assert!(err.reverted_with(VestingWallet::OwnableInvalidOwner {
        owner: Address::ZERO
    }));

    Ok(())
}

mod erc20_vesting {
    use super::*;

    #[e2e::test]
    async fn check_vesting_schedule(alice: Account) -> eyre::Result<()> {
        let balance = 1000_u64;
        let start = START;
        let duration = DURATION;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(alice.address(), start, duration))
            .deploy()
            .await?
            .address()?;
        let contract = VestingWallet::new(contract_addr, &alice.wallet);

        let erc20_address = erc20::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);
        let _ = watch!(erc20_alice.mint(contract_addr, U256::from(balance)));

        for i in 0..64 {
            let timestamp = i * duration / 60 + start;
            let expected_amount = U256::from(std::cmp::min(
                balance,
                balance * (timestamp - start) / duration,
            ));

            // TODO: update timestamp

            let VestingWallet::vestedAmount_1Return { vestedAmount } = contract
                .vestedAmount_1(erc20_address, timestamp)
                .call()
                .await?;
            assert_eq!(
                expected_amount, vestedAmount,
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
