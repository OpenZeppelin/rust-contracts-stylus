#![cfg(feature = "e2e")]

use abi::Erc20;
use alloy::{
    primitives::{uint, Address, U256},
    sol,
};
use e2e::{
    receipt, send, watch, Account, EventExt, Panic, PanicCode, ReceiptExt,
    Revert,
};
use eyre::Result;

use crate::Erc20Example::constructorCall;

mod abi;

sol!("src/constructor.sol");

const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "TTK";
const CAP: U256 = uint!(1_000_000_U256);

impl Default for constructorCall {
    fn default() -> Self {
        ctr(CAP)
    }
}

fn ctr(cap: U256) -> constructorCall {
    Erc20Example::constructorCall {
        name_: TOKEN_NAME.to_owned(),
        symbol_: TOKEN_SYMBOL.to_owned(),
        cap_: cap,
    }
}

// ============================================================================
// Integration Tests: ERC-20 Token + Metadata Extension
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20::new(contract_addr, &alice.wallet);

    let Erc20::nameReturn { name } = contract.name().call().await?;
    let Erc20::symbolReturn { symbol } = contract.symbol().call().await?;
    let Erc20::capReturn { cap } = contract.cap().call().await?;
    let Erc20::decimalsReturn { decimals } = contract.decimals().call().await?;
    let Erc20::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(name, TOKEN_NAME.to_owned());
    assert_eq!(symbol, TOKEN_SYMBOL.to_owned());
    assert_eq!(cap, CAP);
    assert_eq!(decimals, 10);
    assert_eq!(total_supply, U256::ZERO);
    Ok(())
}

#[e2e::test]
async fn mints(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let Erc20::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(U256::ZERO, initial_balance);
    assert_eq!(U256::ZERO, initial_supply);

    let one = uint!(1_U256);
    let receipt = receipt!(contract.mint(alice_addr, one))?;
    assert!(receipt.emits(Erc20::Transfer {
        from: Address::ZERO,
        to: alice_addr,
        value: one,
    }));

    let Erc20::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(initial_balance + one, balance);
    assert_eq!(initial_supply + one, total_supply);
    Ok(())
}

#[e2e::test]
async fn mints_rejects_invalid_receiver(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20::new(contract_addr, &alice.wallet);
    let invalid_receiver = Address::ZERO;

    let Erc20::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(invalid_receiver).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await?;

    let value = uint!(10_U256);
    let err = send!(contract.mint(invalid_receiver, value))
        .expect_err("should not mint tokens for Address::ZERO");
    assert!(err.reverted_with(Erc20::ERC20InvalidReceiver {
        receiver: invalid_receiver
    }));

    let Erc20::balanceOfReturn { balance } =
        contract.balanceOf(invalid_receiver).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(initial_balance, balance);
    assert_eq!(initial_supply, total_supply);
    Ok(())
}

#[e2e::test]
async fn mints_rejects_overflow(alice: Account) -> Result<()> {
    let max_cap = U256::MAX;

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(max_cap))
        .deploy()
        .await?
        .address()?;
    let contract = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let one = uint!(1_U256);

    let _ = watch!(contract.mint(alice_addr, max_cap))?;

    let Erc20::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(initial_supply, max_cap);
    assert_eq!(initial_balance, max_cap);

    let err = send!(contract.mint(alice_addr, one))
        .expect_err("should not exceed U256::MAX");

    assert!(err.panicked_with(PanicCode::ArithmeticOverflow));

    let Erc20::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(initial_balance, balance);
    assert_eq!(initial_supply, total_supply);
    Ok(())
}

#[e2e::test]
async fn transfers(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let receipt = receipt!(contract_alice.transfer(bob_addr, value))?;

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;

    assert!(receipt.emits(Erc20::Transfer {
        from: alice_addr,
        to: bob_addr,
        value
    }));

    assert_eq!(initial_alice_balance - value, alice_balance);
    assert_eq!(initial_bob_balance + value, bob_balance);
    assert_eq!(initial_supply, supply);

    Ok(())
}

