#![cfg(feature = "e2e")]

use abi::{Erc1967Example, UUPSProxyErc20Example};
use alloy::{
    primitives::{uint, Address, B256, U256},
    sol_types::SolCall,
};
use alloy_primitives::U32;
use alloy_sol_types::SolError;
use e2e::{
    constructor, receipt, send, watch, Account, EventExt, Receipt, Revert,
};
use eyre::Result;
use openzeppelin_stylus::proxy::{
    erc1967::utils::IMPLEMENTATION_SLOT,
    utils::uups_upgradeable::{self, UPGRADE_INTERFACE_VERSION},
};
use stylus_sdk::abi::Bytes;

mod abi;

trait Deploy {
    async fn deploy_uups(&self) -> Result<Receipt>;
    async fn deploy_uups_new_version(&self) -> Result<Receipt>;
    async fn deploy_proxy(
        &self,
        implementation: Address,
        owner: Address,
    ) -> Result<Receipt>;
}

impl Deploy for Account {
    async fn deploy_uups(&self) -> Result<Receipt> {
        self.as_deployer()
            .with_constructor(constructor!(self.address()))
            .deploy()
            .await
    }

    async fn deploy_uups_new_version(&self) -> Result<Receipt> {
        self.as_deployer()
            .with_example_name("uups-proxy-new-version")
            .with_constructor(constructor!(self.address()))
            .deploy()
            .await
    }

    async fn deploy_proxy(
        &self,
        implementation: Address,
        owner: Address,
    ) -> Result<Receipt> {
        let data: Bytes =
            UUPSProxyErc20Example::initializeCall { owner }.abi_encode().into();

        self.as_deployer()
            .with_example_name("erc1967")
            .with_constructor(constructor!(implementation, data.clone()))
            .deploy()
            .await
    }
}

#[e2e::test]
async fn upgrade_through_valid_proxy_succeeds(alice: Account) -> Result<()> {
    let logic_addr = alice.deploy_uups().await?.contract_address;

    let proxy_addr =
        alice.deploy_proxy(logic_addr, alice.address()).await?.contract_address;

    let logic = UUPSProxyErc20Example::new(logic_addr, &alice.wallet);
    let proxy = Erc1967Example::new(proxy_addr, &alice.wallet);

    // assert proxy and logic initialized correctly
    assert_eq!(logic_addr, proxy.implementation().call().await?.implementation);
    assert_eq!(
        UPGRADE_INTERFACE_VERSION,
        proxy.UPGRADE_INTERFACE_VERSION().call().await?.version,
    );
    assert_eq!(
        UPGRADE_INTERFACE_VERSION,
        logic.UPGRADE_INTERFACE_VERSION().call().await?.version,
    );
    assert_eq!(
        uups_upgradeable::VERSION_NUMBER,
        U32::from(proxy.getVersion().call().await?.version),
    );
    assert_eq!(
        uups_upgradeable::VERSION_NUMBER,
        U32::from(logic.getVersion().call().await?.version),
    );
    // check that state is set correctly
    assert_eq!(alice.address(), proxy.owner().call().await?.owner);

    // deploy the new UUPS contract
    let new_logic_addr =
        alice.deploy_uups_new_version().await?.contract_address;
    let new_logic = UUPSProxyErc20Example::new(new_logic_addr, &alice.wallet);

    // no need to reinitialize the proxy state when upgrading, as old state
    // should be maintained
    let receipt =
        receipt!(proxy.upgradeToAndCall(new_logic_addr, vec![].into()))?;

    assert!(receipt
        .emits(Erc1967Example::Upgraded { implementation: new_logic_addr }));

    assert_eq!(
        new_logic_addr,
        proxy.implementation().call().await?.implementation
    );
    assert_eq!(
        UPGRADE_INTERFACE_VERSION,
        proxy.UPGRADE_INTERFACE_VERSION().call().await?.version,
    );
    assert_eq!(
        uups_proxy_new_version_example::VERSION_NUMBER,
        U32::from(proxy.getVersion().call().await?.version),
    );
    assert_eq!(
        uups_proxy_new_version_example::VERSION_NUMBER,
        U32::from(new_logic.getVersion().call().await?.version),
    );

    // Alice should still be the owner
    assert_eq!(alice.address(), proxy.owner().call().await?.owner);

    Ok(())
}

