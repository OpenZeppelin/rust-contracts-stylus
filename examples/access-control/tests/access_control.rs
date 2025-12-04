#![cfg(feature = "e2e")]

use abi::AccessControl::{
    self, AccessControlBadConfirmation, AccessControlUnauthorizedAccount,
    RoleAdminChanged, RoleGranted, RoleRevoked,
};
use alloy::hex;
use e2e::{
    constructor, receipt, send, watch, Account, Constructor, EventExt, Revert,
    DEPLOYER_ADDRESS,
};
use eyre::Result;
use stylus_sdk::alloy_primitives::{Address, U256};

mod abi;

const DEFAULT_ADMIN_ROLE: [u8; 32] =
    openzeppelin_stylus::access::control::AccessControl::DEFAULT_ADMIN_ROLE;
const ROLE: [u8; 32] = access_control_example::TRANSFER_ROLE;
const NEW_ADMIN_ROLE: [u8; 32] =
    hex!("879ce0d4bfd332649ca3552efe772a38d64a315eb70ab69689fd309c735946b5");

fn ctr(admin: Address) -> Constructor {
    constructor!(admin)
}

// ============================================================================
// Integration Tests: AccessControl
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let receipt = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?;
    let contract = AccessControl::new(receipt.contract_address, &alice.wallet);

    // `StylusDeployer` is the message sender, so currently it becomes the
    // `sender` in the `RoleGranted` event
    let stylus_deployer = std::env::var(DEPLOYER_ADDRESS)?.parse()?;

    assert!(receipt.emits(RoleGranted {
        role: DEFAULT_ADMIN_ROLE.into(),
        account: alice_addr,
        sender: stylus_deployer
    }));

    let AccessControl::hasRoleReturn { hasRole } =
        contract.hasRole(DEFAULT_ADMIN_ROLE.into(), alice_addr).call().await?;
    assert!(hasRole);

    let AccessControl::getRoleMemberReturn { member } = contract
        .getRoleMember(DEFAULT_ADMIN_ROLE.into(), U256::ZERO)
        .call()
        .await?;
    assert_eq!(member, alice_addr);

    let AccessControl::getRoleMemberCountReturn { count } =
        contract.getRoleMemberCount(DEFAULT_ADMIN_ROLE.into()).call().await?;
    assert_eq!(count, U256::ONE);

    let AccessControl::getRoleMembersReturn { members } =
        contract.getRoleMembers(DEFAULT_ADMIN_ROLE.into()).call().await?;
    assert_eq!(members, vec![alice_addr]);

    Ok(())
}

#[e2e::test]
async fn other_roles_admin_is_the_default_admin_role(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let AccessControl::getRoleAdminReturn { role } =
        contract.getRoleAdmin(ROLE.into()).call().await?;
    assert_eq!(*role, DEFAULT_ADMIN_ROLE);

    Ok(())
}

