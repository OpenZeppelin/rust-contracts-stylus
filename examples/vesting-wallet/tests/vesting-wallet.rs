#![cfg(feature = "e2e")]

use abi::VestingWallet;
use alloy::{
    eips::BlockId,
    network::TransactionBuilder,
    primitives::{Address, U256},
    providers::Provider,
    rpc::types::{BlockTransactionsKind, TransactionRequest},
};
use e2e::{
    constructor, receipt, send, watch, Account, Constructor,
    ContractInitializationError, EventExt, Revert, RustPanic,
};
use mock::{erc20, erc20::ERC20Mock};

mod abi;
mod mock;

const BALANCE: u64 = 1000;
const DURATION: u64 = 365 * 86400; // 1 year

fn ctr(
    beneficiary: Address,
    start_timestamp: u64,
    duration_seconds: u64,
) -> Constructor {
    constructor!(beneficiary, start_timestamp, duration_seconds)
}

async fn block_timestamp(account: &Account) -> eyre::Result<u64> {
    let timestamp = account
        .wallet
        .get_block(BlockId::latest(), BlockTransactionsKind::Hashes)
        .await?
        .expect("latest block should exist")
        .header
        .timestamp;

    Ok(timestamp)
}

/// Since the block timestamp can theoretically change between the initial fetch
/// (to calculate the `start` timestamp) and the final release of vested funds
/// in the test, it is best we assert that the released amount is within
/// some predefined range.
/// The reason why the timestamp can change is that we perform many mutations
/// on-chain, from deploying and activating contracts, sending initial ETH/ERC20
/// to the contract and then finally releasing the funds.
fn assert_in_delta(expected: U256, actual: U256) {
    let diff = expected.abs_diff(actual);
    let delta = U256::ONE;
    assert!(diff <= delta, "Your result of {actual} should be within {delta} of the expected result {expected}");
}

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let start_timestamp = block_timestamp(&alice).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address(), start_timestamp, DURATION))
        .deploy()
        .await?
        .contract_address;
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

    // TODO: assert the actual `OwnableInvalidOwner` error was returned once
    // `StylusDeployer` is able to return the exact revert reason from
    // constructors. assert!(err.
    // reverted_with(VestingWallet::OwnableInvalidOwner {     owner:
    // Address::ZERO }));

    assert!(err.downcast_ref::<ContractInitializationError>().is_some());

    Ok(())
}

mod ether_vesting {
    use super::*;

    async fn deploy(
        account: &Account,
        start: u64,
        duration: u64,
        allocation: u64,
    ) -> eyre::Result<Address> {
        let contract_addr = account
            .as_deployer()
            .with_constructor(ctr(account.address(), start, duration))
            .deploy()
            .await?
            .contract_address;

        let tx = TransactionRequest::default()
            .with_from(account.address())
            .with_to(contract_addr)
            .with_value(U256::from(allocation));

        account.wallet.send_transaction(tx).await?.watch().await?;

        Ok(contract_addr)
    }

    async fn run_check_release(
        alice: Account,
        time_passed: u64,
    ) -> eyre::Result<()> {
        let timestamp = block_timestamp(&alice).await?;
        let start = timestamp - time_passed;
        let expected_amount = U256::from(std::cmp::min(
            BALANCE,
            BALANCE * time_passed / DURATION,
        ));
        let contract_addr = deploy(&alice, start, DURATION, BALANCE).await?;
        let contract = VestingWallet::new(contract_addr, &alice.wallet);

        let old_alice_balance =
            alice.wallet.get_balance(alice.address()).await?;
        let old_contract_balance =
            alice.wallet.get_balance(contract_addr).await?;

        let released = contract.released_0().call().await?.released;
        let releasable = contract.releasable_0().call().await?.releasable;
        assert_eq!(U256::ZERO, released);
        assert_in_delta(expected_amount, releasable);

        let receipt = receipt!(contract.release_0())?;

        let alice_balance = alice.wallet.get_balance(alice.address()).await?;
        let contract_balance = alice.wallet.get_balance(contract_addr).await?;
        let released = contract.released_0().call().await?.released;
        let releasable = contract.releasable_0().call().await?.releasable;
        assert_in_delta(expected_amount, released);
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
    async fn check_release_0_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, 0).await
    }

    #[e2e::test]
    async fn check_release_25_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, DURATION / 4).await
    }

    #[e2e::test]
    async fn check_release_50_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, DURATION / 2).await
    }

    #[e2e::test]
    async fn check_release_100_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, DURATION).await
    }

    #[e2e::test]
    async fn check_release_100_percent_vesting_in_the_past(
        alice: Account,
    ) -> eyre::Result<()> {
        run_check_release(alice, DURATION * 4 / 3).await
    }

    #[e2e::test]
    async fn check_vested_amount(alice: Account) -> eyre::Result<()> {
        let start = block_timestamp(&alice).await?;
        let contract_addr = deploy(&alice, start, DURATION, BALANCE).await?;

        let contract = VestingWallet::new(contract_addr, &alice.wallet);

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
}

