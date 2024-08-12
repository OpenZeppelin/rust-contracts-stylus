#![cfg(feature = "e2e")]

use abi::Erc20Permit;
use alloy::{
    primitives::{b256, keccak256, Address, B256, U256},
    sol,
    sol_types::{SolConstructor, SolType},
};
use alloy_primitives::uint;
use e2e::{receipt, send, watch, Account, EventExt, Revert};
use eyre::Result;

mod abi;

sol!("src/constructor.sol");

// Saturday, 1 January 2000 00:00:00
const EXPIRED_DEADLINE: U256 = uint!(946_684_800_U256);

// Wednesday, 1 January 3000 00:00:00
const FAIR_DEADLINE: U256 = uint!(32_503_680_000_U256);

// keccak256("Permit(address owner,address spender,uint256 value,uint256
// nonce,uint256 deadline)")
const PERMIT_TYPEHASH: B256 =
    b256!("6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c9");

type PermitStructHashTuple = sol! {
    tuple(bytes32, address, address, uint256, uint256, uint256)
};

macro_rules! domain_separator {
    ($contract:expr) => {{
        let Erc20Permit::DOMAIN_SEPARATORReturn { domainSeparator } = $contract
            .DOMAIN_SEPARATOR()
            .call()
            .await
            .expect("should return `DOMAIN_SEPARATOR`");
        B256::from_slice(domainSeparator.as_slice())
    }};
}

async fn deploy(rpc_url: &str, private_key: &str) -> eyre::Result<Address> {
    let args = Erc20PermitExample::constructorCall {};
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(rpc_url, private_key, Some(args)).await
}

fn to_typed_data_hash(domain_separator: B256, struct_hash: B256) -> B256 {
    let typed_dat_hash =
        openzeppelin_stylus::utils::cryptography::eip712::to_typed_data_hash(
            &domain_separator,
            &struct_hash,
        );

    B256::from_slice(typed_dat_hash.as_slice())
}

fn permit_struct_hash(
    owner: Address,
    spender: Address,
    value: U256,
    nonce: U256,
    deadline: U256,
) -> B256 {
    keccak256(PermitStructHashTuple::abi_encode(&(
        *PERMIT_TYPEHASH,
        owner,
        spender,
        value,
        nonce,
        deadline,
    )))
}

// ============================================================================
// Integration Tests: ERC-20 Permit Extension
// ============================================================================

#[e2e::test]
async fn error_when_expired_deadline_for_permit(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let _ = watch!(contract_alice.mint(alice_addr, balance))?;

    let struct_hash = permit_struct_hash(
        alice_addr,
        bob_addr,
        balance,
        U256::ZERO,
        EXPIRED_DEADLINE,
    );

    let typed_data_hash =
        to_typed_data_hash(domain_separator!(contract_alice), struct_hash);
    let signature = alice
        .sign_hash(&alloy::primitives::B256::from_slice(
            typed_data_hash.as_slice(),
        ))
        .await;

    let err = send!(contract_alice.permit(
        alice_addr,
        bob_addr,
        balance,
        EXPIRED_DEADLINE,
        signature.v().y_parity_byte_non_eip155().unwrap(),
        signature.r().into(),
        signature.s().into()
    ))
    .expect_err("should return `ERC2612ExpiredSignature`");
    assert!(err.reverted_with(Erc20Permit::ERC2612ExpiredSignature {
        deadline: EXPIRED_DEADLINE
    }));

    Ok(())
}