#[e2e::test]
async fn upgrading_directly_on_uups_reverts(alice: Account) -> Result<()> {
    let logic_addr = alice.deploy_uups().await?.contract_address;
    let logic = UUPSProxyErc20Example::new(logic_addr, &alice.wallet);

    let new_logic_addr = alice.deploy_uups().await?.contract_address;

    let err = send!(logic.upgradeToAndCall(new_logic_addr, vec![].into()))
        .expect_err("should revert");

    assert!(err
        .reverted_with(UUPSProxyErc20Example::UUPSUnauthorizedCallContext {}));

    Ok(())
}

#[e2e::test]
async fn upgrading_via_invalid_erc1967_proxy_reverts(
    alice: Account,
) -> Result<()> {
    let logic_addr = alice.deploy_uups().await?.contract_address;

    let data: Bytes =
        UUPSProxyErc20Example::initializeCall { owner: alice.address() }
            .abi_encode()
            .into();

    let proxy_addr = alice
        .as_deployer()
        .with_example_name("erc1967-invalid")
        .with_constructor(constructor!(logic_addr, data.clone()))
        .deploy()
        .await?
        .contract_address;

    let proxy = Erc1967Example::new(proxy_addr, &alice.wallet);

    let new_logic_addr =
        alice.deploy_uups_new_version().await?.contract_address;

    let err = send!(proxy.upgradeToAndCall(new_logic_addr, vec![].into()))
        .expect_err("should revert");

    assert!(err
        .reverted_with(UUPSProxyErc20Example::UUPSUnauthorizedCallContext {}));

    Ok(())
}

#[e2e::test]
async fn set_version_doesnt_revert_if_called_more_than_once(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let logic_addr = alice.deploy_uups().await?.contract_address;

    let proxy_addr =
        alice.deploy_proxy(logic_addr, alice.address()).await?.contract_address;

    let proxy = Erc1967Example::new(proxy_addr, &alice.wallet);

    // internally calls `set_version`
    assert!(watch!(proxy.initialize(bob.address())).is_ok());

    // CAUTION: Bob is now the owner
    assert_eq!(bob.address(), proxy.owner().call().await?.owner);

    Ok(())
}

#[e2e::test]
async fn fallback_works(alice: Account, bob: Account) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let logic_addr = alice.deploy_uups().await?.contract_address;

    let proxy_addr =
        alice.deploy_proxy(logic_addr, alice_addr).await?.contract_address;

    let proxy = Erc1967Example::new(proxy_addr, &alice.wallet);

    // verify initial balance is [`U256::ZERO`].
    assert_eq!(U256::ZERO, proxy.balanceOf(alice_addr).call().await?.balance);

    assert_eq!(U256::ZERO, proxy.totalSupply().call().await?.totalSupply);

    // mint 1000 tokens.
    let amount = uint!(1000_U256);
    watch!(proxy.mint(alice_addr, amount))?;

    // check that the balance can be accurately fetched through the proxy.
    assert_eq!(amount, proxy.balanceOf(alice_addr).call().await?.balance);

    // check that the total supply can be accurately fetched through the proxy.
    assert_eq!(amount, proxy.totalSupply().call().await?.totalSupply);

    // check that the balance can be transferred through the proxy.
    let receipt = receipt!(proxy.transfer(bob_addr, amount))?;

    assert!(receipt.emits(Erc1967Example::Transfer {
        from: alice_addr,
        to: bob_addr,
        value: amount,
    }));

    // assert state was properly updated
    assert_eq!(U256::ZERO, proxy.balanceOf(alice_addr).call().await?.balance);

    assert_eq!(amount, proxy.balanceOf(bob_addr).call().await?.balance);

    assert_eq!(amount, proxy.totalSupply().call().await?.totalSupply);

    Ok(())
}

