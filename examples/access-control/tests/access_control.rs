#![cfg(feature = "e2e")]

use access_control_example::TRANSFER_ROLE;
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

const DEFAULT_ADMIN_ROLE: [u8; 32] =
    openzeppelin_stylus::access::control::AccessControl::DEFAULT_ADMIN_ROLE;

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

    let AccessControl::hasRoleReturn { hasRole } =
        contract.hasRole(DEFAULT_ADMIN_ROLE.into(), alice_addr).call().await?;
    assert!(hasRole);

    Ok(())
}

#[e2e::test]
async fn other_roles_admin_is_the_default_adming_role(
    alice: Account,
) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let AccessControl::getRoleAdminReturn { role } =
        contract.getRoleAdmin(TRANSFER_ROLE.into()).call().await?;
    assert_eq!(*role, DEFAULT_ADMIN_ROLE);

    Ok(())
}

#[e2e::test]
async fn default_roles_admin_is_default(alice: Account) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let AccessControl::getRoleAdminReturn { role } =
        contract.getRoleAdmin(TRANSFER_ROLE.into()).call().await?;
    assert_eq!(*role, DEFAULT_ADMIN_ROLE);

    let AccessControl::getRoleAdminReturn { role } =
        contract.getRoleAdmin(DEFAULT_ADMIN_ROLE.into()).call().await?;
    assert_eq!(*role, DEFAULT_ADMIN_ROLE);

    Ok(())
}

#[e2e::test]
async fn non_admin_cannot_grant_role(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
    let contract = AccessControl::new(contract_addr, &bob.wallet);

    let err = send!(contract.grantRole(TRANSFER_ROLE.into(), alice.address()))
        .expect_err("should not have permission to grant roles");
    assert!(err.reverted_with(
        AccessControl::AccessControlUnauthorizedAccount {
            account: bob.address(),
            neededRole: DEFAULT_ADMIN_ROLE.into()
        }
    ));

    Ok(())
}

#[e2e::test]
async fn accounts_can_be_granted_roles_multiple_times(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let _ = watch!(contract.grantRole(TRANSFER_ROLE.into(), bob.address()))?;
    let _ = watch!(contract.grantRole(TRANSFER_ROLE.into(), bob.address()))?;
    /// TODO: Check `RoleGranted` does not get emitted.
    Ok(())
}