#[e2e::test]
async fn transfer_rejects_insufficient_balance(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(11_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let err = send!(contract_alice.transfer(bob_addr, value))
        .expect_err("should not transfer when insufficient balance");
    assert!(err.reverted_with(Erc20::ERC20InsufficientBalance {
        sender: alice_addr,
        balance,
        needed: value
    }));

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_supply, supply);

    Ok(())
}

#[e2e::test]
async fn transfer_rejects_invalid_receiver(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let invalid_receiver = Address::ZERO;

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_receiver_balance } =
        contract_alice.balanceOf(invalid_receiver).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let err = send!(contract_alice.transfer(invalid_receiver, value))
        .expect_err("should not transfer to Address::ZERO");
    assert!(err.reverted_with(Erc20::ERC20InvalidReceiver {
        receiver: invalid_receiver
    }));

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: receiver_balance } =
        contract_alice.balanceOf(invalid_receiver).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_receiver_balance, receiver_balance);
    assert_eq!(initial_supply, supply);

    Ok(())
}

#[e2e::test]
async fn approves(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let one = uint!(1_U256);
    let ten = uint!(10_U256);

    let Erc20::allowanceReturn { allowance: initial_alice_bob_allowance } =
        contract.allowance(alice_addr, bob_addr).call().await?;
    let Erc20::allowanceReturn { allowance: initial_bob_alice_allowance } =
        contract.allowance(bob_addr, alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_bob_balance } =
        contract.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(U256::ZERO, initial_alice_bob_allowance);
    assert_eq!(U256::ZERO, initial_bob_alice_allowance);

    let receipt = receipt!(contract.approve(bob_addr, one))?;
    assert!(receipt.emits(Erc20::Approval {
        owner: alice_addr,
        spender: bob_addr,
        value: one,
    }));

    let Erc20::allowanceReturn { allowance: alice_bob_allowance } =
        contract.allowance(alice_addr, bob_addr).call().await?;
    let Erc20::allowanceReturn { allowance: bob_alice_allowance } =
        contract.allowance(bob_addr, alice_addr).call().await?;

    assert_eq!(initial_alice_bob_allowance + one, alice_bob_allowance);
    assert_eq!(initial_bob_alice_allowance, bob_alice_allowance);

    let receipt = receipt!(contract.approve(bob_addr, ten))?;
    assert!(receipt.emits(Erc20::Approval {
        owner: alice_addr,
        spender: bob_addr,
        value: ten,
    }));

    let Erc20::allowanceReturn { allowance: alice_bob_allowance } =
        contract.allowance(alice_addr, bob_addr).call().await?;
    let Erc20::allowanceReturn { allowance: bob_alice_allowance } =
        contract.allowance(bob_addr, alice_addr).call().await?;

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract.totalSupply().call().await?;

    assert_eq!(initial_alice_bob_allowance + ten, alice_bob_allowance);
    assert_eq!(initial_bob_alice_allowance, bob_alice_allowance);
    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_supply, supply);

    Ok(())
}

#[e2e::test]
async fn approve_rejects_invalid_spender(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let invalid_spender = Address::ZERO;

    let ten = uint!(10_U256);

    let Erc20::allowanceReturn { allowance: initial_alice_spender_allowance } =
        contract.allowance(alice_addr, invalid_spender).call().await?;
    let Erc20::allowanceReturn { allowance: initial_spender_alice_allowance } =
        contract.allowance(invalid_spender, alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_spender_balance } =
        contract.balanceOf(invalid_spender).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(U256::ZERO, initial_alice_spender_allowance);
    assert_eq!(U256::ZERO, initial_spender_alice_allowance);

    let err = send!(contract.approve(invalid_spender, ten))
        .expect_err("should not approve for Address::ZERO");

    assert!(err.reverted_with(Erc20::ERC20InvalidSpender {
        spender: invalid_spender
    }));

    let Erc20::allowanceReturn { allowance: alice_spender_allowance } =
        contract.allowance(alice_addr, invalid_spender).call().await?;
    let Erc20::allowanceReturn { allowance: spender_alice_allowance } =
        contract.allowance(invalid_spender, alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: spender_balance } =
        contract.balanceOf(invalid_spender).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract.totalSupply().call().await?;

    assert_eq!(initial_alice_spender_allowance, alice_spender_allowance);
    assert_eq!(initial_spender_alice_allowance, spender_alice_allowance);
    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_spender_balance, spender_balance);
    assert_eq!(initial_supply, supply);

    Ok(())
}

