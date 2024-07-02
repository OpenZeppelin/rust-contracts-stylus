#![cfg(feature = "e2e")]

use abi::AccessControl::{
    self, AccessControlBadConfirmation, AccessControlUnauthorizedAccount,
    RoleAdminChanged, RoleGranted, RoleRevoked,
};
use alloy::{hex, primitives::Address, sol_types::SolConstructor};
use e2e::{receipt, send, watch, Account, EventExt, Revert};
use eyre::Result;

mod abi;

const DEFAULT_ADMIN_ROLE: [u8; 32] =
    openzeppelin_stylus::access::control::AccessControl::DEFAULT_ADMIN_ROLE;
const ROLE: [u8; 32] = access_control_example::TRANSFER_ROLE;
const NEW_ADMIN_ROLE: [u8; 32] =
    hex!("879ce0d4bfd332649ca3552efe772a38d64a315eb70ab69689fd309c735946b5");

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
    assert_eq!(hasRole, true);

    Ok(())
}

#[e2e::test]
async fn other_roles_admin_is_the_default_admin_role(
    alice: Account,
) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let AccessControl::getRoleAdminReturn { role } =
        contract.getRoleAdmin(ROLE.into()).call().await?;
    assert_eq!(*role, DEFAULT_ADMIN_ROLE);

    Ok(())
}

#[e2e::test]
async fn default_role_is_default_admin(alice: Account) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
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
async fn non_admin_cannot_grant_role(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
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
async fn accounts_can_be_granted_roles_multiple_times(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
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
async fn not_granted_roles_can_be_revoked(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr = deploy(&alice).await?;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let AccessControl::hasRoleReturn { hasRole } =
        contract.hasRole(ROLE.into(), alice_addr).call().await?;
    assert_eq!(hasRole, false);

    let receipt = receipt!(contract.revokeRole(ROLE.into(), alice_addr))?;
    assert!(!receipt.emits(RoleRevoked {
        role: ROLE.into(),
        account: alice_addr,
        sender: alice_addr
    }));

    Ok(())
}

#[e2e::test]
async fn admin_can_revoke_role(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
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
async fn non_admin_cannot_revoke_role(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
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
async fn roles_can_be_revoked_multiple_times(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
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
async fn not_granted_roles_can_be_renounced(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr = deploy(&alice).await?;
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
async fn bearer_can_renounce_role(alice: Account, bob: Account) -> Result<()> {
    let bob_addr = bob.address();
    let contract_addr = deploy(&alice).await?;
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
async fn only_sender_can_renounce(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
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
async fn roles_can_be_renounced_multiple_times(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr = deploy(&alice).await?;
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
async fn a_roles_admin_role_can_change(alice: Account) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
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
async fn the_new_admin_can_grant_roles(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
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
async fn the_new_admin_can_revoke_roles(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
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
async fn previous_admins_no_longer_grant_roles(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
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
async fn previous_admins_no_longer_revoke_roles(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
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