#[e2e::test]
async fn upgrade_to_non_erc1822_reverts(alice: Account) -> Result<()> {
    // deploy a valid UUPS-compatible logic contract (v1).
    let logic_addr = alice.deploy_uups().await?.contract_address;

    // deploy proxy using logic v1.
    let proxy_addr =
        alice.deploy_proxy(logic_addr, alice.address()).await?.contract_address;

    let proxy = Erc1967Example::new(proxy_addr, &alice.wallet);

    // deploy an "invalid" logic contract that does NOT have the proxiable UUID.
    let invalid_logic_addr = alice
        .as_deployer()
        .with_example_name("erc20-permit")
        .deploy()
        .await?
        .contract_address;

    // try upgrading to the invalid implementation.
    let err = send!(proxy.upgradeToAndCall(invalid_logic_addr, vec![].into()))
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
) -> Result<()> {
    let alice_addr = alice.address();

    // deploy a valid UUPS-compatible logic contract (v1).
    let logic_v1_addr = alice.deploy_uups().await?.contract_address;

    let proxy_addr =
        alice.deploy_proxy(logic_v1_addr, alice_addr).await?.contract_address;

    let proxy = Erc1967Example::new(proxy_addr, &alice.wallet);

    // deploy an "invalid" logic contract that has an invalid proxiable UUID.
    // see: examples/ownable/src/lib.rs
    let invalid_logic_addr = alice
        .as_deployer()
        .with_example_name("ownable")
        .deploy()
        .await?
        .contract_address;

    // try upgrading to the invalid implementation.
    let err = send!(proxy.upgradeToAndCall(invalid_logic_addr, vec![].into()))
        .expect_err("upgrade should revert due to aninvalid proxiable UUID");

    assert!(err.reverted_with(
        UUPSProxyErc20Example::UUPSUnsupportedProxiableUUID {
            slot: B256::ZERO
        }
    ));

    Ok(())
}

#[e2e::test]
async fn upgrade_preserves_storage(alice: Account) -> Result<()> {
    let alice_addr = alice.address();

    let logic_v1_addr = alice.deploy_uups().await?.contract_address;

    let proxy_addr =
        alice.deploy_proxy(logic_v1_addr, alice_addr).await?.contract_address;

    // interact with proxy via logic V1.
    let proxy = Erc1967Example::new(proxy_addr, &alice.wallet);

    // mint tokens pre-upgrade.
    let amount = uint!(12345_U256);
    watch!(proxy.mint(alice_addr, amount))?;

    let old_balance = proxy.balanceOf(alice_addr).call().await?.balance;
    let old_total_supply = proxy.totalSupply().call().await?.totalSupply;

    assert_eq!(amount, old_balance);
    assert_eq!(amount, old_total_supply);

    // deploy logic V2 (must use same storage layout).
    let logic_v2_addr = alice.deploy_uups_new_version().await?.contract_address;

    // upgrade proxy to logic V2.
    let receipt =
        receipt!(proxy.upgradeToAndCall(logic_v2_addr, vec![].into()))?;

    assert!(receipt
        .emits(Erc1967Example::Upgraded { implementation: logic_v2_addr }));

    // verify storage consistency.
    assert_eq!(old_balance, proxy.balanceOf(alice_addr).call().await?.balance);
    assert_eq!(old_total_supply, proxy.totalSupply().call().await?.totalSupply);

    Ok(())
}

#[e2e::test]
async fn upgrade_to_same_implementation_succeeds(alice: Account) -> Result<()> {
    let alice_addr = alice.address();

    let logic_addr = alice.deploy_uups().await?.contract_address;

    let proxy_addr =
        alice.deploy_proxy(logic_addr, alice_addr).await?.contract_address;

    let proxy = Erc1967Example::new(proxy_addr, &alice.wallet);

    // sanity check: implementation is correct.
    let current_impl = proxy.implementation().call().await?.implementation;
    assert_eq!(current_impl, logic_addr);

    // try re-upgrading to the same implementation.
    let receipt = receipt!(proxy.upgradeToAndCall(logic_addr, vec![].into()))?;

    assert!(
        receipt.emits(Erc1967Example::Upgraded { implementation: logic_addr })
    );

    // confirm implementation didn't change.
    let new_impl = proxy.implementation().call().await?.implementation;
    assert_eq!(new_impl, logic_addr);

    Ok(())
}