mod erc20_vesting {
    use super::*;

    async fn deploy(
        account: &Account,
        start: u64,
        duration: u64,
    ) -> eyre::Result<Address> {
        let contract_addr = account
            .as_deployer()
            .with_constructor(ctr(account.address(), start, duration))
            .deploy()
            .await?
            .contract_address;
        Ok(contract_addr)
    }

    async fn deploy_erc20(
        account: &Account,
        mint_to: Address,
        allocation: U256,
    ) -> eyre::Result<Address> {
        let erc20_address = erc20::deploy(&account.wallet).await?;
        let erc20 = ERC20Mock::new(erc20_address, &account.wallet);
        watch!(erc20.mint(mint_to, allocation))?;
        Ok(erc20_address)
    }

    async fn deploy_erc20_return_false(
        account: &Account,
        mint_to: Address,
        allocation: u64,
    ) -> eyre::Result<Address> {
        use mock::{
            erc20_return_false, erc20_return_false::ERC20ReturnFalseMock,
        };

        let erc20_address = erc20_return_false::deploy(&account.wallet).await?;
        let erc20 = ERC20ReturnFalseMock::new(erc20_address, &account.wallet);
        watch!(erc20.mint(mint_to, U256::from(allocation)))?;
        Ok(erc20_address)
    }

    async fn run_check_release(
        alice: Account,
        time_passed: u64,
    ) -> eyre::Result<()> {
        let timestamp = block_timestamp(&alice).await?;
        let start = timestamp - time_passed;
        let expected_amount = U256::from(std::cmp::min(
            BALANCE,
            BALANCE * time_passed / DURATION,
        ));
        let contract_addr = deploy(&alice, start, DURATION).await?;
        let erc20_address =
            deploy_erc20(&alice, contract_addr, U256::from(BALANCE)).await?;

        let contract = VestingWallet::new(contract_addr, &alice.wallet);
        let erc20 = ERC20Mock::new(erc20_address, &alice.wallet);

        let old_alice_balance =
            erc20.balanceOf(alice.address()).call().await?.balance;
        let old_contract_balance =
            erc20.balanceOf(contract_addr).call().await?.balance;

        let released =
            contract.released_1(erc20_address).call().await?.released;
        let releasable =
            contract.releasable_1(erc20_address).call().await?.releasable;
        assert_eq!(U256::ZERO, released);
        assert_in_delta(expected_amount, releasable);

        let receipt = receipt!(contract.release_1(erc20_address))?;

        let alice_balance =
            erc20.balanceOf(alice.address()).call().await?.balance;
        let contract_balance =
            erc20.balanceOf(contract_addr).call().await?.balance;
        let released =
            contract.released_1(erc20_address).call().await?.released;
        let releasable =
            contract.releasable_1(erc20_address).call().await?.releasable;
        assert_in_delta(expected_amount, released);
        assert_in_delta(U256::ZERO, releasable);
        assert_in_delta(old_alice_balance + released, alice_balance);
        assert_in_delta(old_contract_balance - released, contract_balance);

        assert!(receipt.emits(VestingWallet::ERC20Released {
            token: erc20_address,
            amount: released
        }));

        Ok(())
    }

