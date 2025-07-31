#![cfg(feature = "e2e")]

use abi::{Erc1967Example, UUPSProxyErc20Example};
use alloy::{
    primitives::{Address, U256},
    sol_types::SolCall,
};
use e2e::{
    constructor, receipt, send, watch, Account, Constructor, EventExt, Revert,
};
use eyre::Result;
use stylus_sdk::abi::Bytes;

mod abi;

fn ctr(implementation: Address) -> Constructor {
    constructor!(implementation)
}

fn erc1967_ctr(implementation: Address, data: Bytes) -> Constructor {
    constructor!(implementation, data.clone())
}

#[e2e::test]
async fn constructs(alice: Account, deployer: Account) -> Result<()> {
    let logic_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;

    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967")
        .with_constructor(erc1967_ctr(logic_addr, vec![].into()))
        .deploy()
        .await?
        .contract_address;

    let proxy_contract = Erc1967Example::new(proxy_addr, &alice.wallet);

    assert_eq!(
        logic_addr,
        proxy_contract.implementation().call().await?.implementation
    );

    assert_eq!(
        U256::ZERO,
        proxy_contract.totalSupply().call().await?.totalSupply
    );

    Ok(())
}

#[e2e::test]
async fn constructs_with_data(alice: Account, deployer: Account) -> Result<()> {
    let alice_addr = alice.address();
    let logic_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    // mint 1000 tokens.
    let amount = U256::from(1000);

    let data =
        UUPSProxyErc20Example::mintCall { account: alice_addr, value: amount };
    let data = data.abi_encode();

    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967")
        .with_constructor(erc1967_ctr(logic_addr, data.into()))
        .deploy()
        .await?
        .contract_address;

    let proxy_contract = Erc1967Example::new(proxy_addr, &alice.wallet);

    assert_eq!(
        logic_addr,
        proxy_contract.implementation().call().await?.implementation
    );

    assert_eq!(amount, proxy_contract.totalSupply().call().await?.totalSupply);

    // check that the balance can be accurately fetched through the proxy.
    assert_eq!(
        amount,
        proxy_contract.balanceOf(alice_addr).call().await?.balance
    );

    Ok(())
}

#[e2e::test]
async fn fallback(alice: Account, bob: Account) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let logic_addr = bob
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    let proxy_addr = alice
        .as_deployer()
        .with_constructor(ctr(logic_addr))
        .deploy()
        .await?
        .contract_address;

    let proxy_contract = Erc1967Example::new(proxy_addr, &alice.wallet);

    // verify initial balance is [`U256::ZERO`].
    assert_eq!(
        U256::ZERO,
        proxy_contract.balanceOf(alice_addr).call().await?.balance
    );

    assert_eq!(
        U256::ZERO,
        proxy_contract.totalSupply().call().await?.totalSupply
    );

    // mint 1000 tokens.
    let amount = U256::from(1000);
    watch!(proxy_contract.mint(alice_addr, amount))?;

    // check that the balance can be accurately fetched through the proxy.
    assert_eq!(
        amount,
        proxy_contract.balanceOf(alice_addr).call().await?.balance
    );

    // check that the total supply can be accurately fetched through the proxy.
    assert_eq!(amount, proxy_contract.totalSupply().call().await?.totalSupply);

    // check that the balance can be transferred through the proxy.
    let receipt = receipt!(proxy_contract.transfer(bob_addr, amount))?;

    assert!(receipt.emits(UUPSProxyErc20Example::Transfer {
        from: alice_addr,
        to: bob_addr,
        value: amount,
    }));

    assert_eq!(
        U256::ZERO,
        proxy_contract.balanceOf(alice_addr).call().await?.balance
    );

    assert_eq!(
        amount,
        proxy_contract.balanceOf(bob_addr).call().await?.balance
    );

    assert_eq!(amount, proxy_contract.totalSupply().call().await?.totalSupply);

    Ok(())
}

#[e2e::test]
async fn fallback_returns_error(alice: Account, bob: Account) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let logic_addr = bob
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    let proxy_addr = alice
        .as_deployer()
        .with_constructor(ctr(logic_addr))
        .deploy()
        .await?
        .contract_address;

    let proxy_contract = Erc1967Example::new(proxy_addr, &alice.wallet);

    let err = send!(proxy_contract.transfer(bob_addr, U256::from(1000)))
        .expect_err("should revert");

    assert!(err.reverted_with(
        UUPSProxyErc20Example::ERC20InsufficientBalance {
            sender: alice.address(),
            balance: U256::ZERO,
            needed: U256::from(1000),
        }
    ));

    Ok(())
}
