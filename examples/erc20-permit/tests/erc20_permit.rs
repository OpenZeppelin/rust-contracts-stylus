#![cfg(feature = "e2e")]

use abi::Erc20Permit;
use alloy::{
    primitives::{keccak256, Address, B256, U256},
    sol,
    sol_types::SolType,
};
use alloy_primitives::uint;
use e2e::{receipt, send, watch, Account, EventExt, Revert};
use eyre::Result;
mod abi;

// Saturday, 1 January 2000 00:00:00
const EXPIRED_DEADLINE: U256 = uint!(946_684_800_U256);

// Wednesday, 1 January 3000 00:00:00
const FAIR_DEADLINE: U256 = uint!(32_503_680_000_U256);

const PERMIT_TYPEHASH: [u8; 32] =
    keccak_const::Keccak256::new()
        .update(b"Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)")
        .finalize();

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
        PERMIT_TYPEHASH,
        owner,
        spender,
        value,
        nonce,
        deadline,
    )))
}

// I was unable to find a function in alloy that converts `v` into [non-eip155
// value], so I implemented the logic manually.
//
// [non-eip155 value]: https://eips.ethereum.org/EIPS/eip-155
fn to_non_eip155_v(v: bool) -> u8 {
    u8::from(v) + 27
}

// ============================================================================
// Integration Tests: ERC-20 Permit Extension
// ============================================================================

#[e2e::test]
async fn error_when_expired_deadline_for_permit(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    watch!(contract_alice.mint(alice_addr, balance))?;

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
        to_non_eip155_v(signature.v()),
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
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    watch!(contract_alice.mint(alice_addr, balance))?;

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
        to_non_eip155_v(signature.v()),
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

    assert_eq!(initial_nonce + U256::ONE, nonce);

    let contract_bob = Erc20Permit::new(contract_addr, &bob.wallet);
    let value = balance - U256::ONE;
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
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    watch!(contract_alice.mint(alice_addr, balance))?;

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

    watch!(contract_alice.permit(
        alice_addr,
        bob_addr,
        balance,
        FAIR_DEADLINE,
        to_non_eip155_v(signature.v()),
        signature.r().into(),
        signature.s().into()
    ))?;

    let err = send!(contract_alice.permit(
        alice_addr,
        bob_addr,
        balance,
        FAIR_DEADLINE,
        to_non_eip155_v(signature.v()),
        signature.r().into(),
        signature.s().into()
    ))
    .expect_err("should return `ERC2612InvalidSigner`");

    let struct_hash = permit_struct_hash(
        alice_addr,
        bob_addr,
        balance,
        U256::ONE,
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
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc20Permit::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    watch!(contract_alice.mint(alice_addr, balance))?;

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
        to_non_eip155_v(signature.v()),
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
