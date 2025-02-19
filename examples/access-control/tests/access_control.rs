#![cfg(feature = "e2e")]

use abi::AccessControl::{
    self, AccessControlBadConfirmation, AccessControlUnauthorizedAccount,
    RoleAdminChanged, RoleGranted, RoleRevoked,
};
use alloy::hex;
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt, Revert};
use eyre::Result;

mod abi;

const DEFAULT_ADMIN_ROLE: [u8; 32] =
    openzeppelin_stylus::access::control::AccessControl::DEFAULT_ADMIN_ROLE;
const ROLE: [u8; 32] = access_control_example::TRANSFER_ROLE;
const NEW_ADMIN_ROLE: [u8; 32] =
    hex!("879ce0d4bfd332649ca3552efe772a38d64a315eb70ab69689fd309c735946b5");

// ============================================================================
// Integration Tests: AccessControl
// ============================================================================

#[e2e::test]
async fn constructor_succeeds_with_default_admin_role(
    alice: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let receipt = alice.as_deployer().deploy().await?;
    let contract = AccessControl::new(receipt.address()?, &alice.wallet);

    assert!(receipt.emits(RoleGranted {
        role: DEFAULT_ADMIN_ROLE.into(),
        account: alice_addr,
        sender: alice_addr
    }));

    let AccessControl::hasRoleReturn { hasRole } =
        contract.hasRole(DEFAULT_ADMIN_ROLE.into(), alice_addr).call().await?;
    assert!(hasRole);

    Ok(())
}

#[e2e::test]
async fn get_role_admin_returns_default_admin_role(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let AccessControl::getRoleAdminReturn { role } =
        contract.getRoleAdmin(ROLE.into()).call().await?;
    assert_eq!(*role, DEFAULT_ADMIN_ROLE);

    Ok(())
}

#[e2e::test]
async fn get_role_admin_returns_correct_default_admin_role(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let AccessControl::getRoleAdminReturn { role } =
        contract.getRoleAdmin(ROLE.into()).call().await?;
    assert_eq!(*role, DEFAULT_ADMIN_ROLE);

    let AccessControl::getRoleAdminReturn { role } =
        contract.getRoleAdmin(DEFAULT_ADMIN_ROLE.into()).call().await?;
    assert_eq!(*role, DEFAULT_ADMIN_ROLE);

    Ok(())
}

#[e2e::test]
async fn grant_role_reverts_when_caller_lacks_admin_permission(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &bob.wallet);

    let err = send!(contract.grantRole(ROLE.into(), alice.address()))
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
async fn grant_role_succeeds_with_multiple_identical_grants(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let receipt = receipt!(contract.grantRole(ROLE.into(), bob_addr))?;
    assert!(receipt.emits(RoleGranted {
        role: ROLE.into(),
        account: bob_addr,
        sender: alice_addr
    }));
    let receipt = receipt!(contract.grantRole(ROLE.into(), bob_addr))?;
    assert!(!receipt.emits(RoleGranted {
        role: ROLE.into(),
        account: bob_addr,
        sender: alice_addr
    }));

    Ok(())
}

#[e2e::test]
async fn revoke_role_succeeds_when_role_not_previously_granted(
    alice: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let AccessControl::hasRoleReturn { hasRole } =
        contract.hasRole(ROLE.into(), alice_addr).call().await?;
    assert!(!hasRole);

    let receipt = receipt!(contract.revokeRole(ROLE.into(), alice_addr))?;
    assert!(!receipt.emits(RoleRevoked {
        role: ROLE.into(),
        account: alice_addr,
        sender: alice_addr
    }));

    Ok(())
}

#[e2e::test]
async fn revoke_role_succeeds_for_previously_granted_role(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let _ = watch!(contract.grantRole(ROLE.into(), bob_addr))?;

    let receipt = receipt!(contract.revokeRole(ROLE.into(), bob_addr))?;
    assert!(receipt.emits(RoleRevoked {
        role: ROLE.into(),
        account: bob_addr,
        sender: alice_addr
    }));

    Ok(())
}

#[e2e::test]
async fn revoke_role_reverts_when_caller_lacks_admin_permission(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let _ = watch!(contract.grantRole(ROLE.into(), alice_addr))?;

    let contract = AccessControl::new(contract_addr, &bob.wallet);
    let err = send!(contract.revokeRole(ROLE.into(), alice_addr))
        .expect_err("non-admin should not be able to revoke role");
    assert!(err.reverted_with(AccessControlUnauthorizedAccount {
        account: bob_addr,
        neededRole: DEFAULT_ADMIN_ROLE.into()
    }));

    Ok(())
}

#[e2e::test]
async fn revoke_role_succeeds_with_multiple_identical_revokes(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let _ = watch!(contract.revokeRole(ROLE.into(), bob_addr))?;
    let receipt = receipt!(contract.revokeRole(ROLE.into(), bob_addr))?;
    assert!(!receipt.emits(RoleRevoked {
        role: ROLE.into(),
        account: bob_addr,
        sender: alice_addr
    }));

    Ok(())
}

#[e2e::test]
async fn renounce_role_succeeds_when_role_not_previously_granted(
    alice: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let receipt = receipt!(contract.renounceRole(ROLE.into(), alice_addr))?;
    assert!(!receipt.emits(RoleRevoked {
        role: ROLE.into(),
        account: alice_addr,
        sender: alice_addr
    }));

    Ok(())
}

