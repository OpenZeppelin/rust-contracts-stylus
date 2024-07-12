#![cfg(feature = "e2e")]

use alloy::{
    primitives::{Address, U256},
    sol,
    sol_types::SolConstructor,
};
use alloy_primitives::uint;
use e2e::{watch, Account, EventExt, Revert};

use crate::abi::Erc721;

mod abi;

sol!("src/constructor.sol");

fn random_token_id() -> U256 {
    let num: u32 = rand::random();
    U256::from(num)
}

async fn deploy(rpc_url: &str, private_key: &str) -> eyre::Result<Address> {
    let args = Erc721Example::constructorCall {};
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(rpc_url, private_key, Some(args)).await
}

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let receivers = vec![alice_addr];
    let amounts = vec![uint!(10_U256)];
    let _ = watch!(contract.init(receivers, amounts))?;
    let balance = contract.balanceOf(alice_addr).call().await?.balance;
    assert_eq!(balance, uint!(10_U256));
    Ok(())
}

// TODO#q: add erc721 implementation related tests
