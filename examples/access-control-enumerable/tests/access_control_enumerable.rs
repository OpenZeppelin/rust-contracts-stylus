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