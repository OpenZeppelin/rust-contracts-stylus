#![cfg(feature = "e2e")]

use abi::Erc721;
use alloy::primitives::{fixed_bytes, uint, Address, Bytes, U256};
use e2e::{
    receipt, send, watch, Account, EventExt, PanicCode, ReceiptExt, Revert,
};
use mock::{receiver, receiver::ERC721ReceiverMock};

mod abi;
mod mock;

fn random_token_id() -> U256 {
    let num: u32 = rand::random();
    U256::from(num)
}

// ============================================================================
// Integration Tests: ERC-721 Token
// ============================================================================

#[e2e::test]
async fn constructor_initializes_contract_with_default_settings(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let paused = contract.paused().call().await?.paused;

    assert!(!paused);

    Ok(())
}

#[e2e::test]
async fn balance_of_reverts_when_owner_invalid(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);
    let invalid_owner = Address::ZERO;

    let err = contract
        .balanceOf(invalid_owner)
        .call()
        .await
        .expect_err("should return `ERC721InvalidOwner`");
    assert!(
        err.reverted_with(Erc721::ERC721InvalidOwner { owner: invalid_owner })
    );

    Ok(())
}

#[e2e::test]
async fn balance_of_returns_zero_for_new_address(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let Erc721::balanceOfReturn { balance } =
        contract.balanceOf(alice.address()).call().await?;
    assert_eq!(uint!(0_U256), balance);

    Ok(())
}

#[e2e::test]
async fn owner_of_reverts_when_token_nonexistent(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);
    let token_id = random_token_id();

    let err = contract
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    Ok(())
}

#[e2e::test]
async fn mint_creates_token_for_valid_recipient(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();
    let receipt = receipt!(contract.mint(alice_addr, token_id))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: Address::ZERO,
        to: alice_addr,
        tokenId: token_id
    }));

    let Erc721::ownerOfReturn { ownerOf: owner_of } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, owner_of);

    let Erc721::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;
    assert_eq!(uint!(1_U256), balance);

    Ok(())
}

