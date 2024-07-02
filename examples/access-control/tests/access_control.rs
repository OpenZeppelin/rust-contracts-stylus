#![cfg(feature = "e2e")]

use alloy::{
    primitives::Address,
    providers::Provider,
    rpc::types::{BlockNumberOrTag, Filter},
    sol,
    sol_types::{SolConstructor, SolError, SolEvent},
};
use e2e::{receipt, send, Account, EventExt, Revert};
use eyre::Result;

use crate::abi::AccessControl;

mod abi;

async fn deploy(account: &Account) -> eyre::Result<Address> {
    let args = AccessControl::constructorCall {};
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(account.url(), &account.pk(), Some(args)).await
}

// ============================================================================
// Integration Tests: AccessControl
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr = deploy(&alice).await?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let default_role =
        openzeppelin_stylus::access::control::AccessControl::DEFAULT_ADMIN_ROLE;
    let AccessControl::hasRoleReturn { hasRole } =
        contract.hasRole(default_role.into(), alice_addr).call().await?;
    assert!(hasRole);

    Ok(())
}