#[e2e::test]
async fn transfers_from(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let contract_bob = Erc20::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let _ = watch!(contract_alice.approve(bob_addr, balance))?;

    let Erc20::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    let receipt =
        receipt!(contract_bob.transferFrom(alice_addr, bob_addr, value))?;

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;
    let Erc20::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert!(receipt.emits(Erc20::Transfer {
        from: alice_addr,
        to: bob_addr,
        value
    }));

    assert_eq!(initial_alice_balance - value, alice_balance);
    assert_eq!(initial_bob_balance + value, bob_balance);
    assert_eq!(initial_supply, supply);
    assert_eq!(initial_allowance - value, allowance);

    Ok(())
}

#[e2e::test]
async fn transfer_from_reverts_insufficient_balance(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let contract_bob = Erc20::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(1_U256);
    let value = uint!(10_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let _ = watch!(contract_alice.approve(bob_addr, value))?;

    let Erc20::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    let err = send!(contract_bob.transferFrom(alice_addr, bob_addr, value))
        .expect_err("should not transfer when insufficient balance");

    assert!(err.reverted_with(Erc20::ERC20InsufficientBalance {
        sender: alice_addr,
        balance,
        needed: value
    }));

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;
    let Erc20::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_supply, supply);
    assert_eq!(initial_allowance, allowance);

    Ok(())
}

#[e2e::test]
async fn transfer_from_rejects_insufficient_allowance(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let contract_bob = Erc20::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let Erc20::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert_eq!(initial_allowance, U256::ZERO);

    let err = send!(contract_bob.transferFrom(alice_addr, bob_addr, value))
        .expect_err("should not transfer when insufficient allowance");

    assert!(err.reverted_with(Erc20::ERC20InsufficientAllowance {
        spender: bob_addr,
        allowance: U256::ZERO,
        needed: value
    }));

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;
    let Erc20::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_supply, supply);
    assert_eq!(initial_allowance, allowance);

    Ok(())
}

#[e2e::test]
async fn transfer_from_rejects_invalid_receiver(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let contract_bob = Erc20::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let invalid_receiver = Address::ZERO;

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_receiver_balance } =
        contract_alice.balanceOf(invalid_receiver).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let _ = watch!(contract_alice.approve(bob_addr, balance))?;

    let Erc20::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    let err =
        send!(contract_bob.transferFrom(alice_addr, invalid_receiver, value))
            .expect_err("should not transfer to Address::ZERO");

    assert!(err.reverted_with(Erc20::ERC20InvalidReceiver {
        receiver: invalid_receiver
    }));

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: receiver_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;
    let Erc20::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_receiver_balance, receiver_balance);
    assert_eq!(initial_supply, supply);
    assert_eq!(initial_allowance, allowance);

    Ok(())
}

// ============================================================================
// Integration Tests: ERC-20 Burnable Extension
// ============================================================================