#[e2e::test]
async fn permit_works(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let _ = watch!(contract_alice.mint(alice_addr, balance))?;

    let struct_hash = permit_struct_hash(
        alice_addr,
        bob_addr,
        balance,
        U256::ZERO,
        FAIR_DEADLINE,
    );

    let typed_data_hash =
        to_typed_data_hash(domain_separator!(contract_alice), struct_hash);
    let signature = alice
        .sign_hash(&alloy::primitives::B256::from_slice(
            typed_data_hash.as_slice(),
        ))
        .await;

    let Erc20Permit::noncesReturn { nonce: initial_nonce } =
        contract_alice.nonces(alice_addr).call().await?;

    let Erc20Permit::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    let receipt = receipt!(contract_alice.permit(
        alice_addr,
        bob_addr,
        balance,
        FAIR_DEADLINE,
        signature.v().y_parity_byte_non_eip155().unwrap(),
        signature.r().into(),
        signature.s().into()
    ))?;

    assert!(receipt.emits(Erc20Permit::Approval {
        owner: alice_addr,
        spender: bob_addr,
        value: balance,
    }));

    let Erc20Permit::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert_eq!(initial_allowance + balance, allowance);

    let Erc20Permit::noncesReturn { nonce } =
        contract_alice.nonces(alice_addr).call().await?;

    assert_eq!(initial_nonce + uint!(1_U256), nonce);

    let contract_bob = Erc20Permit::new(contract_addr, &bob.wallet);
    let value = balance - uint!(1_U256);
    let Erc20Permit::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;

    let receipt =
        receipt!(contract_bob.transferFrom(alice_addr, bob_addr, value))?;

    let Erc20Permit::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20Permit::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert!(receipt.emits(Erc20Permit::Transfer {
        from: alice_addr,
        to: bob_addr,
        value
    }));

    assert_eq!(initial_alice_balance - value, alice_balance);
    assert_eq!(initial_bob_balance + value, bob_balance);
    assert_eq!(initial_allowance + balance - value, allowance);

    Ok(())
}

#[e2e::test]
async fn permit_rejects_reused_signature(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let _ = watch!(contract_alice.mint(alice_addr, balance))?;

    let struct_hash = permit_struct_hash(
        alice_addr,
        bob_addr,
        balance,
        U256::ZERO,
        FAIR_DEADLINE,
    );

    let typed_data_hash =
        to_typed_data_hash(domain_separator!(contract_alice), struct_hash);
    let signature = alice
        .sign_hash(&alloy::primitives::B256::from_slice(
            typed_data_hash.as_slice(),
        ))
        .await;

    let _ = watch!(contract_alice.permit(
        alice_addr,
        bob_addr,
        balance,
        FAIR_DEADLINE,
        signature.v().y_parity_byte_non_eip155().unwrap(),
        signature.r().into(),
        signature.s().into()
    ))?;

    let err = send!(contract_alice.permit(
        alice_addr,
        bob_addr,
        balance,
        FAIR_DEADLINE,
        signature.v().y_parity_byte_non_eip155().unwrap(),
        signature.r().into(),
        signature.s().into()
    ))
    .expect_err("should return `ERC2612InvalidSigner`");

    let struct_hash = permit_struct_hash(
        alice_addr,
        bob_addr,
        balance,
        U256::from(1),
        FAIR_DEADLINE,
    );

    let typed_data_hash =
        to_typed_data_hash(domain_separator!(contract_alice), struct_hash);

    let recovered = signature
        .recover_address_from_prehash(&alloy::primitives::B256::from_slice(
            typed_data_hash.as_slice(),
        ))
        .expect("should recover");

    assert!(err.reverted_with(Erc20Permit::ERC2612InvalidSigner {
        signer: recovered,
        owner: alice_addr
    }));

    Ok(())
}

#[e2e::test]
async fn permit_rejects_invalid_signature(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let _ = watch!(contract_alice.mint(alice_addr, balance))?;

    let struct_hash = permit_struct_hash(
        alice_addr,
        bob_addr,
        balance,
        U256::ZERO,
        FAIR_DEADLINE,
    );

    let typed_data_hash =
        to_typed_data_hash(domain_separator!(contract_alice), struct_hash);
    let signature = bob
        .sign_hash(&alloy::primitives::B256::from_slice(
            typed_data_hash.as_slice(),
        ))
        .await;

    let err = send!(contract_alice.permit(
        alice_addr,
        bob_addr,
        balance,
        FAIR_DEADLINE,
        signature.v().y_parity_byte_non_eip155().unwrap(),
        signature.r().into(),
        signature.s().into()
    ))
    .expect_err("should return `ERC2612InvalidSigner`");

    assert!(err.reverted_with(Erc20Permit::ERC2612InvalidSigner {
        signer: bob_addr,
        owner: alice_addr
    }));

    Ok(())
}

// ============================================================================
// Integration Tests: ERC-20 Token
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc20Permit::new(contract_addr, &alice.wallet);

    let Erc20Permit::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(total_supply, U256::ZERO);
    Ok(())
}