    #[e2e::test]
    async fn check_release_0_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, 0).await
    }

    #[e2e::test]
    async fn check_release_25_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, DURATION / 4).await
    }

    #[e2e::test]
    async fn check_release_50_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, DURATION / 2).await
    }

    #[e2e::test]
    async fn check_release_100_percent(alice: Account) -> eyre::Result<()> {
        run_check_release(alice, DURATION).await
    }

    #[e2e::test]
    async fn check_release_100_percent_vesting_in_the_past(
        alice: Account,
    ) -> eyre::Result<()> {
        run_check_release(alice, DURATION * 4 / 3).await
    }

    #[e2e::test]
    async fn check_vested_amount(alice: Account) -> eyre::Result<()> {
        let start = block_timestamp(&alice).await?;
        let contract_addr = deploy(&alice, start, DURATION).await?;
        let erc20_address =
            deploy_erc20(&alice, contract_addr, U256::from(BALANCE)).await?;

        let contract = VestingWallet::new(contract_addr, &alice.wallet);

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
    async fn releasable_erc20_reverts_on_invalid_token(
        alice: Account,
    ) -> eyre::Result<()> {
        let start = block_timestamp(&alice).await?;
        let contract_addr = deploy(&alice, start, DURATION).await?;

        let contract = VestingWallet::new(contract_addr, &alice.wallet);

        let err = send!(contract.releasable_1(Address::ZERO))
            .expect_err("should not get releasable amount for invalid token");

        assert!(err.reverted_with(VestingWallet::InvalidToken {
            token: Address::ZERO
        }));

        Ok(())
    }

    #[e2e::test]
    async fn release_erc20_reverts_on_invalid_token(
        alice: Account,
    ) -> eyre::Result<()> {
        let start = block_timestamp(&alice).await?;
        let contract_addr = deploy(&alice, start, DURATION).await?;

        let contract = VestingWallet::new(contract_addr, &alice.wallet);

        let err = send!(contract.release_1(Address::ZERO))
            .expect_err("should not release for invalid token");

        assert!(err.reverted_with(VestingWallet::InvalidToken {
            token: Address::ZERO
        }));

        Ok(())
    }

    #[e2e::test]
    async fn release_erc20_reverts_on_failed_transfer(
        alice: Account,
    ) -> eyre::Result<()> {
        let start = block_timestamp(&alice).await?;
        let contract_addr = deploy(&alice, start, DURATION).await?;
        let erc20_address =
            deploy_erc20_return_false(&alice, contract_addr, BALANCE).await?;

        let contract = VestingWallet::new(contract_addr, &alice.wallet);

        let err = send!(contract.release_1(erc20_address))
            .expect_err("should not release when transfer fails");

        assert!(err.reverted_with(VestingWallet::SafeErc20FailedOperation {
            token: erc20_address
        }));

        Ok(())
    }

    #[e2e::test]
    async fn vested_amount_erc20_reverts_on_invalid_token(
        alice: Account,
    ) -> eyre::Result<()> {
        let start = block_timestamp(&alice).await?;
        let contract_addr = deploy(&alice, start, DURATION).await?;

        let contract = VestingWallet::new(contract_addr, &alice.wallet);

        let err = send!(contract.vestedAmount_1(Address::ZERO, start))
            .expect_err("should not get vested amount for invalid token");

        assert!(err.reverted_with(VestingWallet::InvalidToken {
            token: Address::ZERO
        }));

        Ok(())
    }

    #[e2e::test]
    async fn vested_amount_reverts_on_scaled_allocation_overflow(
        alice: Account,
    ) -> eyre::Result<()> {
        let start = block_timestamp(&alice).await?;
        let timestamp = DURATION / 2 + start;
        let contract_addr = deploy(&alice, start, DURATION).await?;
        let erc20_address =
            deploy_erc20(&alice, contract_addr, U256::MAX).await?;

        let contract = VestingWallet::new(contract_addr, &alice.wallet);

        let err = send!(contract.vestedAmount_1(erc20_address, timestamp))
            .expect_err("should exceed `U256::MAX`");

        assert!(err.panicked());

        Ok(())
    }
}