#[e2e::test]
async fn burns(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let receipt = receipt!(contract_alice.burn(value))?;

    let Erc20::balanceOfReturn { balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;

    assert!(receipt.emits(Erc20::Transfer {
        from: alice_addr,
        to: Address::ZERO,
        value,
    }));

    assert_eq!(initial_balance - value, balance);
    assert_eq!(initial_supply - value, supply);

    Ok(())
}

#[e2e::test]
async fn burn_rejects_insufficient_balance(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let balance = uint!(10_U256);
    let value = uint!(11_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let err = send!(contract_alice.burn(value))
        .expect_err("should not burn when insufficient balance");
    assert!(err.reverted_with(Erc20::ERC20InsufficientBalance {
        sender: alice_addr,
        balance,
        needed: value
    }));

    let Erc20::balanceOfReturn { balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;

    assert_eq!(initial_balance, balance);
    assert_eq!(initial_supply, supply);

    Ok(())
}

#[e2e::test]
async fn burns_from(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let contract_bob = Erc20::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let _ = watch!(contract_alice.approve(bob_addr, balance))?;

    let Erc20::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    let receipt = receipt!(contract_bob.burnFrom(alice_addr, value))?;

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;
    let Erc20::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert!(receipt.emits(Erc20::Transfer {
        from: alice_addr,
        to: Address::ZERO,
        value,
    }));

    assert_eq!(initial_alice_balance - value, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_supply - value, supply);
    assert_eq!(initial_allowance - value, allowance);

    Ok(())
}

#[e2e::test]
async fn burn_from_reverts_insufficient_balance(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let contract_bob = Erc20::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(1_U256);
    let value = uint!(10_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let _ = watch!(contract_alice.approve(bob_addr, value))?;

    let Erc20::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    let err = send!(contract_bob.burnFrom(alice_addr, value))
        .expect_err("should not burn when insufficient balance");

    assert!(err.reverted_with(Erc20::ERC20InsufficientBalance {
        sender: alice_addr,
        balance,
        needed: value
    }));

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;
    let Erc20::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_supply, supply);
    assert_eq!(initial_allowance, allowance);

    Ok(())
}

#[e2e::test]
async fn burn_from_rejects_insufficient_allowance(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let contract_bob = Erc20::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let Erc20::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert_eq!(initial_allowance, U256::ZERO);

    let err = send!(contract_bob.burnFrom(alice_addr, value))
        .expect_err("should not burn when insufficient allowance");

    assert!(err.reverted_with(Erc20::ERC20InsufficientAllowance {
        spender: bob_addr,
        allowance: U256::ZERO,
        needed: value
    }));

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;
    let Erc20::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_supply, supply);
    assert_eq!(initial_allowance, allowance);

    Ok(())
}

// ============================================================================
// Integration Tests: ERC-20 Capped Extension
// ============================================================================

#[e2e::test]
async fn mint_rejects_exceeding_cap(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let one = uint!(1_U256);
    let two = uint!(2_U256);
    let cap = CAP;
    let balance = cap - one;

    let _ = watch!(contract_alice.mint(alice_addr, balance))?;

    let Erc20::balanceOfReturn { balance: initial_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let err = send!(contract_alice.mint(alice_addr, two))
        .expect_err("should not mint when exceeding the cap");
    assert!(err.reverted_with(Erc20::ERC20ExceededCap {
        increased_supply: balance + two,
        cap
    }));

    let Erc20::balanceOfReturn { balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;

    assert_eq!(initial_balance, balance);
    assert_eq!(initial_supply, supply);

    Ok(())
}

#[e2e::test]
async fn mint_rejects_when_cap_reached(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let one = uint!(1_U256);
    let cap = CAP;
    let balance = cap;

    let _ = watch!(contract_alice.mint(alice_addr, balance))?;

    let Erc20::balanceOfReturn { balance: initial_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let err = send!(contract_alice.mint(alice_addr, one))
        .expect_err("should not mint when the cap is reached");
    assert!(err.reverted_with(Erc20::ERC20ExceededCap {
        increased_supply: balance + one,
        cap
    }));

    let Erc20::balanceOfReturn { balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;

    assert_eq!(initial_balance, balance);
    assert_eq!(initial_supply, supply);

    Ok(())
}

#[e2e::test]
async fn should_not_deploy_capped_with_invalid_cap(
    alice: Account,
) -> Result<()> {
    let invalid_cap = U256::ZERO;
    let err = alice
        .as_deployer()
        .with_constructor(ctr(invalid_cap))
        .deploy()
        .await
        .expect_err("should not deploy due to `ERC20InvalidCap`");

    assert!(err.reverted_with(Erc20::ERC20InvalidCap { cap: invalid_cap }));

    Ok(())
}

// ============================================================================
// Integration Tests: ERC-20 Pausable Extension
// ============================================================================

#[e2e::test]
async fn pauses(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20::new(contract_addr, &alice.wallet);

    let receipt = receipt!(contract.pause())?;

    assert!(receipt.emits(Erc20::Paused { account: alice.address() }));

    let Erc20::pausedReturn { paused } = contract.paused().call().await?;

    assert_eq!(true, paused);

    Ok(())
}

#[e2e::test]
async fn unpauses(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20::new(contract_addr, &alice.wallet);

    let _ = watch!(contract.pause())?;

    let receipt = receipt!(contract.unpause())?;

    assert!(receipt.emits(Erc20::Unpaused { account: alice.address() }));

    let Erc20::pausedReturn { paused } = contract.paused().call().await?;

    assert_eq!(false, paused);

    Ok(())
}

#[e2e::test]
async fn error_when_burn_in_paused_state(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await?;

    let _ = watch!(contract.pause())?;

    let err =
        send!(contract.burn(value)).expect_err("should return EnforcedPause");
    assert!(err.reverted_with(Erc20::EnforcedPause {}));

    let Erc20::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract.totalSupply().call().await?;

    assert_eq!(initial_balance, balance);
    assert_eq!(initial_supply, supply);

    Ok(())
}

#[e2e::test]
async fn error_when_burn_from_in_paused_state(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let contract_bob = Erc20::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let _ = watch!(contract_alice.approve(bob_addr, balance))?;

    let Erc20::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    let _ = watch!(contract_alice.pause())?;

    let err = send!(contract_bob.burnFrom(alice_addr, value))
        .expect_err("should return EnforcedPause");
    assert!(err.reverted_with(Erc20::EnforcedPause {}));

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;
    let Erc20::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_supply, supply);
    assert_eq!(initial_allowance, allowance);

    Ok(())
}

#[e2e::test]
async fn error_when_mint_in_paused_state(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let Erc20::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(U256::ZERO, initial_balance);
    assert_eq!(U256::ZERO, initial_supply);

    let _ = watch!(contract.pause())?;

    let err = send!(contract.mint(alice_addr, uint!(1_U256)))
        .expect_err("should return EnforcedPause");
    assert!(err.reverted_with(Erc20::EnforcedPause {}));

    let Erc20::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(initial_balance, balance);
    assert_eq!(initial_supply, total_supply);
    Ok(())
}

#[e2e::test]
async fn error_when_transfer_in_paused_state(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let _ = watch!(contract_alice.pause())?;

    let err = send!(contract_alice.transfer(bob_addr, uint!(1_U256)))
        .expect_err("should return EnforcedPause");
    assert!(err.reverted_with(Erc20::EnforcedPause {}));

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_supply, supply);

    Ok(())
}

#[e2e::test]
async fn error_when_transfer_from(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc20::new(contract_addr, &alice.wallet);
    let contract_bob = Erc20::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let _ = watch!(contract_alice.approve(bob_addr, balance))?;

    let Erc20::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    let _ = watch!(contract_alice.pause())?;

    let err =
        send!(contract_bob.transferFrom(alice_addr, bob_addr, uint!(1_U256)))
            .expect_err("should return EnforcedPause");
    assert!(err.reverted_with(Erc20::EnforcedPause {}));

    let Erc20::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;
    let Erc20::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_supply, supply);
    assert_eq!(initial_allowance, allowance);

    Ok(())
}
