#![cfg(feature = "e2e")]

use abi::{Erc1967Example, UUPSProxyErc20Example};
use alloy::{
    primitives::{Address, B256, U256},
    sol_types::SolCall,
};
use e2e::{
    constructor, receipt, send, watch, Account, Constructor, EventExt, Revert,
};
use eyre::Result;
use openzeppelin_stylus::proxy::{
    erc1967::utils::IMPLEMENTATION_SLOT,
    utils::uups_upgradeable::UPGRADE_INTERFACE_VERSION,
};
use stylus_sdk::abi::Bytes;

mod abi;

fn ctr(owner: Address) -> Constructor {
    constructor!(owner)
}

fn erc1967_ctr(implementation: Address, data: Bytes) -> Constructor {
    constructor!(implementation, data.clone())
}

#[e2e::test]
async fn constructs(alice: Account, deployer: Account) -> Result<()> {
    let alice_addr = alice.address();

    let logic_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    let data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_addr,
        owner: alice_addr,
    }
    .abi_encode();

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
async fn upgrade_interface_version(
    alice: Account,
    deployer: Account,
) -> Result<()> {
    let alice_addr = alice.address();

    let logic_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    let data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_addr,
        owner: alice_addr,
    }
    .abi_encode();

    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967")
        .with_constructor(erc1967_ctr(logic_addr, data.into()))
        .deploy()
        .await?
        .contract_address;

    let proxy_contract = Erc1967Example::new(proxy_addr, &alice.wallet);

    assert_eq!(
        UPGRADE_INTERFACE_VERSION,
        proxy_contract.upgradeInterfaceVersion().call().await?._0
    );

    let logic_contract = UUPSProxyErc20Example::new(logic_addr, &alice.wallet);

    assert_eq!(
        UPGRADE_INTERFACE_VERSION,
        logic_contract.upgradeInterfaceVersion().call().await?._0
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
    }
    .abi_encode();

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
    }
    .abi_encode();

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
    }
    .abi_encode();

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
    }
    .abi_encode();

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
    }
    .abi_encode();

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
async fn upgrade_to_no_proxiable_uuid_reverts(
    alice: Account,
    deployer: Account,
) -> Result<()> {
    let alice_addr = alice.address();

    // deploy a valid UUPS-compatible logic contract (v1).
    let logic_v1_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    let init_data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_v1_addr,
        owner: alice_addr,
    }
    .abi_encode();

    // deploy proxy using logic v1.
    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967")
        .with_constructor(erc1967_ctr(logic_v1_addr, init_data.into()))
        .deploy()
        .await?
        .contract_address;

    let proxy_contract = Erc1967Example::new(proxy_addr, &alice.wallet);

    // deploy an "invalid" logic contract that does NOT have the proxiable UUID.
    let invalid_logic_addr = deployer
        .as_deployer()
        .with_example_name("erc20-permit")
        .deploy()
        .await?
        .contract_address;

    // try upgrading to the invalid implementation.
    let err = send!(
        proxy_contract.upgradeToAndCall(invalid_logic_addr, vec![].into())
    )
    .expect_err("upgrade should revert due to no proxiable UUID interface");

    assert!(err.reverted_with(
        UUPSProxyErc20Example::ERC1967InvalidImplementation {
            implementation: invalid_logic_addr
        }
    ));

    Ok(())
}

#[e2e::test]
async fn upgrade_to_invalid_proxiable_uuid_reverts(
    alice: Account,
    deployer: Account,
) -> Result<()> {
    let alice_addr = alice.address();

    // deploy a valid UUPS-compatible logic contract (v1).
    let logic_v1_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    let init_data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_v1_addr,
        owner: alice_addr,
    }
    .abi_encode();

    // deploy proxy using logic v1.
    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967")
        .with_constructor(erc1967_ctr(logic_v1_addr, init_data.into()))
        .deploy()
        .await?
        .contract_address;

    let proxy_contract = Erc1967Example::new(proxy_addr, &alice.wallet);

    // deploy an "invalid" logic contract that has an invalid proxiable UUID.
    let invalid_logic_addr = deployer
        .as_deployer()
        .with_example_name("ownable")
        .deploy()
        .await?
        .contract_address;

    // try upgrading to the invalid implementation.
    let err = send!(
        proxy_contract.upgradeToAndCall(invalid_logic_addr, vec![].into())
    )
    .expect_err("upgrade should revert due to aninvalid proxiable UUID");

    assert!(err.reverted_with(
        UUPSProxyErc20Example::UUPSUnsupportedProxiableUUID {
            slot: B256::ZERO
        }
    ));

    Ok(())
}

