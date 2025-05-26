#![cfg(feature = "e2e")]

use access_control_enumerable_example::{MINTER_ROLE, BURNER_ROLE};
use alloy_primitives::{Address, U256};
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt};
use eyre::Result;
use openzeppelin_stylus::access::control::AccessControl;

// ============================================================================
// Integration Tests: AccessControlEnumerable Example
// ============================================================================

const NAME: &str = "Test Token";
const SYMBOL: &str = "TEST";

#[e2e::test]
async fn happy_path_minter_role(
    alice: Account,
    bob: Account,
    charlie: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer()
        .args(NAME, SYMBOL)
        .deploy()
        .await?
        .address()?;
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &alice.wallet,
    );

    // Initially no minters
    let minters = contract.get_minters().call().await?;
    assert!(minters.is_empty());

    // Add Bob and Charlie as minters
    watch!(contract.add_minter(bob.address()))?;
    watch!(contract.add_minter(charlie.address()))?;

    // Verify minters list
    let minters = contract.get_minters().call().await?;
    assert_eq!(minters.len(), 2);
    assert!(minters.contains(&bob.address()));
    assert!(minters.contains(&charlie.address()));

    // Bob can mint tokens
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &bob.wallet,
    );
    watch!(contract.mint(bob.address(), U256::from(1000)))?;

    // Remove Bob as minter
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &alice.wallet,
    );
    watch!(contract.remove_minter(bob.address()))?;

    // Verify updated minters list
    let minters = contract.get_minters().call().await?;
    assert_eq!(minters.len(), 1);
    assert!(!minters.contains(&bob.address()));
    assert!(minters.contains(&charlie.address()));

    Ok(())
}

#[e2e::test]
async fn happy_path_burner_role(
    alice: Account,
    bob: Account,
    charlie: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer()
        .args(NAME, SYMBOL)
        .deploy()
        .await?
        .address()?;
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &alice.wallet,
    );

    // Initially no burners
    let burners = contract.get_burners().call().await?;
    assert!(burners.is_empty());

    // Add Bob as burner
    watch!(contract.add_burner(bob.address()))?;

    // Verify burners list
    let burners = contract.get_burners().call().await?;
    assert_eq!(burners.len(), 1);
    assert!(burners.contains(&bob.address()));

    // First mint some tokens to burn
    watch!(contract.add_minter(alice.address()))?;
    watch!(contract.mint(bob.address(), U256::from(1000)))?;

    // Bob can burn tokens
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &bob.wallet,
    );
    watch!(contract.burn(bob.address(), U256::from(500)))?;

    // Remove Bob as burner
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &alice.wallet,
    );
    watch!(contract.remove_burner(bob.address()))?;

    // Verify updated burners list
    let burners = contract.get_burners().call().await?;
    assert!(burners.is_empty());

    Ok(())
}

#[e2e::test]
async fn happy_path_multiple_roles(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer()
        .args(NAME, SYMBOL)
        .deploy()
        .await?
        .address()?;
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &alice.wallet,
    );

    // Add Bob as both minter and burner
    watch!(contract.add_minter(bob.address()))?;
    watch!(contract.add_burner(bob.address()))?;

    // Verify both role lists
    let minters = contract.get_minters().call().await?;
    let burners = contract.get_burners().call().await?;
    assert_eq!(minters.len(), 1);
    assert_eq!(burners.len(), 1);
    assert!(minters.contains(&bob.address()));
    assert!(burners.contains(&bob.address()));

    // Bob can both mint and burn
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &bob.wallet,
    );
    watch!(contract.mint(bob.address(), U256::from(1000)))?;
    watch!(contract.burn(bob.address(), U256::from(500)))?;

    Ok(())
}

#[e2e::test]
async fn test_unauthorized_access(
    alice: Account,
    bob: Account,
    charlie: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer()
        .args(NAME, SYMBOL)
        .deploy()
        .await?
        .address()?;
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &bob.wallet,
    );

    // Bob (non-admin) tries to add Charlie as minter
    let result = contract.add_minter(charlie.address()).call().await;
    assert!(result.is_err(), "Non-admin should not be able to add minters");

    // Bob tries to mint tokens without minter role
    let result = contract.mint(bob.address(), U256::from(1000)).call().await;
    assert!(result.is_err(), "Account without minter role should not be able to mint");

    Ok(())
}

#[e2e::test]
async fn test_role_enumeration_edge_cases(
    alice: Account,
    bob: Account,
    charlie: Account,
    dave: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer()
        .args(NAME, SYMBOL)
        .deploy()
        .await?
        .address()?;
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &alice.wallet,
    );

    // Initially no minters or burners
    let minters = contract.get_minters().call().await?;
    let burners = contract.get_burners().call().await?;
    assert!(minters.is_empty(), "Should start with no minters");
    assert!(burners.is_empty(), "Should start with no burners");

    // Add multiple minters
    watch!(contract.add_minter(bob.address()))?;
    watch!(contract.add_minter(charlie.address()))?;
    watch!(contract.add_minter(dave.address()))?;

    // Verify all minters are listed
    let minters = contract.get_minters().call().await?;
    assert_eq!(minters.len(), 3, "Should have three minters");
    assert!(minters.contains(&bob.address()), "Bob should be a minter");
    assert!(minters.contains(&charlie.address()), "Charlie should be a minter");
    assert!(minters.contains(&dave.address()), "Dave should be a minter");

    // Remove minters and verify list updates
    watch!(contract.remove_minter(charlie.address()))?;
    let minters = contract.get_minters().call().await?;
    assert_eq!(minters.len(), 2, "Should have two minters after removal");
    assert!(!minters.contains(&charlie.address()), "Charlie should no longer be a minter");

    // Try to remove a non-existent minter (should not error)
    watch!(contract.remove_minter(charlie.address()))?;
    let minters = contract.get_minters().call().await?;
    assert_eq!(minters.len(), 2, "Minter count should not change when removing non-existent minter");

    Ok(())
}

#[e2e::test]
async fn test_role_management(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer()
        .args(NAME, SYMBOL)
        .deploy()
        .await?
        .address()?;
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &alice.wallet,
    );

    // Add Bob as minter
    watch!(contract.add_minter(bob.address()))?;

    // Adding same account twice should not affect enumeration
    watch!(contract.add_minter(bob.address()))?;
    let minters = contract.get_minters().call().await?;
    assert_eq!(minters.len(), 1, "Adding same minter twice should not create duplicate");

    // Bob should be able to mint
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &bob.wallet,
    );
    watch!(contract.mint(bob.address(), U256::from(1000)))?;

    // Remove Bob's minter role
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &alice.wallet,
    );
    watch!(contract.remove_minter(bob.address()))?;

    // Bob should no longer be able to mint
    let contract = access_control_enumerable_example::AccessControlEnumerableExample::new(
        contract_addr,
        &bob.wallet,
    );
    let result = contract.mint(bob.address(), U256::from(1000)).call().await;
    assert!(result.is_err(), "Should not be able to mint after role revocation");

    Ok(())
}