#[e2e::test]
async fn mint_reverts_when_token_id_exists(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
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
async fn mint_reverts_when_receiver_invalid(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();
    let invalid_receiver = Address::ZERO;

    let err = send!(contract.mint(invalid_receiver, token_id))
        .expect_err("should not mint a token for invalid receiver");
    assert!(err.reverted_with(Erc721::ERC721InvalidReceiver {
        receiver: invalid_receiver
    }));

    Ok(())
}

#[e2e::test]
async fn transfer_from_moves_token_between_owners(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_bob_balance } =
        contract.balanceOf(bob_addr).call().await?;

    let receipt =
        receipt!(contract.transferFrom(alice_addr, bob_addr, token_id))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob_addr).call().await?;

    let one = uint!(1_U256);
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn transfer_from_succeeds_with_token_approval(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.approve(bob_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_bob_balance } =
        contract_bob.balanceOf(bob_addr).call().await?;

    let receipt =
        receipt!(contract_bob.transferFrom(alice_addr, bob_addr, token_id))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract_alice.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract_bob.balanceOf(bob_addr).call().await?;

    let one = uint!(1_U256);
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn transfer_from_succeeds_with_approval_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.setApprovalForAll(bob_addr, true))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_bob_balance } =
        contract_bob.balanceOf(bob_addr).call().await?;

    let receipt =
        receipt!(contract_bob.transferFrom(alice_addr, bob_addr, token_id))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract_alice.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract_bob.balanceOf(bob_addr).call().await?;

    let one = uint!(1_U256);
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn transfer_from_reverts_when_receiver_invalid(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let invalid_receiver = Address::ZERO;
    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let err =
        send!(contract.transferFrom(alice_addr, invalid_receiver, token_id))
            .expect_err("should not transfer the token to invalid receiver");

    assert!(err.reverted_with(Erc721::ERC721InvalidReceiver {
        receiver: invalid_receiver
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn transfer_from_reverts_when_sender_is_not_owner(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let err = send!(contract.transferFrom(dave_addr, bob_addr, token_id))
        .expect_err("should not transfer the token from incorrect owner");

    assert!(err.reverted_with(Erc721::ERC721IncorrectOwner {
        sender: dave_addr,
        owner: alice_addr,
        tokenId: token_id
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn transfer_from_reverts_when_approval_insufficient(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let contract = Erc721::new(contract_addr, &bob.wallet);
    let err = send!(contract.transferFrom(alice_addr, bob_addr, token_id))
        .expect_err("should not transfer unapproved token");

    assert!(err.reverted_with(Erc721::ERC721InsufficientApproval {
        operator: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn transfer_from_reverts_when_token_nonexistent(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();

    let err = send!(contract.transferFrom(alice_addr, bob.address(), token_id))
        .expect_err("should not transfer a non-existent token");
    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    let err = contract
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_moves_token_between_owners(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_bob_balance } =
        contract.balanceOf(bob_addr).call().await?;

    let receipt =
        receipt!(contract.safeTransferFrom_0(alice_addr, bob_addr, token_id))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob_addr).call().await?;

    let one = uint!(1_U256);
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_succeeds_with_receiver_contract(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address =
        receiver::deploy(&alice.wallet, ERC721ReceiverMock::RevertType::None)
            .await?;

    let alice_addr = alice.address();
    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_receiver_balance } =
        contract.balanceOf(receiver_address).call().await?;

    let receipt = receipt!(contract.safeTransferFrom_0(
        alice_addr,
        receiver_address,
        token_id
    ))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: receiver_address,
        tokenId: token_id,
    }));

    assert!(receipt.emits(ERC721ReceiverMock::Received {
        operator: alice_addr,
        from: alice_addr,
        tokenId: token_id,
        data: fixed_bytes!("").into(),
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(receiver_address, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: receiver_balance } =
        contract.balanceOf(receiver_address).call().await?;

    let one = uint!(1_U256);
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_receiver_balance + one, receiver_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_succeeds_with_token_approval(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.approve(bob_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_bob_balance } =
        contract_bob.balanceOf(bob_addr).call().await?;

    let receipt = receipt!(
        contract_bob.safeTransferFrom_0(alice_addr, bob_addr, token_id)
    )?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract_alice.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract_bob.balanceOf(bob_addr).call().await?;

    let one = uint!(1_U256);
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_succeeds_with_approval_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.setApprovalForAll(bob_addr, true))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_bob_balance } =
        contract_bob.balanceOf(bob_addr).call().await?;

    let receipt = receipt!(
        contract_bob.safeTransferFrom_0(alice_addr, bob_addr, token_id)
    )?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract_alice.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract_bob.balanceOf(bob_addr).call().await?;

    let one = uint!(1_U256);
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_reverts_when_receiver_invalid(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let invalid_receiver = Address::ZERO;
    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let err = send!(contract.safeTransferFrom_0(
        alice_addr,
        invalid_receiver,
        token_id
    ))
    .expect_err("should not transfer the token to invalid receiver");
    assert!(err.reverted_with(Erc721::ERC721InvalidReceiver {
        receiver: invalid_receiver
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_reverts_when_owner_incorrect(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let err = send!(contract.safeTransferFrom_0(dave_addr, bob_addr, token_id))
        .expect_err("should not transfer the token from incorrect owner");

    assert!(err.reverted_with(Erc721::ERC721IncorrectOwner {
        sender: dave_addr,
        owner: alice_addr,
        tokenId: token_id
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_reverts_when_approval_insufficient(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let contract = Erc721::new(contract_addr, &bob.wallet);

    let err =
        send!(contract.safeTransferFrom_0(alice_addr, bob_addr, token_id))
            .expect_err("should not transfer unapproved token");

    assert!(err.reverted_with(Erc721::ERC721InsufficientApproval {
        operator: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_reverts_when_token_nonexistent(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();

    let err =
        send!(contract.safeTransferFrom_0(alice_addr, bob.address(), token_id))
            .expect_err("should not transfer a non-existent token");
    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    let err = contract
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_moves_token_with_data(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_bob_balance } =
        contract.balanceOf(bob_addr).call().await?;

    let receipt = receipt!(contract.safeTransferFrom_1(
        alice_addr,
        bob_addr,
        token_id,
        fixed_bytes!("deadbeef").into()
    ))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob_addr).call().await?;

    let one = uint!(1_U256);
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_succeeds_with_data_and_receiver_contract(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address =
        receiver::deploy(&alice.wallet, ERC721ReceiverMock::RevertType::None)
            .await?;

    let alice_addr = alice.address();
    let token_id = random_token_id();
    let data: Bytes = fixed_bytes!("deadbeef").into();

    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_receiver_balance } =
        contract.balanceOf(receiver_address).call().await?;

    let receipt = receipt!(contract.safeTransferFrom_1(
        alice_addr,
        receiver_address,
        token_id,
        data.clone()
    ))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: receiver_address,
        tokenId: token_id,
    }));

    assert!(receipt.emits(ERC721ReceiverMock::Received {
        operator: alice_addr,
        from: alice_addr,
        tokenId: token_id,
        data,
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(receiver_address, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: receiver_balance } =
        contract.balanceOf(receiver_address).call().await?;

    let one = uint!(1_U256);
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_receiver_balance + one, receiver_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_moves_token_with_data_and_approval(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.approve(bob_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_bob_balance } =
        contract_bob.balanceOf(bob_addr).call().await?;

    let receipt = receipt!(contract_bob.safeTransferFrom_1(
        alice_addr,
        bob_addr,
        token_id,
        fixed_bytes!("deadbeef").into()
    ))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract_alice.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract_bob.balanceOf(bob_addr).call().await?;

    let one = uint!(1_U256);
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_moves_token_with_data_and_approval_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.setApprovalForAll(bob_addr, true))?;

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: initial_bob_balance } =
        contract_bob.balanceOf(bob_addr).call().await?;

    let receipt = receipt!(contract_bob.safeTransferFrom_1(
        alice_addr,
        bob_addr,
        token_id,
        fixed_bytes!("deadbeef").into()
    ))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract_alice.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let Erc721::balanceOfReturn { balance: bob_balance } =
        contract_bob.balanceOf(bob_addr).call().await?;

    let one = uint!(1_U256);
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_with_data_reverts_when_receiver_invalid(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let invalid_receiver = Address::ZERO;
    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let err = send!(contract.safeTransferFrom_1(
        alice_addr,
        invalid_receiver,
        token_id,
        fixed_bytes!("deadbeef").into()
    ))
    .expect_err("should not transfer the token to invalid receiver");
    assert!(err.reverted_with(Erc721::ERC721InvalidReceiver {
        receiver: invalid_receiver
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_with_data_reverts_when_owner_incorrect(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let err = send!(contract.safeTransferFrom_1(
        dave_addr,
        bob_addr,
        token_id,
        fixed_bytes!("deadbeef").into()
    ))
    .expect_err("should not transfer the token from incorrect owner");

    assert!(err.reverted_with(Erc721::ERC721IncorrectOwner {
        sender: dave_addr,
        owner: alice_addr,
        tokenId: token_id
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_with_data_reverts_when_approval_insufficient(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let contract = Erc721::new(contract_addr, &bob.wallet);

    let err = send!(contract.safeTransferFrom_1(
        alice_addr,
        bob_addr,
        token_id,
        fixed_bytes!("deadbeef").into()
    ))
    .expect_err("should not transfer unapproved token");

    assert!(err.reverted_with(Erc721::ERC721InsufficientApproval {
        operator: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_with_data_reverts_when_token_nonexistent(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();

    let err = send!(contract.safeTransferFrom_1(
        alice_addr,
        bob.address(),
        token_id,
        fixed_bytes!("deadbeef").into()
    ))
    .expect_err("should not transfer a non-existent token");
    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    let err = contract
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_reverts_when_receiver_reverts_with_reason(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC721ReceiverMock::RevertType::RevertWithMessage,
    )
    .await?;

    let alice_addr = alice.address();
    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let err = send!(contract.safeTransferFrom_0(
        alice_addr,
        receiver_address,
        token_id
    ))
    .expect_err("should not transfer when receiver errors with reason");

    assert!(err.reverted_with(Erc721::Error {
        message: "ERC721ReceiverMock: reverting".to_string()
    }));

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_reverts_when_receiver_reverts_without_reason(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC721ReceiverMock::RevertType::RevertWithoutMessage,
    )
    .await?;

    let alice_addr = alice.address();
    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let err = send!(contract.safeTransferFrom_0(
        alice_addr,
        receiver_address,
        token_id
    ))
    .expect_err("should not transfer when receiver reverts");

    assert!(err.reverted_with(Erc721::ERC721InvalidReceiver {
        receiver: receiver_address
    }));

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_reverts_when_receiver_panics(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address =
        receiver::deploy(&alice.wallet, ERC721ReceiverMock::RevertType::Panic)
            .await?;

    let alice_addr = alice.address();
    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let err = send!(contract.safeTransferFrom_0(
        alice_addr,
        receiver_address,
        token_id
    ))
    .expect_err("should not transfer when receiver panics");

    assert!(err.reverted_with(Erc721::Panic {
        code: U256::from(PanicCode::DivisionByZero as u8)
    }));

    Ok(())
}

#[e2e::test]
async fn approve_sets_token_approval_for_spender(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::getApprovedReturn { approved } =
        contract.getApproved(token_id).call().await?;
    assert_eq!(Address::ZERO, approved);

    let receipt = receipt!(contract.approve(bob_addr, token_id))?;

    assert!(receipt.emits(Erc721::Approval {
        owner: alice_addr,
        approved: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::getApprovedReturn { approved } =
        contract.getApproved(token_id).call().await?;
    assert_eq!(bob_addr, approved);

    Ok(())
}

#[e2e::test]
async fn approve_reverts_when_token_nonexistent(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let bob_addr = bob.address();
    let token_id = random_token_id();

    let err = send!(contract.approve(bob_addr, token_id))
        .expect_err("should not approve for a non-existent token");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    Ok(())
}

#[e2e::test]
async fn approve_reverts_when_approver_invalid(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;

    let err = send!(contract_bob.approve(bob_addr, token_id))
        .expect_err("should not approve when invalid approver");

    assert!(
        err.reverted_with(Erc721::ERC721InvalidApprover { approver: bob_addr })
    );

    let Erc721::getApprovedReturn { approved } =
        contract_bob.getApproved(token_id).call().await?;
    assert_eq!(Address::ZERO, approved);

    Ok(())
}

#[e2e::test]
async fn get_approved_reverts_when_token_nonexistent(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let err = contract
        .getApproved(token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );
    Ok(())
}

#[e2e::test]
async fn set_approval_for_all_updates_operator_permission(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let approved_value = true;
    let receipt =
        receipt!(contract.setApprovalForAll(bob_addr, approved_value))?;

    assert!(receipt.emits(Erc721::ApprovalForAll {
        owner: alice_addr,
        operator: bob_addr,
        approved: approved_value,
    }));

    let Erc721::isApprovedForAllReturn { approved } =
        contract.isApprovedForAll(alice_addr, bob_addr).call().await?;
    assert_eq!(approved_value, approved);

    let approved_value = false;
    let receipt =
        receipt!(contract.setApprovalForAll(bob_addr, approved_value))?;

    assert!(receipt.emits(Erc721::ApprovalForAll {
        owner: alice_addr,
        operator: bob_addr,
        approved: approved_value,
    }));

    let Erc721::isApprovedForAllReturn { approved } =
        contract.isApprovedForAll(alice_addr, bob_addr).call().await?;
    assert_eq!(approved_value, approved);

    Ok(())
}

#[e2e::test]
async fn set_approval_for_all_reverts_when_operator_invalid(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let invalid_operator = Address::ZERO;

    let err = send!(contract.setApprovalForAll(invalid_operator, true))
        .expect_err("should return ERC721InvalidOperator");

    assert!(err.reverted_with(Erc721::ERC721InvalidOperator {
        operator: invalid_operator
    }));

    Ok(())
}

#[e2e::test]
async fn is_approved_for_all_returns_false_for_zero_address(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let invalid_operator = Address::ZERO;

    let Erc721::isApprovedForAllReturn { approved } = contract
        .isApprovedForAll(alice.address(), invalid_operator)
        .call()
        .await?;

    assert!(!approved);

    Ok(())
}

#[e2e::test]
async fn safe_mint_creates_token_without_data(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let token_id = random_token_id();
    let data = Bytes::new();

    let initial_balance =
        contract.balanceOf(alice.address()).call().await?.balance;

    let receipt = receipt!(contract.safeMint(alice_addr, token_id, data))?;
    assert!(receipt.emits(Erc721::Transfer {
        from: Address::ZERO,
        to: alice_addr,
        tokenId: token_id,
    }));

    let owner_of = contract.ownerOf(token_id).call().await?.ownerOf;
    assert_eq!(alice_addr, owner_of);

    let balance = contract.balanceOf(alice.address()).call().await?.balance;
    assert_eq!(balance, initial_balance + uint!(1_U256));

    Ok(())
}

#[e2e::test]
async fn safe_mint_creates_token_with_data(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let token_id = random_token_id();
    let data: Bytes = fixed_bytes!("deadbeef").into();

    let initial_balance =
        contract.balanceOf(alice.address()).call().await?.balance;

    let receipt = receipt!(contract.safeMint(alice_addr, token_id, data))?;
    assert!(receipt.emits(Erc721::Transfer {
        from: Address::ZERO,
        to: alice_addr,
        tokenId: token_id,
    }));

    let owner_of = contract.ownerOf(token_id).call().await?.ownerOf;
    assert_eq!(alice_addr, owner_of);

    let balance = contract.balanceOf(alice.address()).call().await?.balance;
    assert_eq!(balance, initial_balance + uint!(1_U256));

    Ok(())
}

#[e2e::test]
async fn safe_mint_succeeds_with_receiver_contract_without_data(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);
    let receiver_address =
        receiver::deploy(&alice.wallet, ERC721ReceiverMock::RevertType::None)
            .await?;

    let token_id = random_token_id();
    let data = Bytes::new();

    let initial_balance =
        contract.balanceOf(alice.address()).call().await?.balance;

    let receipt =
        receipt!(contract.safeMint(receiver_address, token_id, data.clone()))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: Address::ZERO,
        to: receiver_address,
        tokenId: token_id,
    }));

    assert!(receipt.emits(ERC721ReceiverMock::Received {
        operator: alice.address(),
        from: Address::ZERO,
        tokenId: token_id,
        data,
    }));

    let owner_of = contract.ownerOf(token_id).call().await?.ownerOf;
    assert_eq!(receiver_address, owner_of);

    let balance = contract.balanceOf(receiver_address).call().await?.balance;
    assert_eq!(balance, initial_balance + uint!(1_U256));

    Ok(())
}

#[e2e::test]
async fn safe_mint_succeeds_with_receiver_contract_and_data(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);
    let receiver_address =
        receiver::deploy(&alice.wallet, ERC721ReceiverMock::RevertType::None)
            .await?;

    let token_id = random_token_id();
    let data: Bytes = fixed_bytes!("deadbeef").into();

    let initial_balance =
        contract.balanceOf(alice.address()).call().await?.balance;

    let receipt =
        receipt!(contract.safeMint(receiver_address, token_id, data.clone()))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: Address::ZERO,
        to: receiver_address,
        tokenId: token_id,
    }));

    assert!(receipt.emits(ERC721ReceiverMock::Received {
        operator: alice.address(),
        from: Address::ZERO,
        tokenId: token_id,
        data,
    }));

    let owner_of = contract.ownerOf(token_id).call().await?.ownerOf;
    assert_eq!(receiver_address, owner_of);

    let balance = contract.balanceOf(receiver_address).call().await?.balance;
    assert_eq!(balance, initial_balance + uint!(1_U256));

    Ok(())
}

#[e2e::test]
async fn safe_mint_reverts_when_receiver_invalid(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();
    let data: Bytes = fixed_bytes!("deadbeef").into();

    let err = send!(contract.safeMint(contract_addr, token_id, data))
        .expect_err("should not safe mint the token to invalid receiver");

    assert!(err.reverted_with(Erc721::ERC721InvalidReceiver {
        receiver: contract_addr
    }));

    Ok(())
}

#[e2e::test]
async fn safe_mint_reverts_when_sender_invalid(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();
    let data: Bytes = fixed_bytes!("deadbeef").into();

    _ = watch!(contract.mint(alice.address(), token_id))?;

    let err = send!(contract.safeMint(bob.address(), token_id, data))
        .expect_err("should not safe mint an existing token");

    assert!(err
        .reverted_with(Erc721::ERC721InvalidSender { sender: Address::ZERO }));

    Ok(())
}

#[e2e::test]
async fn safe_mint_reverts_when_receiver_reverts_with_reason(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC721ReceiverMock::RevertType::RevertWithMessage,
    )
    .await?;

    let token_id = random_token_id();
    let data: Bytes = fixed_bytes!("deadbeef").into();

    let err = send!(contract.safeMint(receiver_address, token_id, data))
        .expect_err("should not safe mint when receiver errors with reason");

    assert!(err.reverted_with(Erc721::Error {
        message: "ERC721ReceiverMock: reverting".to_string()
    }));

    Ok(())
}

#[e2e::test]
async fn safe_mint_reverts_when_receiver_reverts_without_reason(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC721ReceiverMock::RevertType::RevertWithoutMessage,
    )
    .await?;

    let token_id = random_token_id();
    let data: Bytes = fixed_bytes!("deadbeef").into();

    let err = send!(contract.safeMint(receiver_address, token_id, data))
        .expect_err(
            "should not safe mint when receiver reverts without reason",
        );

    assert!(err.reverted_with(Erc721::ERC721InvalidReceiver {
        receiver: receiver_address
    }));

    Ok(())
}

#[e2e::test]
async fn safe_mint_reverts_when_receiver_panics(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address =
        receiver::deploy(&alice.wallet, ERC721ReceiverMock::RevertType::Panic)
            .await?;

    let token_id = random_token_id();
    let data: Bytes = fixed_bytes!("deadbeef").into();

    let err = send!(contract.safeMint(receiver_address, token_id, data))
        .expect_err("should not safe mint when receiver panics");

    assert!(err.reverted_with(Erc721::Panic {
        code: U256::from(PanicCode::DivisionByZero as u8)
    }));

    Ok(())
}

// ============================================================================
// Integration Tests: ERC-721 Pausable Extension
// ============================================================================

#[e2e::test]
async fn pause_changes_contract_state_to_paused(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receipt = receipt!(contract.pause())?;

    assert!(receipt.emits(Erc721::Paused { account: alice.address() }));

    let Erc721::pausedReturn { paused } = contract.paused().call().await?;

    assert!(paused);

    Ok(())
}

#[e2e::test]
async fn pause_reverts_when_already_paused(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;

    let contract = Erc721::new(contract_addr, &alice.wallet);

    let _ = watch!(contract.pause())?;

    let err =
        send!(contract.pause()).expect_err("should return `EnforcedPause`");

    assert!(err.reverted_with(Erc721::EnforcedPause {}));

    Ok(())
}

#[e2e::test]
async fn unpause_changes_contract_state_to_unpaused(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let _ = watch!(contract.pause())?;

    let receipt = receipt!(contract.unpause())?;

    assert!(receipt.emits(Erc721::Unpaused { account: alice.address() }));

    let Erc721::pausedReturn { paused } = contract.paused().call().await?;

    assert!(!paused);

    Ok(())
}

#[e2e::test]
async fn unpause_reverts_when_already_unpaused(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;

    let contract = Erc721::new(contract_addr, &alice.wallet);

    let paused = contract.paused().call().await?.paused;

    assert!(!paused);

    let err =
        send!(contract.unpause()).expect_err("should return `ExpectedPause`");

    assert!(err.reverted_with(Erc721::ExpectedPause {}));

    Ok(())
}

#[e2e::test]
async fn burn_reverts_when_paused(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let _ = watch!(contract.pause());

    let err = send!(contract.burn(token_id))
        .expect_err("should return `EnforcedPause`");

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
async fn mint_reverts_when_paused(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();

    let _ = watch!(contract.pause());

    let err = send!(contract.mint(alice_addr, token_id))
        .expect_err("should return `EnforcedPause`");
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
async fn transfer_reverts_when_paused(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
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
        .expect_err("should return `EnforcedPause`");
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
async fn safe_transfer_from_reverts_when_paused(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
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
            .expect_err("should return `EnforcedPause`");
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
async fn safe_transfer_from_with_data_reverts_when_paused(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
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
    .expect_err("should return `EnforcedPause`");
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
async fn safe_mint_reverts_when_paused(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();

    let Erc721::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let _ = watch!(contract.pause())?;

    let err = send!(contract.safeMint(
        alice_addr,
        token_id,
        fixed_bytes!("deadbeef").into()
    ))
    .expect_err("should return `EnforcedPause`");
    assert!(err.reverted_with(Erc721::EnforcedPause {}));

    let err = contract
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    let Erc721::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);

    Ok(())
}

// ============================================================================
// Integration Tests: ERC-721 Burnable Extension
// ============================================================================

#[e2e::test]
async fn burn_removes_token_from_owner(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let receipt = receipt!(contract.burn(token_id))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: Address::ZERO,
        tokenId: token_id,
    }));

    let Erc721::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;

    assert_eq!(initial_balance - uint!(1_U256), balance);

    let err = contract
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    Ok(())
}

#[e2e::test]
async fn burn_succeeds_with_token_approval(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.approve(bob_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let receipt = receipt!(contract_bob.burn(token_id))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: Address::ZERO,
        tokenId: token_id,
    }));

    let Erc721::balanceOfReturn { balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    assert_eq!(initial_balance - uint!(1_U256), balance);

    let err = contract_bob
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    Ok(())
}

#[e2e::test]
async fn burn_succeeds_with_approval_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.setApprovalForAll(bob_addr, true))?;

    let Erc721::balanceOfReturn { balance: initial_balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    let receipt = receipt!(contract_bob.burn(token_id))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: Address::ZERO,
        tokenId: token_id,
    }));

    let Erc721::balanceOfReturn { balance } =
        contract_alice.balanceOf(alice_addr).call().await?;

    assert_eq!(initial_balance - uint!(1_U256), balance);

    let err = contract_bob
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    Ok(())
}

#[e2e::test]
async fn burn_reverts_when_approval_insufficient(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr).call().await?;

    let contract = Erc721::new(contract_addr, &bob.wallet);
    let err = send!(contract.burn(token_id))
        .expect_err("should not burn unapproved token");

    assert!(err.reverted_with(Erc721::ERC721InsufficientApproval {
        operator: bob_addr,
        tokenId: token_id,
    }));

    let Erc721::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;

    assert_eq!(initial_balance, balance);

    Ok(())
}

#[e2e::test]
async fn burn_reverts_when_token_nonexistent(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let err = send!(contract.burn(token_id))
        .expect_err("should not burn a non-existent token");
    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );
    Ok(())
}

// ============================================================================
// Integration Tests: ERC-721 Enumerable Extension
// ============================================================================

#[e2e::test]
async fn total_supply_returns_correct_number_of_tokens(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let token_1 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_1))?;

    let token_2 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_2))?;

    let Erc721::totalSupplyReturn { totalSupply } =
        contract.totalSupply().call().await?;

    assert_eq!(uint!(2_U256), totalSupply);

    Ok(())
}

#[e2e::test]
async fn token_of_owner_by_index_reverts_when_index_out_of_bounds(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let _ = watch!(contract.mint(alice_addr, random_token_id()))?;
    let _ = watch!(contract.mint(alice_addr, random_token_id()))?;

    let index_out_of_bound = uint!(2_U256);

    let err = contract
        .tokenOfOwnerByIndex(alice_addr, index_out_of_bound)
        .call()
        .await
        .expect_err("should return `ERC721OutOfBoundsIndex`");

    assert!(err.reverted_with(Erc721::ERC721OutOfBoundsIndex {
        owner: alice_addr,
        index: index_out_of_bound
    }));

    Ok(())
}

#[e2e::test]
async fn token_of_owner_by_index_reverts_when_no_tokens(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let index = uint!(0_U256);

    let err = contract
        .tokenOfOwnerByIndex(alice_addr, index)
        .call()
        .await
        .expect_err("should return `ERC721OutOfBoundsIndex`");

    assert!(err.reverted_with(Erc721::ERC721OutOfBoundsIndex {
        owner: alice_addr,
        index
    }));

    Ok(())
}

#[e2e::test]
async fn token_of_owner_by_index_returns_correct_token_id(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let token_0 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_0))?;

    let token_1 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_1))?;

    let Erc721::tokenOfOwnerByIndexReturn { tokenId } =
        contract.tokenOfOwnerByIndex(alice_addr, uint!(0_U256)).call().await?;
    assert_eq!(token_0, tokenId);

    let Erc721::tokenOfOwnerByIndexReturn { tokenId } =
        contract.tokenOfOwnerByIndex(alice_addr, uint!(1_U256)).call().await?;
    assert_eq!(token_1, tokenId);

    Ok(())
}

#[e2e::test]
async fn token_of_owner_by_index_updates_after_token_transfer(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let token_0 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_0))?;

    let token_1 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_1))?;

    let _ = watch!(contract.transferFrom(alice_addr, bob_addr, token_1))?;
    let _ = watch!(contract.transferFrom(alice_addr, bob_addr, token_0))?;

    // should be in reverse order
    let index = uint!(0_U256);
    let Erc721::tokenOfOwnerByIndexReturn { tokenId } =
        contract.tokenOfOwnerByIndex(bob_addr, index).call().await?;
    assert_eq!(token_1, tokenId);
    let err = contract
        .tokenOfOwnerByIndex(alice_addr, index)
        .call()
        .await
        .expect_err("should return `ERC721OutOfBoundsIndex`");
    assert!(err.reverted_with(Erc721::ERC721OutOfBoundsIndex {
        owner: alice_addr,
        index
    }));

    let index = uint!(1_U256);
    let Erc721::tokenOfOwnerByIndexReturn { tokenId } =
        contract.tokenOfOwnerByIndex(bob_addr, index).call().await?;
    assert_eq!(token_0, tokenId);
    let err = contract
        .tokenOfOwnerByIndex(alice_addr, index)
        .call()
        .await
        .expect_err("should return `ERC721OutOfBoundsIndex`");
    assert!(err.reverted_with(Erc721::ERC721OutOfBoundsIndex {
        owner: alice_addr,
        index
    }));

    let Erc721::totalSupplyReturn { totalSupply } =
        contract.totalSupply().call().await?;

    assert_eq!(uint!(2_U256), totalSupply);

    Ok(())
}

#[e2e::test]
async fn token_by_index_reverts_when_no_tokens(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let index = uint!(0_U256);

    let err = contract
        .tokenByIndex(index)
        .call()
        .await
        .expect_err("should return `ERC721OutOfBoundsIndex`");

    assert!(err.reverted_with(Erc721::ERC721OutOfBoundsIndex {
        owner: Address::ZERO,
        index
    }));

    Ok(())
}

#[e2e::test]
async fn token_by_index_reverts_when_index_out_of_bounds(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let _ = watch!(contract.mint(alice_addr, random_token_id()))?;
    let _ = watch!(contract.mint(alice_addr, random_token_id()))?;

    let index_out_of_bound = uint!(2_U256);

    let err = contract
        .tokenByIndex(index_out_of_bound)
        .call()
        .await
        .expect_err("should return `ERC721OutOfBoundsIndex`");

    assert!(err.reverted_with(Erc721::ERC721OutOfBoundsIndex {
        owner: Address::ZERO,
        index: index_out_of_bound
    }));

    Ok(())
}

#[e2e::test]
async fn token_by_index_returns_correct_token_id(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let token_0 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_0))?;

    let token_1 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_1))?;

    let Erc721::tokenByIndexReturn { tokenId } =
        contract.tokenByIndex(uint!(0_U256)).call().await?;
    assert_eq!(token_0, tokenId);

    let Erc721::tokenByIndexReturn { tokenId } =
        contract.tokenByIndex(uint!(1_U256)).call().await?;
    assert_eq!(token_1, tokenId);

    Ok(())
}

#[e2e::test]
async fn token_by_index_updates_after_token_burn(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let token_0 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_0))?;

    let token_1 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_1))?;

    let _ = watch!(contract.burn(token_1))?;

    let Erc721::tokenByIndexReturn { tokenId } =
        contract.tokenByIndex(uint!(0_U256)).call().await?;
    assert_eq!(token_0, tokenId);

    let index_of_burnt_token = uint!(1_U256);
    let err = contract
        .tokenByIndex(index_of_burnt_token)
        .call()
        .await
        .expect_err("should return `ERC721OutOfBoundsIndex`");

    assert!(err.reverted_with(Erc721::ERC721OutOfBoundsIndex {
        owner: Address::ZERO,
        index: index_of_burnt_token
    }));

    let Erc721::totalSupplyReturn { totalSupply } =
        contract.totalSupply().call().await?;

    assert_eq!(uint!(1_U256), totalSupply);

    Ok(())
}

#[e2e::test]
async fn token_by_index_updates_after_burn_and_new_mints(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let token_0 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_0))?;

    let token_1 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_1))?;

    let _ = watch!(contract.burn(token_1))?;

    let token_2 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_2))?;

    let token_3 = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_3))?;

    let Erc721::tokenByIndexReturn { tokenId } =
        contract.tokenByIndex(uint!(0_U256)).call().await?;
    assert_eq!(token_0, tokenId);

    let Erc721::tokenByIndexReturn { tokenId } =
        contract.tokenByIndex(uint!(1_U256)).call().await?;
    assert_eq!(token_2, tokenId);

    let Erc721::tokenByIndexReturn { tokenId } =
        contract.tokenByIndex(uint!(2_U256)).call().await?;
    assert_eq!(token_3, tokenId);

    Ok(())
}

