#![cfg(feature = "e2e")]

use abi::VestingWallet;
use alloy::{eips::BlockId, providers::Provider, sol};
use alloy_primitives::{Address, U256};
use e2e::{receipt, watch, Account, EventExt, ReceiptExt, Revert};
use mock::{erc20, erc20::ERC20Mock};

use crate::VestingWalletExample::constructorCall;

mod abi;
mod mock;

sol!("src/constructor.sol");

const BALANCE: u64 = 1000;
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

/// Since the block timestamp can theoretically change between the initial fetch (to calculate the `start` timestamp)
/// and the final release of vested funds in the test, it is best we assert that the released amount is within
/// some predefined range.
/// The reason why the timestamp can change is that we perform many mutations on-chain, from deploying and activating
/// contracts, sending initial ETH/ERC20 to the contract and then finally releasing the funds.
fn assert_in_delta(expected: U256, actual: U256) {
    let diff = expected.abs_diff(actual);
    let delta = U256::from(1);
    assert!(diff <= delta,"Your result of {actual} should be within {delta} of the expected result {expected}");
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

    async fn run_check_release(
        alice: Account,
        expected_releasable: U256,
        time_passed: u64,
    ) -> eyre::Result<()> {
        let timestamp = block_timestamp(&alice).await?;
        let start = timestamp - time_passed;

        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(alice.address(), start, DURATION))
            .deploy()
            .await?
            .address()?;
        let contract = VestingWallet::new(contract_addr, &alice.wallet);

        let _ = watch!(contract.receiveEther().value(U256::from(BALANCE)))?;

        let old_alice_balance =
            alice.wallet.get_balance(alice.address()).await?;
        let old_contract_balance =
            alice.wallet.get_balance(contract_addr).await?;

        let released = contract.released_0().call().await?.released;
        let releasable = contract.releasable_0().call().await?.releasable;
        assert_eq!(U256::ZERO, released);
        assert_in_delta(expected_releasable, releasable);

        let receipt = receipt!(contract.release_0())?;

        let alice_balance = alice.wallet.get_balance(alice.address()).await?;
        let contract_balance = alice.wallet.get_balance(contract_addr).await?;
        let released = contract.released_0().call().await?.released;
        let releasable = contract.releasable_0().call().await?.releasable;
        assert_in_delta(expected_releasable, released);
        assert_in_delta(U256::ZERO, releasable);
        assert_in_delta(
            old_alice_balance + released
                - U256::from(receipt.gas_used * receipt.effective_gas_price),
            alice_balance,
        );
        assert_in_delta(old_contract_balance - released, contract_balance);

        assert!(
            receipt.emits(VestingWallet::EtherReleased { amount: released })
        );

        Ok(())
    }

    #[e2e::test]
    async fn check_vesting_schedule(alice: Account) -> eyre::Result<()> {
        let start = block_timestamp(&alice).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(alice.address(), start, DURATION))
            .deploy()
            .await?
            .address()?;
        let contract = VestingWallet::new(contract_addr, &alice.wallet);

        let _ = watch!(contract.receiveEther().value(U256::from(BALANCE)))?;

        for i in 0..64 {
            let timestamp = i * DURATION / 60 + start;
            let expected_amount = U256::from(std::cmp::min(
                BALANCE,
                BALANCE * (timestamp - start) / DURATION,
            ));

            let vested_amount =
                contract.vestedAmount_0(timestamp).call().await?.vestedAmount;
            assert_eq!(
                expected_amount, vested_amount,
                "\n---\ni: {i}\nstart: {start}\ntimestamp: {timestamp}\n---\n"
            );
        }

        Ok(())
    }

    #[e2e::test]
    async fn check_release_0_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, U256::ZERO, 0).await
    }

    #[e2e::test]
    async fn check_release_25_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, U256::from(BALANCE / 4), DURATION / 4).await
    }

    #[e2e::test]
    async fn check_release_50_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, U256::from(BALANCE / 2), DURATION / 2).await
    }

    #[e2e::test]
    async fn check_release_100_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, U256::from(BALANCE), DURATION).await
    }

    #[e2e::test]
    async fn check_release_100_percent_vesting_in_the_past(
        alice: Account,
    ) -> eyre::Result<()> {
        run_check_release(alice, U256::from(BALANCE), DURATION * 4 / 3).await
    }
}

mod erc20_vesting {
    use super::*;

    async fn run_check_release(
        alice: Account,
        expected_releasable: U256,
        time_passed: u64,
    ) -> eyre::Result<()> {
        let timestamp = block_timestamp(&alice).await?;
        let start = timestamp - time_passed;

        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(alice.address(), start, DURATION))
            .deploy()
            .await?
            .address()?;
        let contract = VestingWallet::new(contract_addr, &alice.wallet);

        let erc20_address = erc20::deploy(&alice.wallet).await?;
        let erc20 = ERC20Mock::new(erc20_address, &alice.wallet);
        let _ = watch!(erc20.mint(contract_addr, U256::from(BALANCE)))?;

        let old_alice_balance =
            erc20.balanceOf(alice.address()).call().await?.balance;
        let old_contract_balance =
            erc20.balanceOf(contract_addr).call().await?.balance;

        let released =
            contract.released_1(erc20_address).call().await?.released;
        let releasable =
            contract.releasable_1(erc20_address).call().await?.releasable;
        assert_eq!(U256::ZERO, released);
        assert_in_delta(expected_releasable, releasable);

        let receipt = receipt!(contract.release_1(erc20_address))?;

        let alice_balance =
            erc20.balanceOf(alice.address()).call().await?.balance;
        let contract_balance =
            erc20.balanceOf(contract_addr).call().await?.balance;
        let released =
            contract.released_1(erc20_address).call().await?.released;
        let releasable =
            contract.releasable_1(erc20_address).call().await?.releasable;
        assert_in_delta(expected_releasable, released);
        assert_in_delta(U256::ZERO, releasable);
        assert_in_delta(old_alice_balance + released, alice_balance);
        assert_in_delta(old_contract_balance - released, contract_balance);

        assert!(
            receipt.emits(VestingWallet::EtherReleased { amount: released })
        );

        Ok(())
    }

    #[e2e::test]
    async fn check_vesting_schedule(alice: Account) -> eyre::Result<()> {
        let start = block_timestamp(&alice).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(alice.address(), start, DURATION))
            .deploy()
            .await?
            .address()?;
        let contract = VestingWallet::new(contract_addr, &alice.wallet);

        let erc20_address = erc20::deploy(&alice.wallet).await?;
        let erc20 = ERC20Mock::new(erc20_address, &alice.wallet);
        let _ = watch!(erc20.mint(contract_addr, U256::from(BALANCE)))?;

        for i in 0..64 {
            let timestamp = i * DURATION / 60 + start;
            let expected_amount = U256::from(std::cmp::min(
                BALANCE,
                BALANCE * (timestamp - start) / DURATION,
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
        }

        Ok(())
    }

    #[e2e::test]
    async fn check_release_0_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, U256::ZERO, 0).await
    }

    #[e2e::test]
    async fn check_release_25_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, U256::from(BALANCE / 4), DURATION / 4).await
    }

    #[e2e::test]
    async fn check_release_50_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, U256::from(BALANCE / 2), DURATION / 2).await
    }

    #[e2e::test]
    async fn check_release_100_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, U256::from(BALANCE), DURATION).await
    }

    #[e2e::test]
    async fn check_release_100_percent_vesting_in_the_past(
        alice: Account,
    ) -> eyre::Result<()> {
        run_check_release(alice, U256::from(BALANCE), DURATION * 4 / 3).await
    }
}
