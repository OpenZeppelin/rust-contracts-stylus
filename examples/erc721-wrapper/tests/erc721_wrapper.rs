#![cfg(feature = "e2e")]

use abi::{Erc721, Erc721Wrapper};
use alloy::primitives::{Address, U256};
use e2e::{
    constructor, receipt, watch, Account, Constructor, EventExt, Revert,
};
use eyre::Result;

mod abi;
mod mock;
use mock::{erc721, erc721::ERC721Mock};

fn ctr(asset_addr: Address) -> Constructor {
    constructor!(asset_addr)
}

async fn deploy(account: &Account) -> Result<(Address, Address)> {
    let asset_addr = erc721::deploy(&account.wallet).await?;

    let contract_addr = account
        .as_deployer()
        .with_constructor(ctr(asset_addr))
        .deploy()
        .await?
        .contract_address;

    Ok((asset_addr, contract_addr))
}

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let asset_address = erc721::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .contract_address;
    let contract = Erc721Wrapper::new(contract_addr, alice.wallet);

    let underlying = contract.underlying().call().await?.underlying;
    assert_eq!(underlying, asset_address);

    Ok(())
}

#[e2e::test]
async fn deposit_for_success(alice: Account) -> Result<()> {
    let token_id = U256::ONE;
    let (asset_addr, contract_addr) = deploy(&alice).await?;
    let alice_address = alice.address();
    let asset = ERC721Mock::new(asset_addr, &alice.wallet);
    let contract = Erc721Wrapper::new(contract_addr, &alice.wallet);

    watch!(asset.safeMint(alice_address, token_id))?;
    watch!(asset.approve(contract_addr, token_id))?;

    let initial_alice_balance = asset.balanceOf(alice_address).call().await?._0;
    let initial_contract_balance =
        asset.balanceOf(contract_addr).call().await?._0;
    let initial_wrapped_balance =
        contract.balanceOf(alice_address).call().await?.balance;

    let receipt = receipt!(contract.depositFor(alice_address, vec![token_id]))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_address,
        to: contract_addr,
        tokenId: token_id,
    }));

    assert!(receipt.emits(Erc721Wrapper::Transfer {
        from: Address::ZERO,
        to: alice_address,
        tokenId: token_id,
    }));

    let wrapped_owner = contract.ownerOf(token_id).call().await?.owner;
    assert_eq!(wrapped_owner, alice_address);

    let underlying_owner = asset.ownerOf(token_id).call().await?._0;
    assert_eq!(underlying_owner, contract_addr);

    let one = U256::ONE;

    assert_eq!(
        initial_alice_balance - one,
        asset.balanceOf(alice_address).call().await?._0
    );
    assert_eq!(
        initial_contract_balance + one,
        asset.balanceOf(contract_addr).call().await?._0
    );
    assert_eq!(
        initial_wrapped_balance + one,
        contract.balanceOf(alice_address).call().await?.balance
    );

    Ok(())
}

#[e2e::test]
async fn withdraw_to_success(alice: Account) -> Result<()> {
    let token_id = U256::ONE;
    let (asset_addr, contract_addr) = deploy(&alice).await?;
    let alice_address = alice.address();
    let asset = ERC721Mock::new(asset_addr, &alice.wallet);
    let contract = Erc721Wrapper::new(contract_addr, &alice.wallet);

    watch!(asset.safeMint(alice_address, token_id))?;
    watch!(asset.approve(contract_addr, token_id))?;
    watch!(contract.depositFor(alice_address, vec![token_id]))?;

    let initial_alice_balance = asset.balanceOf(alice_address).call().await?._0;
    let initial_contract_balance =
        asset.balanceOf(contract_addr).call().await?._0;
    let initial_wrapped_balance =
        contract.balanceOf(alice_address).call().await?.balance;

    let receipt = receipt!(contract.withdrawTo(alice_address, vec![token_id]))?;

    assert!(receipt.emits(Erc721Wrapper::Transfer {
        from: alice_address,
        to: Address::ZERO,
        tokenId: token_id,
    }));

    assert!(receipt.emits(Erc721::Transfer {
        from: contract_addr,
        to: alice_address,
        tokenId: token_id,
    }));

    let err = contract
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(err.reverted_with(Erc721Wrapper::ERC721NonexistentToken {
        tokenId: token_id
    }));

    let underlying_owner = asset.ownerOf(token_id).call().await?._0;
    assert_eq!(underlying_owner, alice_address);

    let one = U256::ONE;

    assert_eq!(
        initial_alice_balance + one,
        asset.balanceOf(alice_address).call().await?._0
    );
    assert_eq!(
        initial_contract_balance - one,
        asset.balanceOf(contract_addr).call().await?._0
    );
    assert_eq!(
        initial_wrapped_balance - one,
        contract.balanceOf(alice_address).call().await?.balance
    );

    Ok(())
}
