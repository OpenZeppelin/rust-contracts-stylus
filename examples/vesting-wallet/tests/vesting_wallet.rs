#![cfg(feature = "e2e")]

use alloy::sol;
use alloy_primitives::{Address, U256};
use e2e::{Account, ReceiptExt};

use crate::{abi::VestingWallet, VestingWalletExample::constructorCall};

mod abi;

sol!("src/constructor.sol");

const HOUR: u64 = 3600;
const YEAR: u64 = 365 * 86400;

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
    let start = HOUR;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address(), start, YEAR))
        .deploy()
        .await?
        .address()?;
    let contract = VestingWallet::new(contract_addr, &alice.wallet);

    let VestingWallet::startReturn { start: _start } =
        contract.start().call().await?;
    let VestingWallet::durationReturn { duration } =
        contract.duration().call().await?;
    let VestingWallet::endReturn { end } = contract.end().call().await?;

    assert_eq!(U256::from(start), _start);
    assert_eq!(U256::from(YEAR), duration);
    assert_eq!(end, U256::from(start + YEAR));

    Ok(())
}