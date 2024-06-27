#![cfg(feature = "e2e")]

use alloy::{
    primitives::{fixed_bytes, Address, Bytes, U256},
    sol,
    sol_types::SolConstructor,
};
use alloy_primitives::uint;
use e2e::{receipt, send, watch, Account, EventExt, Revert};

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

    assert_eq!(TOKEN_NAME.to_owned(), name);
    assert_eq!(TOKEN_SYMBOL.to_owned(), symbol);

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
        .expect_err("should return `ERC721InvalidOwner`");
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
async fn error_when_owner_of_nonexistent_token(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
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
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::ownerOfReturn { ownerOf: owner_of } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, owner_of);

    let Erc721::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr).call().await?;
    assert_eq!(uint!(1_U256), balance);

    Ok(())
}

#[e2e::test]
async fn error_when_minting_token_id_twice(alice: Account) -> eyre::Result<()> {
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
async fn error_when_minting_token_invalid_receiver(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
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
    assert_eq!(bob_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn transfers_from_approved_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.approve(bob_addr, token_id))?;

    let receipt =
        receipt!(contract_bob.transferFrom(alice_addr, bob_addr, token_id))?;

    receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    });

    let Erc721::ownerOfReturn { ownerOf } =
        contract_alice.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn transfers_from_approved_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.setApprovalForAll(bob_addr, true))?;

    let receipt =
        receipt!(contract_bob.transferFrom(alice_addr, bob_addr, token_id))?;

    receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    });

    let Erc721::ownerOfReturn { ownerOf } =
        contract_alice.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn error_when_transfer_from_transfers_to_invalid_receiver(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
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
async fn error_when_transfer_from_transfers_from_incorrect_owner(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
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
async fn error_when_transfer_from_transfers_with_insufficient_approval(
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
    let err = send!(contract.transferFrom(alice_addr, bob_addr, token_id))
        .expect_err("should not transfer unapproved token");

    assert!(err.reverted_with(Erc721::ERC721InsufficientApproval {
        operator: bob_addr,
        tokenId: token_id,
    }));

    Ok(())
}

#[e2e::test]
async fn error_when_transfer_nonexistent_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
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
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let receipt =
        receipt!(contract.safeTransferFrom_0(alice_addr, bob_addr, token_id))?;

    receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    });

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

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
    assert_eq!(receiver_address, ownerOf);

    Ok(())
}

#[e2e::test]
async fn safe_transfers_from_approved_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.approve(bob_addr, token_id))?;

    let receipt = receipt!(
        contract_bob.safeTransferFrom_0(alice_addr, bob_addr, token_id)
    )?;

    receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    });

    let Erc721::ownerOfReturn { ownerOf } =
        contract_alice.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn safe_transfers_from_approved_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.setApprovalForAll(bob_addr, true))?;

    let receipt = receipt!(
        contract_bob.safeTransferFrom_0(alice_addr, bob_addr, token_id)
    )?;

    receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    });

    let Erc721::ownerOfReturn { ownerOf } =
        contract_alice.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn error_when_safe_transfer_from_transfers_to_invalid_receiver(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
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
async fn error_when_safe_transfer_from_transfers_from_incorrect_owner(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
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
async fn error_when_safe_transfer_from_transfers_with_insufficient_approval(
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

    let err =
        send!(contract.safeTransferFrom_0(alice_addr, bob_addr, token_id))
            .expect_err("should not transfer unapproved token");

    assert!(err.reverted_with(Erc721::ERC721InsufficientApproval {
        operator: bob_addr,
        tokenId: token_id,
    }));

    Ok(())
}

#[e2e::test]
async fn error_when_safe_transfer_nonexistent_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
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

// TODO: Test reverts & panics on ERC721ReceiverMock for
// `Erc721::safeTransferFrom_0`.

#[e2e::test]
async fn safe_transfers_from_with_data(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();
    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let receipt = receipt!(contract.safeTransferFrom_1(
        alice_addr,
        bob_addr,
        token_id,
        fixed_bytes!("deadbeef").into()
    ))?;

    receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    });

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn safe_transfers_with_data_to_receiver_contract(
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
    let data: Bytes = fixed_bytes!("deadbeef").into();

    let _ = watch!(contract.mint(alice_addr, token_id))?;
    let receipt = receipt!(contract.safeTransferFrom_1(
        alice_addr,
        receiver_address,
        token_id,
        data.clone()
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
        data,
    });

    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(receiver_address, ownerOf);

    Ok(())
}

#[e2e::test]
async fn safe_transfers_from_with_data_approved_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.approve(bob_addr, token_id))?;

    let receipt = receipt!(contract_bob.safeTransferFrom_1(
        alice_addr,
        bob_addr,
        token_id,
        fixed_bytes!("deadbeef").into()
    ))?;

    receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    });

    let Erc721::ownerOfReturn { ownerOf } =
        contract_alice.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn safe_transfers_from_with_data_approved_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract_alice = Erc721::new(contract_addr, &alice.wallet);
    let contract_bob = Erc721::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract_alice.mint(alice_addr, token_id))?;
    let _ = watch!(contract_alice.setApprovalForAll(bob_addr, true))?;

    let receipt = receipt!(contract_bob.safeTransferFrom_1(
        alice_addr,
        bob_addr,
        token_id,
        fixed_bytes!("deadbeef").into()
    ))?;

    receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: bob_addr,
        tokenId: token_id,
    });

    let Erc721::ownerOfReturn { ownerOf } =
        contract_alice.ownerOf(token_id).call().await?;
    assert_eq!(bob_addr, ownerOf);

    Ok(())
}

#[e2e::test]
async fn error_when_safe_transfer_from_with_data_transfers_to_invalid_receiver(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
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
async fn error_when_safe_transfer_from_with_data_transfers_from_incorrect_owner(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
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
async fn error_when_safe_transfer_from_with_data_transfers_with_insufficient_approval(
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

    Ok(())
}

#[e2e::test]
async fn error_when_safe_transfer_with_data_nonexistent_token(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
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

// TODO: Test reverts & panics on ERC721ReceiverMock for
// `Erc721::safeTransferFrom_1`.

#[e2e::test]
async fn approves(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice_addr, token_id))?;

    let Erc721::getApprovedReturn { approved } =
        contract.getApproved(token_id).call().await?;
    assert_eq!(Address::ZERO, approved);

    let receipt = receipt!(contract.approve(bob_addr, token_id))?;

    receipt.emits(Erc721::Approval {
        owner: alice_addr,
        approved: bob_addr,
        tokenId: token_id,
    });

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
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
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
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
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