#[e2e::test]
async fn mints(alice: Account) -> Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let Erc20Permit::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(U256::ZERO, initial_balance);
    assert_eq!(U256::ZERO, initial_supply);

    let one = uint!(1_U256);
    let receipt = receipt!(contract.mint(alice_addr, one))?;
    assert!(receipt.emits(Erc20Permit::Transfer {
        from: Address::ZERO,
        to: alice_addr,
        value: one,
    }));

    let Erc20Permit::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(initial_balance + one, balance);
    assert_eq!(initial_supply + one, total_supply);
    Ok(())
}

#[e2e::test]
async fn mints_rejects_invalid_receiver(alice: Account) -> Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc20Permit::new(contract_addr, &alice.wallet);
    let invalid_receiver = Address::ZERO;

    let Erc20Permit::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(invalid_receiver).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await?;

    let value = uint!(10_U256);
    let err = send!(contract.mint(invalid_receiver, value))
        .expect_err("should not mint tokens for Address::ZERO");
    assert!(err.reverted_with(Erc20Permit::ERC20InvalidReceiver {
        receiver: invalid_receiver
    }));

    let Erc20Permit::balanceOfReturn { balance } =
        contract.balanceOf(invalid_receiver).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(initial_balance, balance);
    assert_eq!(initial_supply, total_supply);
    Ok(())
}

#[e2e::test]
async fn transfers(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20Permit::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let receipt = receipt!(contract_alice.transfer(bob_addr, value))?;

    let Erc20Permit::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;

    assert!(receipt.emits(Erc20Permit::Transfer {
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
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(11_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20Permit::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let err = send!(contract_alice.transfer(bob_addr, value))
        .expect_err("should not transfer when insufficient balance");
    assert!(err.reverted_with(Erc20Permit::ERC20InsufficientBalance {
        sender: alice_addr,
        balance,
        needed: value
    }));

    let Erc20Permit::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_supply, supply);

    Ok(())
}

#[e2e::test]
async fn transfer_rejects_invalid_receiver(alice: Account) -> Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let invalid_receiver = Address::ZERO;

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20Permit::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: initial_receiver_balance } =
        contract_alice.balanceOf(invalid_receiver).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let err = send!(contract_alice.transfer(invalid_receiver, value))
        .expect_err("should not transfer to Address::ZERO");
    assert!(err.reverted_with(Erc20Permit::ERC20InvalidReceiver {
        receiver: invalid_receiver
    }));

    let Erc20Permit::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: receiver_balance } =
        contract_alice.balanceOf(invalid_receiver).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_receiver_balance, receiver_balance);
    assert_eq!(initial_supply, supply);

    Ok(())
}

