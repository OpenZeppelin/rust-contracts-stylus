#![cfg(feature = "e2e")]

use alloy::{
    primitives::{Address, U256},
    sol,
    sol_types::SolConstructor,
};
use e2e::user::User;
use eyre::Result;

use crate::abi::Erc20;

mod abi;

sol!("src/constructor.sol");

async fn deploy(rpc_url: &str, private_key: &str) -> eyre::Result<Address> {
    let name = env!("CARGO_PKG_NAME").replace('-', "_");
    let pkg_dir = env!("CARGO_MANIFEST_DIR");
    let args = Erc20Example::constructorCall {
        name_: "Test Token".to_owned(),
        symbol_: "TTK".to_owned(),
        cap_: U256::from(1),
    };
    let args = alloy::hex::encode(args.abi_encode());
    let contract_addr =
        e2e::deploy::deploy(&name, pkg_dir, rpc_url, private_key, Some(args))
            .await?;

    Ok(contract_addr)
}

#[e2e::test]
async fn mint(alice: User) -> Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc20::new(contract_addr, &alice.signer);

    let Erc20::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice.address()).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await?;

    let one = U256::from(1);
    let _ = contract.mint(alice.address(), one).send().await?.watch().await?;

    let Erc20::balanceOfReturn { balance } =
        contract.balanceOf(alice.address()).call().await?;
    let Erc20::totalSupplyReturn { totalSupply } =
        contract.totalSupply().call().await?;

    assert_eq!(initial_balance + one, balance);
    assert_eq!(initial_supply + one, totalSupply);
    Ok(())
}