// ============================================================================
// Integration Tests: ERC-165 Support Interface
// ============================================================================

#[e2e::test]
async fn supports_interface_returns_correct_interface_ids(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);
    let invalid_interface_id: u32 = 0x_ffffffff;
    let Erc721::supportsInterfaceReturn {
        supportsInterface: supports_interface,
    } = contract.supportsInterface(invalid_interface_id.into()).call().await?;

    assert!(!supports_interface);

    let erc721_interface_id: u32 = 0x80ac58cd;
    let Erc721::supportsInterfaceReturn {
        supportsInterface: supports_interface,
    } = contract.supportsInterface(erc721_interface_id.into()).call().await?;

    assert!(supports_interface);

    let erc165_interface_id: u32 = 0x01ffc9a7;
    let Erc721::supportsInterfaceReturn {
        supportsInterface: supports_interface,
    } = contract.supportsInterface(erc165_interface_id.into()).call().await?;

    assert!(supports_interface);

    let erc721_enumerable_interface_id: u32 = 0x780e9d63;
    let Erc721::supportsInterfaceReturn {
        supportsInterface: supports_interface,
    } = contract
        .supportsInterface(erc721_enumerable_interface_id.into())
        .call()
        .await?;

    assert!(supports_interface);

    Ok(())
}
