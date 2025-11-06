#![cfg(feature = "e2e")]
#![allow(clippy::unreadable_literal)]

use abi::Erc721;
use alloy::{
    primitives::{aliases::B32, fixed_bytes, uint, Address, Bytes, U256},
    sol_types::SolError,
};
use e2e::{receipt, send, watch, Account, EventExt, PanicCode, Revert};
use mock::{receiver, receiver::ERC721ReceiverMock};

mod abi;
mod mock;

fn random_token_id() -> U256 {
    let num: u32 = rand::random();
    U256::from(num)
}

trait EncodeAsStr {
    fn encode_as_str(&self) -> String;
}

impl<T: SolError> EncodeAsStr for T {
    fn encode_as_str(&self) -> String {
        let expected_error = self.abi_encode();
        String::from_utf8_lossy(&expected_error).to_string()
    }
}

// ============================================================================
// Integration Tests: ERC-721 Token
// ============================================================================

#[e2e::test]
async fn error_when_checking_balance_of_invalid_owner(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn balance_of_zero_balance(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let Erc721::balanceOfReturn { balance } =
        contract.balanceOf(alice.address()).call().await?;
    assert_eq!(U256::ZERO, balance);

    Ok(())
}

#[e2e::test]
async fn error_when_checking_owner_of_nonexistent_token(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn mints(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
    assert_eq!(U256::ONE, balance);

    Ok(())
}

#[e2e::test]
async fn error_when_minting_token_id_twice(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();
    watch!(contract.mint(alice_addr, token_id))?;

    let err = send!(contract.mint(alice_addr, token_id))
        .expect_err("should not mint a token id twice");
    assert!(err
        .reverted_with(Erc721::ERC721InvalidSender { sender: Address::ZERO }));

    Ok(())
}

#[e2e::test]
async fn error_when_minting_token_to_invalid_receiver(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn transfers_from(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    watch!(contract.mint(alice_addr, token_id))?;

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

    let one = U256::ONE;
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn transfers_from_approved_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    watch!(contract_alice.mint(alice_addr, token_id))?;
    watch!(contract_alice.approve(bob_addr, token_id))?;

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

    let one = U256::ONE;
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn transfers_from_approved_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    watch!(contract_alice.mint(alice_addr, token_id))?;
    watch!(contract_alice.setApprovalForAll(bob_addr, true))?;

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

    let one = U256::ONE;
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn error_when_transfer_to_invalid_receiver(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let invalid_receiver = Address::ZERO;
    let token_id = random_token_id();

    watch!(contract.mint(alice_addr, token_id))?;

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
async fn error_when_transfer_from_incorrect_owner(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();

    let token_id = random_token_id();

    watch!(contract.mint(alice_addr, token_id))?;

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
async fn error_when_transfer_with_insufficient_approval(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    watch!(contract.mint(alice_addr, token_id))?;

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
async fn error_when_transfer_nonexistent_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn safe_transfers_from(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    watch!(contract.mint(alice_addr, token_id))?;

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

    let one = U256::ONE;
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfers_to_receiver_contract(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address =
        receiver::deploy(&alice.wallet, ERC721ReceiverMock::RevertType::None)
            .await?;

    let alice_addr = alice.address();
    let token_id = random_token_id();

    watch!(contract.mint(alice_addr, token_id))?;

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

    let one = U256::ONE;
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_receiver_balance + one, receiver_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfers_from_approved_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    watch!(contract_alice.mint(alice_addr, token_id))?;
    watch!(contract_alice.approve(bob_addr, token_id))?;

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

    let one = U256::ONE;
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfers_from_approved_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    watch!(contract_alice.mint(alice_addr, token_id))?;
    watch!(contract_alice.setApprovalForAll(bob_addr, true))?;

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

    let one = U256::ONE;
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn error_when_safe_transfer_to_invalid_receiver(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let invalid_receiver = Address::ZERO;
    let token_id = random_token_id();

    watch!(contract.mint(alice_addr, token_id))?;

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
async fn error_when_safe_transfer_from_incorrect_owner(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();

    let token_id = random_token_id();

    watch!(contract.mint(alice_addr, token_id))?;

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
async fn error_when_safe_transfer_with_insufficient_approval(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    watch!(contract.mint(alice_addr, token_id))?;

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
async fn error_when_safe_transfer_nonexistent_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn safe_transfers_from_with_data(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    watch!(contract.mint(alice_addr, token_id))?;

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

    let one = U256::ONE;
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfers_with_data_to_receiver_contract(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address =
        receiver::deploy(&alice.wallet, ERC721ReceiverMock::RevertType::None)
            .await?;

    let alice_addr = alice.address();
    let token_id = random_token_id();
    let data: Bytes = fixed_bytes!("deadbeef").into();

    watch!(contract.mint(alice_addr, token_id))?;

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

    let one = U256::ONE;
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_receiver_balance + one, receiver_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfers_from_with_data_approved_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    watch!(contract_alice.mint(alice_addr, token_id))?;
    watch!(contract_alice.approve(bob_addr, token_id))?;

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

    let one = U256::ONE;
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfers_from_with_data_approved_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    watch!(contract_alice.mint(alice_addr, token_id))?;
    watch!(contract_alice.setApprovalForAll(bob_addr, true))?;

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

    let one = U256::ONE;
    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn error_when_safe_transfer_with_data_to_invalid_receiver(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let invalid_receiver = Address::ZERO;
    let token_id = random_token_id();

    watch!(contract.mint(alice_addr, token_id))?;

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
async fn error_when_safe_transfer_with_data_from_incorrect_owner(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();

    let token_id = random_token_id();

    watch!(contract.mint(alice_addr, token_id))?;

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
async fn error_when_safe_transfer_with_data_with_insufficient_approval(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    watch!(contract.mint(alice_addr, token_id))?;

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
async fn error_when_safe_transfer_with_data_nonexistent_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn errors_when_receiver_reverts_with_reason(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC721ReceiverMock::RevertType::RevertWithMessage,
    )
    .await?;

    let alice_addr = alice.address();
    let token_id = random_token_id();

    watch!(contract.mint(alice_addr, token_id))?;

    let err = send!(contract.safeTransferFrom_0(
        alice_addr,
        receiver_address,
        token_id
    ))
    .expect_err("should not transfer when receiver errors with reason");

    let message =
        Erc721::Error { message: "ERC721ReceiverMock: reverting".to_string() }
            .encode_as_str();

    assert!(err.reverted_with(Erc721::InvalidReceiverWithReason { message }));

    Ok(())
}

#[e2e::test]
async fn errors_when_receiver_reverts_without_reason(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC721ReceiverMock::RevertType::RevertWithoutMessage,
    )
    .await?;

    let alice_addr = alice.address();
    let token_id = random_token_id();

    watch!(contract.mint(alice_addr, token_id))?;

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
async fn errors_when_receiver_panics(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address =
        receiver::deploy(&alice.wallet, ERC721ReceiverMock::RevertType::Panic)
            .await?;

    let alice_addr = alice.address();
    let token_id = random_token_id();

    watch!(contract.mint(alice_addr, token_id))?;

    let err = send!(contract.safeTransferFrom_0(
        alice_addr,
        receiver_address,
        token_id
    ))
    .expect_err("should not transfer when receiver panics");

    let message =
        Erc721::Panic { code: U256::from(PanicCode::DivisionByZero as u8) }
            .encode_as_str();
    assert!(err.reverted_with(Erc721::InvalidReceiverWithReason { message }));

    Ok(())
}

#[e2e::test]
async fn approves(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    watch!(contract.mint(alice_addr, token_id))?;

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
async fn error_when_approve_for_nonexistent_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn error_when_approve_by_invalid_approver(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    watch!(contract_alice.mint(alice_addr, token_id))?;

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
async fn error_when_checking_approved_of_nonexistent_token(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn sets_approval_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn error_when_set_approval_for_all_by_invalid_operator(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn is_approved_for_all_invalid_operator(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn safe_mint_to_eoa_without_data(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
    assert_eq!(balance, initial_balance + U256::ONE);

    Ok(())
}

#[e2e::test]
async fn safe_mint_to_eoa_with_data(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
    assert_eq!(balance, initial_balance + U256::ONE);

    Ok(())
}

#[e2e::test]
async fn safe_mint_to_receiver_contract_without_data(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
    assert_eq!(balance, initial_balance + U256::ONE);

    Ok(())
}

#[e2e::test]
async fn safe_mint_to_receiver_contract_with_data(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
    assert_eq!(balance, initial_balance + U256::ONE);

    Ok(())
}

#[e2e::test]
async fn error_when_safe_mint_to_invalid_receiver_contract(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn error_when_safe_mint_to_invalid_sender_with_data(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();
    let data: Bytes = fixed_bytes!("deadbeef").into();

    watch!(contract.mint(alice.address(), token_id))?;

    let err = send!(contract.safeMint(bob.address(), token_id, data))
        .expect_err("should not safe mint an existing token");

    assert!(err
        .reverted_with(Erc721::ERC721InvalidSender { sender: Address::ZERO }));

    Ok(())
}

#[e2e::test]
async fn error_when_receiver_reverts_with_reason_on_safe_mint_with_data(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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

    let message =
        Erc721::Error { message: "ERC721ReceiverMock: reverting".to_string() }
            .encode_as_str();

    assert!(err.reverted_with(Erc721::InvalidReceiverWithReason { message }));

    Ok(())
}

#[e2e::test]
async fn error_when_receiver_reverts_without_reason_on_safe_mint_with_data(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn error_when_receiver_panics_on_safe_mint_with_data(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let receiver_address =
        receiver::deploy(&alice.wallet, ERC721ReceiverMock::RevertType::Panic)
            .await?;

    let token_id = random_token_id();
    let data: Bytes = fixed_bytes!("deadbeef").into();

    let err = send!(contract.safeMint(receiver_address, token_id, data))
        .expect_err("should not safe mint when receiver panics");

    let message =
        Erc721::Panic { code: U256::from(PanicCode::DivisionByZero as u8) }
            .encode_as_str();
    assert!(err.reverted_with(Erc721::InvalidReceiverWithReason { message }));

    Ok(())
}

// ============================================================================
// Integration Tests: ERC-721 Burnable Extension
// ============================================================================

#[e2e::test]
async fn burns(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();
    watch!(contract.mint(alice_addr, token_id))?;

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

    assert_eq!(initial_balance - U256::ONE, balance);

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
async fn burns_approved_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    watch!(contract_alice.mint(alice_addr, token_id))?;
    watch!(contract_alice.approve(bob_addr, token_id))?;

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

    assert_eq!(initial_balance - U256::ONE, balance);

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
async fn burns_approved_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    watch!(contract_alice.mint(alice_addr, token_id))?;
    watch!(contract_alice.setApprovalForAll(bob_addr, true))?;

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

    assert_eq!(initial_balance - U256::ONE, balance);

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
async fn error_when_burn_with_insufficient_approval(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    watch!(contract.mint(alice_addr, token_id))?;

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
async fn error_when_burn_nonexistent_token(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
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
async fn totally_supply_works(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let token_1 = random_token_id();
    watch!(contract.mint(alice_addr, token_1))?;

    let token_2 = random_token_id();
    watch!(contract.mint(alice_addr, token_2))?;

    let Erc721::totalSupplyReturn { totalSupply } =
        contract.totalSupply().call().await?;

    assert_eq!(uint!(2_U256), totalSupply);

    Ok(())
}

#[e2e::test]
async fn error_when_checking_token_of_owner_by_index_out_of_bound(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    watch!(contract.mint(alice_addr, random_token_id()))?;
    watch!(contract.mint(alice_addr, random_token_id()))?;

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
async fn error_when_checking_token_of_owner_by_index_account_has_no_tokens(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let index = U256::ZERO;

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
async fn token_of_owner_by_index_works(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let token_0 = random_token_id();
    watch!(contract.mint(alice_addr, token_0))?;

    let token_1 = random_token_id();
    watch!(contract.mint(alice_addr, token_1))?;

    let Erc721::tokenOfOwnerByIndexReturn { tokenId } =
        contract.tokenOfOwnerByIndex(alice_addr, U256::ZERO).call().await?;
    assert_eq!(token_0, tokenId);

    let Erc721::tokenOfOwnerByIndexReturn { tokenId } =
        contract.tokenOfOwnerByIndex(alice_addr, U256::ONE).call().await?;
    assert_eq!(token_1, tokenId);

    Ok(())
}

#[e2e::test]
async fn token_of_owner_by_index_after_transfer_to_another_account(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let token_0 = random_token_id();
    watch!(contract.mint(alice_addr, token_0))?;

    let token_1 = random_token_id();
    watch!(contract.mint(alice_addr, token_1))?;

    watch!(contract.transferFrom(alice_addr, bob_addr, token_1))?;
    watch!(contract.transferFrom(alice_addr, bob_addr, token_0))?;

    // should be in reverse order
    let index = U256::ZERO;
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

    let index = U256::ONE;
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
async fn error_when_checking_token_by_index_account_has_no_tokens(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let index = U256::ZERO;

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
async fn error_when_checking_token_by_index_out_of_bound(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    watch!(contract.mint(alice_addr, random_token_id()))?;
    watch!(contract.mint(alice_addr, random_token_id()))?;

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
async fn token_by_index_works(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let token_0 = random_token_id();
    watch!(contract.mint(alice_addr, token_0))?;

    let token_1 = random_token_id();
    watch!(contract.mint(alice_addr, token_1))?;

    let Erc721::tokenByIndexReturn { tokenId } =
        contract.tokenByIndex(U256::ZERO).call().await?;
    assert_eq!(token_0, tokenId);

    let Erc721::tokenByIndexReturn { tokenId } =
        contract.tokenByIndex(U256::ONE).call().await?;
    assert_eq!(token_1, tokenId);

    Ok(())
}

#[e2e::test]
async fn token_by_index_after_burn(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let token_0 = random_token_id();
    watch!(contract.mint(alice_addr, token_0))?;

    let token_1 = random_token_id();
    watch!(contract.mint(alice_addr, token_1))?;

    watch!(contract.burn(token_1))?;

    let Erc721::tokenByIndexReturn { tokenId } =
        contract.tokenByIndex(U256::ZERO).call().await?;
    assert_eq!(token_0, tokenId);

    let index_of_burnt_token = U256::ONE;
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

    assert_eq!(U256::ONE, totalSupply);

    Ok(())
}

#[e2e::test]
async fn token_by_index_after_burn_and_some_mints(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let token_0 = random_token_id();
    watch!(contract.mint(alice_addr, token_0))?;

    let token_1 = random_token_id();
    watch!(contract.mint(alice_addr, token_1))?;

    watch!(contract.burn(token_1))?;

    let token_2 = random_token_id();
    watch!(contract.mint(alice_addr, token_2))?;

    let token_3 = random_token_id();
    watch!(contract.mint(alice_addr, token_3))?;

    let Erc721::tokenByIndexReturn { tokenId } =
        contract.tokenByIndex(U256::ZERO).call().await?;
    assert_eq!(token_0, tokenId);

    let Erc721::tokenByIndexReturn { tokenId } =
        contract.tokenByIndex(U256::ONE).call().await?;
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
async fn supports_interface(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc721::new(contract_addr, &alice.wallet);
    let invalid_interface_id: B32 = 0xffffffff_u32.into();
    let Erc721::supportsInterfaceReturn {
        supportsInterface: supports_interface,
    } = contract.supportsInterface(invalid_interface_id).call().await?;

    assert!(!supports_interface);

    let erc721_interface_id: B32 = 0x80ac58cd_u32.into();
    let Erc721::supportsInterfaceReturn {
        supportsInterface: supports_interface,
    } = contract.supportsInterface(erc721_interface_id).call().await?;

    assert!(supports_interface);

    let erc165_interface_id: B32 = 0x01ffc9a7_u32.into();
    let Erc721::supportsInterfaceReturn {
        supportsInterface: supports_interface,
    } = contract.supportsInterface(erc165_interface_id).call().await?;

    assert!(supports_interface);

    let erc721_enumerable_interface_id: B32 = 0x780e9d63_u32.into();
    let Erc721::supportsInterfaceReturn {
        supportsInterface: supports_interface,
    } = contract
        .supportsInterface(erc721_enumerable_interface_id)
        .call()
        .await?;

    assert!(supports_interface);

    Ok(())
}
