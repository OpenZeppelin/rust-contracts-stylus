#![cfg(feature = "e2e")]

use abi::Erc721HolderExample;
use alloy::primitives::U256;
use e2e::{constructor, Account};
use eyre::Result;
use stylus_sdk::function_selector;

mod abi;

const RECEIVER_FN_SELECTOR: [u8; 4] = function_selector!(
    "onERC721Received",
    alloy_primitives::Address,
    alloy_primitives::Address,
    U256,
    stylus_sdk::abi::Bytes,
);

#[e2e::test]
async fn deploys_successfully(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!())
        .with_example_name("erc721-holder")
        .deploy()
        .await?
        .contract_address;

    let contract = Erc721HolderExample::new(contract_addr, &alice.wallet);

    // Test that the contract can be called and returns the correct selector
    let result = contract
        .onERC721Received(
            alice.address(),
            alice.address(),
            U256::from(1),
            alloy_primitives::Bytes::new(),
        )
        .call()
        .await?;

    assert_eq!(result._0, RECEIVER_FN_SELECTOR);

    Ok(())
}

#[e2e::test]
async fn returns_correct_selector_with_data(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!())
        .with_example_name("erc721-holder")
        .deploy()
        .await?
        .contract_address;

    let contract = Erc721HolderExample::new(contract_addr, &alice.wallet);

    // Test with some data
    let test_data = alloy_primitives::Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]);

    let result = contract
        .onERC721Received(
            alice.address(),
            alice.address(),
            U256::from(42),
            test_data,
        )
        .call()
        .await?;

    assert_eq!(result._0, RECEIVER_FN_SELECTOR);

    Ok(())
}