#[e2e::test]
async fn renounce_role_succeeds_for_previously_granted_role(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let bob_addr = bob.address();
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let _ = watch!(contract.grantRole(ROLE.into(), bob_addr))?;

    let contract = AccessControl::new(contract_addr, &bob.wallet);
    let receipt = receipt!(contract.renounceRole(ROLE.into(), bob_addr))?;
    assert!(receipt.emits(RoleRevoked {
        role: ROLE.into(),
        account: bob_addr,
        sender: bob_addr
    }));

    Ok(())
}

#[e2e::test]
async fn renounce_role_reverts_when_not_role_holder(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let _ = watch!(contract.grantRole(ROLE.into(), bob_addr))?;

    let contract = AccessControl::new(contract_addr, &bob.wallet);
    let err = send!(contract.renounceRole(ROLE.into(), alice_addr))
        .expect_err("only sender should be able to renounce");
    assert!(err.reverted_with(AccessControlBadConfirmation {}));

    Ok(())
}

#[e2e::test]
async fn renounce_role_succeeds_with_multiple_identical_renounces(
    alice: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let _ = watch!(contract.renounceRole(ROLE.into(), alice_addr))?;
    let receipt = receipt!(contract.renounceRole(ROLE.into(), alice_addr))?;
    assert!(!receipt.emits(RoleRevoked {
        role: ROLE.into(),
        account: alice_addr,
        sender: alice_addr
    }));

    Ok(())
}

#[e2e::test]
async fn set_role_admin_succeeds_with_new_admin_role(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let receipt =
        receipt!(contract.setRoleAdmin(ROLE.into(), NEW_ADMIN_ROLE.into()))?;
    assert!(receipt.emits(RoleAdminChanged {
        role: ROLE.into(),
        previousAdminRole: DEFAULT_ADMIN_ROLE.into(),
        newAdminRole: NEW_ADMIN_ROLE.into()
    }));

    let AccessControl::getRoleAdminReturn { role } =
        contract.getRoleAdmin(ROLE.into()).call().await?;
    assert_eq!(*role, NEW_ADMIN_ROLE);

    Ok(())
}

#[e2e::test]
async fn role_admin_change_succeeds_with_new_admin_granting_role(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let receipt =
        receipt!(contract.setRoleAdmin(ROLE.into(), NEW_ADMIN_ROLE.into()))?;
    assert!(receipt.emits(RoleAdminChanged {
        role: ROLE.into(),
        previousAdminRole: DEFAULT_ADMIN_ROLE.into(),
        newAdminRole: NEW_ADMIN_ROLE.into()
    }));

    let _ = watch!(contract.grantRole(NEW_ADMIN_ROLE.into(), bob_addr))?;

    let contract = AccessControl::new(contract_addr, &bob.wallet);
    let receipt = receipt!(contract.grantRole(ROLE.into(), alice_addr))?;
    assert!(receipt.emits(RoleGranted {
        role: ROLE.into(),
        account: alice_addr,
        sender: bob_addr
    }));

    Ok(())
}

#[e2e::test]
async fn role_admin_change_succeeds_with_new_admin_revoking_role(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let receipt =
        receipt!(contract.setRoleAdmin(ROLE.into(), NEW_ADMIN_ROLE.into()))?;
    assert!(receipt.emits(RoleAdminChanged {
        role: ROLE.into(),
        previousAdminRole: DEFAULT_ADMIN_ROLE.into(),
        newAdminRole: NEW_ADMIN_ROLE.into()
    }));

    let _ = watch!(contract.grantRole(NEW_ADMIN_ROLE.into(), bob_addr))?;

    let contract = AccessControl::new(contract_addr, &bob.wallet);
    let _ = watch!(contract.grantRole(ROLE.into(), alice_addr))?;
    let receipt = receipt!(contract.revokeRole(ROLE.into(), alice_addr))?;
    assert!(receipt.emits(RoleRevoked {
        role: ROLE.into(),
        account: alice_addr,
        sender: bob_addr
    }));

    Ok(())
}

#[e2e::test]
async fn role_admin_change_reverts_when_previous_admin_attempts_to_grant_role(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let receipt =
        receipt!(contract.setRoleAdmin(ROLE.into(), NEW_ADMIN_ROLE.into()))?;
    assert!(receipt.emits(RoleAdminChanged {
        role: ROLE.into(),
        previousAdminRole: DEFAULT_ADMIN_ROLE.into(),
        newAdminRole: NEW_ADMIN_ROLE.into()
    }));

    let err = send!(contract.grantRole(ROLE.into(), bob_addr))
        .expect_err("previous admins can't grant roles after admin change");
    assert!(err.reverted_with(AccessControlUnauthorizedAccount {
        account: alice_addr,
        neededRole: NEW_ADMIN_ROLE.into()
    }));

    Ok(())
}

#[e2e::test]
async fn role_admin_change_reverts_when_previous_admin_attempts_to_revoke_role(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let receipt =
        receipt!(contract.setRoleAdmin(ROLE.into(), NEW_ADMIN_ROLE.into()))?;
    assert!(receipt.emits(RoleAdminChanged {
        role: ROLE.into(),
        previousAdminRole: DEFAULT_ADMIN_ROLE.into(),
        newAdminRole: NEW_ADMIN_ROLE.into()
    }));

    let err = send!(contract.revokeRole(ROLE.into(), bob_addr))
        .expect_err("previous admins can't revoke roles after admin change");
    assert!(err.reverted_with(AccessControlUnauthorizedAccount {
        account: alice_addr,
        neededRole: NEW_ADMIN_ROLE.into()
    }));

    Ok(())
}