#[e2e::test]
async fn upgrade_to_implementation_with_same_version_succeeds(
    alice: Account,
) -> Result<()> {
    let alice_addr = alice.address();

    let logic_addr = alice.deploy_uups().await?.contract_address;

    let proxy_addr =
        alice.deploy_proxy(logic_addr, alice_addr).await?.contract_address;

    let proxy = Erc1967Example::new(proxy_addr, &alice.wallet);
    let logic = UUPSProxyErc20Example::new(logic_addr, &alice.wallet);

    assert_eq!(
        uups_upgradeable::VERSION_NUMBER,
        U32::from(proxy.getVersion().call().await?.version),
    );
    assert_eq!(
        uups_upgradeable::VERSION_NUMBER,
        U32::from(logic.getVersion().call().await?.version),
    );

    // sanity check: implementation is correct.
    let new_logic_addr = alice.deploy_uups().await?.contract_address;
    let new_logic = UUPSProxyErc20Example::new(new_logic_addr, &alice.wallet);

    // try re-upgrading to the same implementation.
    let receipt =
        receipt!(proxy.upgradeToAndCall(new_logic_addr, vec![].into()))?;

    assert!(receipt
        .emits(Erc1967Example::Upgraded { implementation: new_logic_addr }));

    // confirm implementation didn't change.
    let new_impl = proxy.implementation().call().await?.implementation;
    assert_eq!(new_impl, new_logic_addr);

    assert_eq!(
        uups_upgradeable::VERSION_NUMBER,
        U32::from(proxy.getVersion().call().await?.version),
    );
    assert_eq!(
        uups_upgradeable::VERSION_NUMBER,
        U32::from(new_logic.getVersion().call().await?.version),
    );

    Ok(())
}

#[e2e::test]
async fn upgrade_reverts_on_underlying_erc1967_upgrade_failure(
    alice: Account,
) -> Result<()> {
    let logic_addr = alice.deploy_uups().await?.contract_address;
    let proxy_addr =
        alice.deploy_proxy(logic_addr, alice.address()).await?.contract_address;

    let proxy = Erc1967Example::new(proxy_addr, &alice.wallet);

    let new_logic_addr =
        alice.deploy_uups_new_version().await?.contract_address;

    // sending empty data + tx value will force the upgrade to revert
    let err = send!(proxy
        .upgradeToAndCall(new_logic_addr, vec![].into())
        .value(U256::ONE))
    .expect_err("should revert");

    assert!(err.reverted_with(Erc1967Example::ERC1967NonPayable {}));

    Ok(())
}

#[e2e::test]
async fn proxiable_uuid_can_only_be_called_directly_on_uups(
    alice: Account,
) -> Result<()> {
    let logic_addr = alice.deploy_uups().await?.contract_address;

    let logic = UUPSProxyErc20Example::new(logic_addr, &alice.wallet);

    assert_eq!(IMPLEMENTATION_SLOT, logic.proxiableUUID().call().await?.uuid);

    // calling through a proxy should revert
    let proxy_addr =
        alice.deploy_proxy(logic_addr, alice.address()).await?.contract_address;
    let proxy = Erc1967Example::new(proxy_addr, &alice.wallet);

    let err = proxy.proxiableUUID().call().await.expect_err("should revert");

    assert!(err
        .reverted_with(UUPSProxyErc20Example::UUPSUnauthorizedCallContext {}));

    Ok(())
}

#[e2e::test]
async fn set_version_can_only_be_delegate_called(alice: Account) -> Result<()> {
    let logic_addr = alice.deploy_uups().await?.contract_address;

    let logic = UUPSProxyErc20Example::new(logic_addr, &alice.wallet);

    let err = logic.setVersion().call().await.expect_err("should revert");

    assert!(err
        .reverted_with(UUPSProxyErc20Example::UUPSUnauthorizedCallContext {}));

    Ok(())
}

#[e2e::test]
async fn version_cannot_be_decreased(alice: Account) -> Result<()> {
    let logic_addr = alice.deploy_uups_new_version().await?.contract_address;

    let proxy_addr =
        alice.deploy_proxy(logic_addr, alice.address()).await?.contract_address;

    let proxy = Erc1967Example::new(proxy_addr, &alice.wallet);

    let previous_logic_addr = alice.deploy_uups().await?.contract_address;

    let err = send!(proxy.upgradeToAndCall(previous_logic_addr, vec![].into()))
        .expect_err("should revert");

    assert!(err.reverted_with(Erc1967Example::FailedCallWithReason {
        reason: UUPSProxyErc20Example::InvalidVersion {
            current_version: uups_proxy_new_version_example::VERSION_NUMBER
                .to()
        }
        .abi_encode()
        .into()
    }));

    Ok(())
}
