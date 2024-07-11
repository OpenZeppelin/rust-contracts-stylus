#![cfg(feature = "e2e")]

use alloy::primitives::{Address, U256};
use alloy_primitives::uint;
use e2e::{Account, EventExt, Revert};

use crate::abi::Erc721;

mod abi;

fn random_token_id() -> U256 {
    let num: u32 = rand::random();
    U256::from(num)
}

async fn deploy(rpc_url: &str, private_key: &str) -> eyre::Result<Address> {
    e2e::deploy(rpc_url, private_key, None).await
}

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let receivers = vec![alice_addr];
    let amounts = vec![uint!(10_U256)];
    let res = contract.init(receivers, amounts).call().await?;
    Ok(())
}

// TODO#q: construct batches