#[e2e::test]
async fn upgrades_preserve_storage(
    alice: Account,
    deployer: Account,
) -> Result<()> {
    let alice_addr = alice.address();

    // deploy logic V1.
    let logic_v1_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    // encode initializer call for logic V1.
    let init_data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_v1_addr,
        owner: alice_addr,
    }
    .abi_encode();

    // deploy proxy using logic V1.
    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967")
        .with_constructor(erc1967_ctr(logic_v1_addr, init_data.into()))
        .deploy()
        .await?
        .contract_address;

    // interact with proxy via logic V1.
    let proxy_contract = Erc1967Example::new(proxy_addr, &alice.wallet);

    // mint tokens pre-upgrade.
    let amount = U256::from(12345);
    watch!(proxy_contract.mint(alice_addr, amount))?;

    assert_eq!(
        amount,
        proxy_contract.balanceOf(alice_addr).call().await?.balance
    );
    assert_eq!(amount, proxy_contract.totalSupply().call().await?.totalSupply);

    // deploy logic V2 (must use same storage layout).
    let logic_v2_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    let data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_v2_addr,
        owner: alice_addr,
    }
    .abi_encode();

    // upgrade proxy to logic V2.
    let receipt =
        receipt!(proxy_contract.upgradeToAndCall(logic_v2_addr, data.into()))?;

    assert!(receipt
        .emits(Erc1967Example::Upgraded { implementation: logic_v2_addr }));

    // verify storage consistency.
    assert_eq!(
        amount,
        proxy_contract.balanceOf(alice_addr).call().await?.balance
    );
    assert_eq!(amount, proxy_contract.totalSupply().call().await?.totalSupply);

    Ok(())
}

#[e2e::test]
async fn upgrade_to_same_implementation_succeeds(
    alice: Account,
    deployer: Account,
) -> Result<()> {
    let alice_addr = alice.address();

    // deploy logic V1 with Alice as owner.
    let logic_addr = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    // encode initializer data.
    let init_data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic_addr,
        owner: alice_addr,
    }
    .abi_encode();

    // deploy proxy pointing to logic V1.
    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967")
        .with_constructor(erc1967_ctr(logic_addr, init_data.into()))
        .deploy()
        .await?
        .contract_address;

    let proxy_contract = Erc1967Example::new(proxy_addr, &alice.wallet);

    // sanity check: implementation is correct.
    let current_impl =
        proxy_contract.implementation().call().await?.implementation;
    assert_eq!(current_impl, logic_addr);

    // try re-upgrading to the same implementation.
    let receipt =
        receipt!(proxy_contract.upgradeToAndCall(logic_addr, vec![].into()))?;

    assert!(
        receipt.emits(Erc1967Example::Upgraded { implementation: logic_addr })
    );

    // confirm implementation didn't change.
    let new_impl = proxy_contract.implementation().call().await?.implementation;
    assert_eq!(new_impl, logic_addr);

    Ok(())
}

#[e2e::test]
async fn upgrade_to_non_contract_address_fails(
    alice: Account,
    deployer: Account,
) -> Result<()> {
    let alice_addr = alice.address();

    let logic = deployer
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;

    let init_data = UUPSProxyErc20Example::initializeCall {
        selfAddress: logic,
        owner: alice_addr,
    }
    .abi_encode();

    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967")
        .with_constructor(erc1967_ctr(logic, init_data.into()))
        .deploy()
        .await?
        .contract_address;

    let proxy = Erc1967Example::new(proxy_addr, &alice.wallet);

    // try to upgrade to a random address (not a contract).
    let non_contract = Address::from([0x99; 20]);
    let err = send!(proxy.upgradeToAndCall(non_contract, vec![].into()))
        .expect_err("Expected revert upgrading to non-contract");

    assert!(err.reverted_with(
        UUPSProxyErc20Example::ERC1967InvalidImplementation {
            implementation: non_contract
        }
    ));

    Ok(())
}