#[e2e::test]
async fn default_role_is_default_admin(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
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
async fn error_when_non_admin_grants_role(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
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
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
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

    let AccessControl::getRoleMemberReturn { member } =
        contract.getRoleMember(ROLE.into(), U256::ZERO).call().await?;
    assert_eq!(member, bob_addr);

    let AccessControl::getRoleMemberCountReturn { count } =
        contract.getRoleMemberCount(ROLE.into()).call().await?;
    assert_eq!(count, U256::ONE);

    let AccessControl::getRoleMembersReturn { members } =
        contract.getRoleMembers(ROLE.into()).call().await?;
    assert_eq!(members, vec![bob_addr]);

    Ok(())
}

#[e2e::test]
async fn not_granted_roles_can_be_revoked(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
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
async fn admin_can_revoke_role(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    watch!(contract.grantRole(ROLE.into(), bob_addr))?;

    let receipt = receipt!(contract.revokeRole(ROLE.into(), bob_addr))?;
    assert!(receipt.emits(RoleRevoked {
        role: ROLE.into(),
        account: bob_addr,
        sender: alice_addr
    }));

    let AccessControl::getRoleMemberCountReturn { count } =
        contract.getRoleMemberCount(ROLE.into()).call().await?;
    assert!(count.is_zero());

    let AccessControl::getRoleMembersReturn { members } =
        contract.getRoleMembers(ROLE.into()).call().await?;
    assert!(members.is_empty());

    Ok(())
}

#[e2e::test]
async fn error_when_non_admin_revokes_role(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    watch!(contract.grantRole(ROLE.into(), alice_addr))?;

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
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    watch!(contract.revokeRole(ROLE.into(), bob_addr))?;
    let receipt = receipt!(contract.revokeRole(ROLE.into(), bob_addr))?;
    assert!(!receipt.emits(RoleRevoked {
        role: ROLE.into(),
        account: bob_addr,
        sender: alice_addr
    }));

    let AccessControl::getRoleMemberCountReturn { count } =
        contract.getRoleMemberCount(ROLE.into()).call().await?;
    assert!(count.is_zero());

    let AccessControl::getRoleMembersReturn { members } =
        contract.getRoleMembers(ROLE.into()).call().await?;
    assert!(members.is_empty());

    Ok(())
}

#[e2e::test]
async fn not_granted_roles_can_be_renounced(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
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
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    watch!(contract.grantRole(ROLE.into(), bob_addr))?;

    let contract = AccessControl::new(contract_addr, &bob.wallet);
    let receipt = receipt!(contract.renounceRole(ROLE.into(), bob_addr))?;
    assert!(receipt.emits(RoleRevoked {
        role: ROLE.into(),
        account: bob_addr,
        sender: bob_addr
    }));

    let AccessControl::getRoleMemberCountReturn { count } =
        contract.getRoleMemberCount(ROLE.into()).call().await?;
    assert!(count.is_zero());

    let AccessControl::getRoleMembersReturn { members } =
        contract.getRoleMembers(ROLE.into()).call().await?;
    assert!(members.is_empty());
    Ok(())
}

#[e2e::test]
async fn error_when_the_one_renouncing_is_not_the_sender(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    watch!(contract.grantRole(ROLE.into(), bob_addr))?;

    let contract = AccessControl::new(contract_addr, &bob.wallet);
    let err = send!(contract.renounceRole(ROLE.into(), alice_addr))
        .expect_err("only sender should be able to renounce");
    assert!(err.reverted_with(AccessControlBadConfirmation {}));

    Ok(())
}

#[e2e::test]
async fn roles_can_be_renounced_multiple_times(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
    let contract = AccessControl::new(contract_addr, &alice.wallet);

    watch!(contract.renounceRole(ROLE.into(), alice_addr))?;
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
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
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
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
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

    watch!(contract.grantRole(NEW_ADMIN_ROLE.into(), bob_addr))?;

    let contract = AccessControl::new(contract_addr, &bob.wallet);
    let receipt = receipt!(contract.grantRole(ROLE.into(), alice_addr))?;
    assert!(receipt.emits(RoleGranted {
        role: ROLE.into(),
        account: alice_addr,
        sender: bob_addr
    }));

    let AccessControl::getRoleMemberReturn { member } =
        contract.getRoleMember(ROLE.into(), U256::ZERO).call().await?;
    assert_eq!(member, alice_addr);

    let AccessControl::getRoleMemberCountReturn { count } =
        contract.getRoleMemberCount(ROLE.into()).call().await?;
    assert_eq!(count, U256::ONE);

    let AccessControl::getRoleMembersReturn { members } =
        contract.getRoleMembers(ROLE.into()).call().await?;
    assert_eq!(members, vec![alice_addr]);

    Ok(())
}

#[e2e::test]
async fn the_new_admin_can_revoke_roles(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
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

    watch!(contract.grantRole(NEW_ADMIN_ROLE.into(), bob_addr))?;

    let contract = AccessControl::new(contract_addr, &bob.wallet);
    watch!(contract.grantRole(ROLE.into(), alice_addr))?;
    let receipt = receipt!(contract.revokeRole(ROLE.into(), alice_addr))?;
    assert!(receipt.emits(RoleRevoked {
        role: ROLE.into(),
        account: alice_addr,
        sender: bob_addr
    }));

    let AccessControl::getRoleMemberCountReturn { count } =
        contract.getRoleMemberCount(ROLE.into()).call().await?;
    assert!(count.is_zero());

    let AccessControl::getRoleMembersReturn { members } =
        contract.getRoleMembers(ROLE.into()).call().await?;
    assert!(members.is_empty());

    Ok(())
}

#[e2e::test]
async fn error_when_previous_admin_grants_roles(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
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
async fn error_when_previous_admin_revokes_roles(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice.address()))
        .deploy()
        .await?
        .contract_address;
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
