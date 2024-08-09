#![cfg(feature = "e2e")]

use abi::Erc721;
use alloy::{
    primitives::{fixed_bytes, Address, U256},
    sol,
    sol_types::SolConstructor,
};
use alloy_primitives::uint;
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt, Revert};

mod abi;

sol!("src/constructor.sol");

fn random_token_id() -> U256 {
    let num: u32 = rand::random();
    U256::from(num)
}

async fn deploy(rpc_url: &str, private_key: &str) -> eyre::Result<Address> {
    let args = Erc721Example::constructorCall {};
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(rpc_url, private_key, Some(args)).await?.address()
}

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let Erc721::pausedReturn { paused } = contract.paused().call().await?;

    assert_eq!(false, paused);

    Ok(())
}

#[e2e::test]
async fn pauses(alice: Account) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receipt = receipt!(contract.pause())?;

    assert!(receipt.emits(Erc721::Paused { account: alice.address() }));

    let Erc721::pausedReturn { paused } = contract.paused().call().await?;

    assert!(paused);

    Ok(())
}

#[e2e::test]
async fn unpauses(alice: Account) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let _ = watch!(contract.pause())?;

    let receipt = receipt!(contract.unpause())?;

    assert!(receipt.emits(Erc721::Unpaused { account: alice.address() }));

    let Erc721::pausedReturn { paused } = contract.paused().call().await?;

    assert_eq!(false, paused);

    Ok(())
}

#[e2e::test]
async fn error_when_burn_in_paused_state(alice: Account) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let _ = watch!(contract.pause());

    let err = send!(contract.burn(token_id))
        .expect_err("should return EnforcedPause");

    assert!(err.reverted_with(Erc721::EnforcedPause {}));

    let Erc721::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;

    assert_eq!(initial_balance, balance);

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;

    assert_eq!(alice_addr, ownerOf);
    Ok(())
}

#[e2e::test]
async fn error_when_mint_in_paused_state(alice: Account) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();

    let _ = watch!(contract.pause());

    let err = send!(contract.mint(alice_addr, token_id))
        .expect_err("should return EnforcedPause");
    assert!(err.reverted_with(Erc721::EnforcedPause {}));

    let err = contract
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return ERC721NonexistentToken");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    let Erc721::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;
    assert_eq!(uint!(0_U256), balance);

    Ok(())
}

#[e2e::test]
async fn error_when_transfer_in_paused_state(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_bob_balance } =
        contract.balanceOf(bob_addr).call().await?;

    let _ = watch!(contract.pause());

    let err = send!(contract.transferFrom(alice_addr, bob_addr, token_id))
        .expect_err("should return EnforcedPause");
    assert!(err.reverted_with(Erc721::EnforcedPause {}));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob_addr).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);

    Ok(())
}

#[e2e::test]
async fn error_when_safe_transfer_in_paused_state(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_bob_balance } =
        contract.balanceOf(bob_addr).call().await?;

    let _ = watch!(contract.pause());

    let err =
        send!(contract.safeTransferFrom_0(alice_addr, bob_addr, token_id))
            .expect_err("should return EnforcedPause");
    assert!(err.reverted_with(Erc721::EnforcedPause {}));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob_addr).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);

    Ok(())
}

#[e2e::test]
async fn error_when_safe_transfer_with_data_in_paused_state(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_bob_balance } =
        contract.balanceOf(bob_addr).call().await?;

    let _ = watch!(contract.pause());

    let err = send!(contract.safeTransferFrom_1(
        alice_addr,
        bob_addr,
        token_id,
        fixed_bytes!("deadbeef").into()
    ))
    .expect_err("should return EnforcedPause");
    assert!(err.reverted_with(Erc721::EnforcedPause {}));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob_addr).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);

    Ok(())
}
