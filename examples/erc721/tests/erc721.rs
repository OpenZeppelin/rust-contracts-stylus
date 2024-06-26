#![cfg(feature = "e2e")]

use alloy::{
    primitives::{fixed_bytes, Address, U256},
    sol,
    sol_types::SolConstructor,
};
use alloy_primitives::uint;
use e2e::{receipt, send, watch, Account, EventExt, Panic, PanicCode, Revert};

use crate::{abi::Erc721, mock_receiver::ERC721ReceiverMock};

mod abi;
mod mock_receiver;

sol!("src/constructor.sol");

const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "NFT";

fn random_token_id() -> U256 {
    let num: u32 = rand::random();
    U256::from(num)
}

async fn deploy(rpc_url: &str, private_key: &str) -> eyre::Result<Address> {
    let args = Erc721Example::constructorCall {
        name_: TOKEN_NAME.to_owned(),
        symbol_: TOKEN_SYMBOL.to_owned(),
    };
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(rpc_url, private_key, Some(args)).await
}

// ============================================================================
// Integration Tests: ERC-721 Token
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let Erc721::nameReturn { name } = contract.name().call().await?;
    let Erc721::symbolReturn { symbol } = contract.symbol().call().await?;

    assert_eq!(name, TOKEN_NAME.to_owned());
    assert_eq!(symbol, TOKEN_SYMBOL.to_owned());

    Ok(())
}

#[e2e::test]
async fn error_when_balance_of_invalid_owner(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);
    let invalid_owner = Address::ZERO;

    let err = contract
        .balanceOf(invalid_owner)
        .call()
        .await
        .expect_err("should return ERC721InvalidOwner");
    assert!(
        err.reverted_with(Erc721::ERC721InvalidOwner { owner: invalid_owner })
    );

    Ok(())
}

#[e2e::test]
async fn balance_of_zero_balance(alice: Account) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let Erc721::balanceOfReturn { balance } =
        contract.balanceOf(alice.address()).call().await?;
    assert_eq!(uint!(0_U256), balance);

    Ok(())
}

#[e2e::test]
async fn mints(alice: Account) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::ownerOfReturn { ownerOf: owner_of } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(owner_of, alice_addr);

    let Erc721::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;
    assert_eq!(uint!(1_U256), balance);

    Ok(())
}

#[e2e::test]
async fn errors_when_reusing_token_id(alice: Account) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let err = send!(contract.mint(alice_addr, token_id))
        .expect_err("should not mint a token id twice");
    assert!(err
        .reverted_with(Erc721::ERC721InvalidSender { sender: Address::ZERO }));

    Ok(())
}

#[e2e::test]
async fn transfers(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;
    let receipt =
        receipt!(contract.transferFrom(alice_addr, bob_addr, token_id))?;

    receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    });

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(ownerOf, bob_addr);

    Ok(())
}

#[e2e::test]
async fn errors_when_transfer_nonexistent_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();
    let tx = contract.transferFrom(alice_addr, bob.address(), token_id);

    let err = send!(tx).expect_err("should not transfer a non-existent token");
    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    Ok(())
}

#[e2e::test]
async fn safe_transfers_to_receiver_contract(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address = mock_receiver::deploy(
        &alice.wallet,
        ERC721ReceiverMock::RevertType::None,
    )
    .await?;

    let alice_addr = alice.address();
    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;
    let receipt = receipt!(contract.safeTransferFrom_0(
        alice_addr,
        receiver_address,
        token_id
    ))?;

    receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: receiver_address,
        tokenId: token_id,
    });

    receipt.emits(ERC721ReceiverMock::Received {
        operator: alice_addr,
        from: alice_addr,
        tokenId: token_id,
        data: fixed_bytes!("").into(),
    });

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(ownerOf, receiver_address);

    Ok(())
}

#[e2e::test]
async fn approves_token_transfer(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;
    let _ = watch!(contract.approve(bob_addr, token_id))?;

    let contract = Erc721::new(contract_addr, &bob.wallet);

    let _ = watch!(contract.transferFrom(alice_addr, bob_addr, token_id))?;

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;

    assert_eq!(ownerOf, bob_addr);

    Ok(())
}

#[e2e::test]
async fn errors_when_transfer_unapproved_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let contract = Erc721::new(contract_addr, &bob.wallet);
    let tx = contract.transferFrom(alice_addr, bob_addr, token_id);

    let err = send!(tx).expect_err("should not transfer unapproved token");

    assert!(err.reverted_with(Erc721::ERC721InsufficientApproval {
        operator: bob_addr,
        tokenId: token_id,
    }));

    Ok(())
}
