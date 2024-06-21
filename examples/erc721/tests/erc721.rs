#![cfg(feature = "e2e")]

use alloy::{
    primitives::{Address, U256},
    sol,
    sol_types::SolConstructor,
};
use alloy_primitives::uint;
use e2e::{receipt, send, watch, EventExt, Revert, User};

use crate::abi::Erc721;

mod abi;

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

#[e2e::test]
async fn constructs(alice: User) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let name = contract.name().call().await?.name;
    let symbol = contract.symbol().call().await?.symbol;

    assert_eq!(name, TOKEN_NAME.to_owned());
    assert_eq!(symbol, TOKEN_SYMBOL.to_owned());
    Ok(())
}

#[e2e::test]
async fn mints(alice: User) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;
    let owner_of = contract.ownerOf(token_id).call().await?.ownerOf;
    assert_eq!(owner_of, alice_addr);

    let balance = contract.balanceOf(alice_addr).call().await?.balance;
    assert!(balance >= uint!(1_U256));
    Ok(())
}

#[e2e::test]
async fn errors_when_reusing_token_id(alice: User) -> eyre::Result<()> {
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
async fn transfers(alice: User, bob: User) -> eyre::Result<()> {
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
    alice: User,
    bob: User,
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
async fn approves_token_transfer(alice: User, bob: User) -> eyre::Result<()> {
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
    assert_ne!(ownerOf, alice_addr);
    assert_eq!(ownerOf, bob_addr);
    Ok(())
}

#[e2e::test]
async fn errors_when_transfer_unapproved_token(
    alice: User,
    bob: User,
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