#[e2e::test]
async fn approves(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let one = uint!(1_U256);
    let ten = uint!(10_U256);

    let Erc20Permit::allowanceReturn { allowance: initial_alice_bob_allowance } =
        contract.allowance(alice_addr, bob_addr).call().await?;
    let Erc20Permit::allowanceReturn { allowance: initial_bob_alice_allowance } =
        contract.allowance(bob_addr, alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: initial_bob_balance } =
        contract.balanceOf(bob_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(U256::ZERO, initial_alice_bob_allowance);
    assert_eq!(U256::ZERO, initial_bob_alice_allowance);

    let receipt = receipt!(contract.approve(bob_addr, one))?;
    assert!(receipt.emits(Erc20Permit::Approval {
        owner: alice_addr,
        spender: bob_addr,
        value: one,
    }));

    let Erc20Permit::allowanceReturn { allowance: alice_bob_allowance } =
        contract.allowance(alice_addr, bob_addr).call().await?;
    let Erc20Permit::allowanceReturn { allowance: bob_alice_allowance } =
        contract.allowance(bob_addr, alice_addr).call().await?;

    assert_eq!(initial_alice_bob_allowance + one, alice_bob_allowance);
    assert_eq!(initial_bob_alice_allowance, bob_alice_allowance);

    let receipt = receipt!(contract.approve(bob_addr, ten))?;
    assert!(receipt.emits(Erc20Permit::Approval {
        owner: alice_addr,
        spender: bob_addr,
        value: ten,
    }));

    let Erc20Permit::allowanceReturn { allowance: alice_bob_allowance } =
        contract.allowance(alice_addr, bob_addr).call().await?;
    let Erc20Permit::allowanceReturn { allowance: bob_alice_allowance } =
        contract.allowance(bob_addr, alice_addr).call().await?;

    let Erc20Permit::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: supply } =
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
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let invalid_spender = Address::ZERO;

    let ten = uint!(10_U256);

    let Erc20Permit::allowanceReturn {
        allowance: initial_alice_spender_allowance,
    } = contract.allowance(alice_addr, invalid_spender).call().await?;
    let Erc20Permit::allowanceReturn {
        allowance: initial_spender_alice_allowance,
    } = contract.allowance(invalid_spender, alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: initial_spender_balance } =
        contract.balanceOf(invalid_spender).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply().call().await?;

    assert_eq!(U256::ZERO, initial_alice_spender_allowance);
    assert_eq!(U256::ZERO, initial_spender_alice_allowance);

    let err = send!(contract.approve(invalid_spender, ten))
        .expect_err("should not approve for Address::ZERO");

    assert!(err.reverted_with(Erc20Permit::ERC20InvalidSpender {
        spender: invalid_spender
    }));

    let Erc20Permit::allowanceReturn { allowance: alice_spender_allowance } =
        contract.allowance(alice_addr, invalid_spender).call().await?;
    let Erc20Permit::allowanceReturn { allowance: spender_alice_allowance } =
        contract.allowance(invalid_spender, alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: spender_balance } =
        contract.balanceOf(invalid_spender).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: supply } =
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
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let contract_bob = Erc20Permit::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20Permit::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let _ = watch!(contract_alice.approve(bob_addr, balance))?;

    let Erc20Permit::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    let receipt =
        receipt!(contract_bob.transferFrom(alice_addr, bob_addr, value))?;

    let Erc20Permit::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;
    let Erc20Permit::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert!(receipt.emits(Erc20Permit::Transfer {
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
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let contract_bob = Erc20Permit::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(1_U256);
    let value = uint!(10_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20Permit::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let _ = watch!(contract_alice.approve(bob_addr, value))?;

    let Erc20Permit::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    let err = send!(contract_bob.transferFrom(alice_addr, bob_addr, value))
        .expect_err("should not transfer when insufficient balance");

    assert!(err.reverted_with(Erc20Permit::ERC20InsufficientBalance {
        sender: alice_addr,
        balance,
        needed: value
    }));

    let Erc20Permit::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;
    let Erc20Permit::allowanceReturn { allowance } =
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
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let contract_bob = Erc20Permit::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20Permit::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let Erc20Permit::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert_eq!(initial_allowance, U256::ZERO);

    let err = send!(contract_bob.transferFrom(alice_addr, bob_addr, value))
        .expect_err("should not transfer when insufficient allowance");

    assert!(err.reverted_with(Erc20Permit::ERC20InsufficientAllowance {
        spender: bob_addr,
        allowance: U256::ZERO,
        needed: value
    }));

    let Erc20Permit::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;
    let Erc20Permit::allowanceReturn { allowance } =
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
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let contract_bob = Erc20Permit::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let invalid_receiver = Address::ZERO;

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    let Erc20Permit::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: initial_receiver_balance } =
        contract_alice.balanceOf(invalid_receiver).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: initial_supply } =
        contract_alice.totalSupply().call().await?;

    let _ = watch!(contract_alice.approve(bob_addr, balance))?;

    let Erc20Permit::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    let err =
        send!(contract_bob.transferFrom(alice_addr, invalid_receiver, value))
            .expect_err("should not transfer to Address::ZERO");

    assert!(err.reverted_with(Erc20Permit::ERC20InvalidReceiver {
        receiver: invalid_receiver
    }));

    let Erc20Permit::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;
    let Erc20Permit::balanceOfReturn { balance: receiver_balance } =
        contract_alice.balanceOf(bob_addr).call().await?;
    let Erc20Permit::totalSupplyReturn { totalSupply: supply } =
        contract_alice.totalSupply().call().await?;
    let Erc20Permit::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_receiver_balance, receiver_balance);
    assert_eq!(initial_supply, supply);
    assert_eq!(initial_allowance, allowance);

    Ok(())
}
