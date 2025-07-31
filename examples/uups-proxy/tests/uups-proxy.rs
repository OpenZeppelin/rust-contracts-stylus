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
use openzeppelin_stylus::proxy::erc1967::utils::IMPLEMENTATION_SLOT;
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
    let alice_addr = alice.address();

    let logic_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;

    let data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_addr,
        owner: alice_addr,
    };
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

    assert_eq!(
        U256::ZERO,
        proxy_contract.totalSupply().call().await?.totalSupply
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

    let data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_addr,
        owner: alice_addr,
    };
    let data = data.abi_encode();

    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967")
        .with_constructor(erc1967_ctr(logic_addr, data.into()))
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

    let data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_addr,
        owner: alice_addr,
    };
    let data = data.abi_encode();

    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967")
        .with_constructor(erc1967_ctr(logic_addr, data.into()))
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

#[e2e::test]
async fn upgrade_by_non_owner_fails(
    alice: Account,
    bob: Account,
    deployer: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    // deploy logic v1.
    let logic_v1_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    // deploy logic v2.
    let logic_v2_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    let data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_v1_addr,
        owner: alice_addr,
    };
    let data = data.abi_encode();

    // deploy proxy with logic v1.
    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967")
        .with_constructor(erc1967_ctr(logic_v1_addr, data.into()))
        .deploy()
        .await?
        .contract_address;

    let proxy_contract = Erc1967Example::new(proxy_addr, &bob.wallet);

    // try to upgrade with bob (non-owner) - should fail.
    let err =
        send!(proxy_contract.upgradeToAndCall(logic_v2_addr, vec![].into()))
            .expect_err("should revert on upgrade");

    assert!(err.reverted_with(
        UUPSProxyErc20Example::OwnableUnauthorizedAccount { account: bob_addr }
    ));

    Ok(())
}

#[e2e::test]
async fn upgrade_via_direct_call_reverts(
    alice: Account,
    deployer: Account,
) -> Result<()> {
    let alice_addr = alice.address();

    // deploy logic v1.
    let logic_v1_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    // deploy logic v2.
    let logic_v2_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    // create contract instance for logic v1.
    let logic_contract = Erc1967Example::new(logic_v1_addr, &alice.wallet);

    // try to upgrade logic v1 directly (not through proxy) - should fail.
    let err =
        send!(logic_contract.upgradeToAndCall(logic_v2_addr, vec![].into()))
            .expect_err("should revert on upgrade");

    assert!(err
        .reverted_with(UUPSProxyErc20Example::UUPSUnauthorizedCallContext {}));

    Ok(())
}

#[e2e::test]
async fn proxiable_uuid_direct_check(
    alice: Account,
    deployer: Account,
) -> Result<()> {
    let alice_addr = alice.address();

    // deploy logic contract.
    let logic_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    // create contract instance.
    let logic_contract = UUPSProxyErc20Example::new(logic_addr, &alice.wallet);

    // get proxiable UUID.
    let result = logic_contract.proxiableUUID().call().await?._0;
    assert_eq!(result, IMPLEMENTATION_SLOT);

    Ok(())
}

#[e2e::test]
async fn upgrades(
    alice: Account,
    _bob: Account,
    _deployer: Account,
) -> Result<()> {
    let alice_addr = alice.address();

    // deploy logic v1 with alice as owner.
    let logic_v1_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    // deploy logic v2 with alice as owner.
    let logic_v2_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    let data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_v1_addr,
        owner: alice_addr,
    };
    let data = data.abi_encode();

    // deploy proxy with logic v1, deployed by alice (who will be the owner).
    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967")
        .with_constructor(erc1967_ctr(logic_v1_addr, data.into()))
        .deploy()
        .await?
        .contract_address;

    let proxy_contract = Erc1967Example::new(proxy_addr, &alice.wallet);

    // verify initial implementation.
    let initial_impl =
        proxy_contract.implementation().call().await?.implementation;
    assert_eq!(initial_impl, logic_v1_addr);

    assert_eq!(alice_addr, proxy_contract.owner().call().await?.owner);

    let data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_v2_addr,
        owner: alice_addr,
    };
    let data = data.abi_encode();
    // upgrade to logic v2.
    let receipt =
        receipt!(proxy_contract.upgradeToAndCall(logic_v2_addr, data.into()))?;

    // check that the Upgraded event was emitted.
    assert!(receipt
        .emits(Erc1967Example::Upgraded { implementation: logic_v2_addr }));

    // verify the implementation was upgraded.
    let new_impl = proxy_contract.implementation().call().await?.implementation;
    assert_eq!(new_impl, logic_v2_addr);

    // check that the balance can be fetched through the upgraded proxy.
    let balance = proxy_contract.balanceOf(alice_addr).call().await?.balance;
    assert_eq!(balance, U256::ZERO);

    Ok(())
}

#[e2e::test]
async fn upgrades_with_ownership_transfer(
    alice: Account,
    bob: Account,
    _deployer: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    // deploy logic v1 with alice as owner.
    let logic_v1_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    // deploy logic v2 with alice as owner.
    let logic_v2_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    let data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_v1_addr,
        owner: alice_addr,
    };
    let data = data.abi_encode();

    // deploy proxy with logic v1, deployed by alice (who will be the owner).
    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967")
        .with_constructor(erc1967_ctr(logic_v1_addr, data.into()))
        .deploy()
        .await?
        .contract_address;

    let proxy_contract = Erc1967Example::new(proxy_addr, &alice.wallet);

    // verify initial implementation.
    let initial_impl =
        proxy_contract.implementation().call().await?.implementation;
    assert_eq!(initial_impl, logic_v1_addr);

    // transfer ownership to bob.
    receipt!(proxy_contract.transferOwnership(bob_addr))?;

    // verify bob is now the owner.
    assert_eq!(bob_addr, proxy_contract.owner().call().await?.owner);

    // alice can no longer upgrade.
    let err =
        send!(proxy_contract.upgradeToAndCall(logic_v2_addr, vec![].into()))
            .expect_err("should revert on upgrade");

    assert!(err.reverted_with(
        UUPSProxyErc20Example::OwnableUnauthorizedAccount {
            account: alice_addr
        }
    ));

    // create proxy contract instance with bob's wallet for the upgrade.
    let proxy_contract_bob = Erc1967Example::new(proxy_addr, &bob.wallet);

    let data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_v2_addr,
        owner: bob_addr,
    };
    let data = data.abi_encode();

    // upgrade to logic v2 with bob as the owner.
    let receipt = receipt!(
        proxy_contract_bob.upgradeToAndCall(logic_v2_addr, data.into())
    )?;

    // check that the Upgraded event was emitted.
    assert!(receipt
        .emits(Erc1967Example::Upgraded { implementation: logic_v2_addr }));

    // verify the implementation was upgraded.
    let new_impl =
        proxy_contract_bob.implementation().call().await?.implementation;
    assert_eq!(new_impl, logic_v2_addr);

    // check that the balance can be fetched through the upgraded proxy.
    let balance =
        proxy_contract_bob.balanceOf(alice_addr).call().await?.balance;
    assert_eq!(balance, U256::ZERO);

    Ok(())
}
