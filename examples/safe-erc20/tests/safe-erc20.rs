#![cfg(feature = "e2e")]

use alloy::primitives::uint;
use e2e::{watch, Account, ReceiptExt};

use abi::SafeErc20Example;
use mock::{erc20, erc20::ERC20Mock};

mod abi;
mod mock;

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = SafeErc20Example::new(contract_addr, &alice.wallet);

    let erc20_address = erc20::deploy(&alice.wallet).await?;
    let erc20 = ERC20Mock::new(erc20_address, &alice.wallet);
    let one = uint!(1_U256);
    let _ = watch!(erc20.mint(alice.address(), one));
    let balance = erc20.balanceOf(alice.address()).call().await?._0;
    assert_eq!(balance, one);

    Ok(())
}

// #[e2e::test]
// async fn transfers_from(alice: Account, bob: Account) -> eyre::Result<()> {
//     let receivers = vec![alice.address(), bob.address()];
//     let amounts = vec![1000_u128, 1000_u128];
//     // Deploy and mint batches of 1000 tokens to Alice and Bob.
//     let receipt = alice
//         .as_deployer()
//         .with_constructor(ctr(receivers, amounts))
//         .deploy()
//         .await?;
//     let contract = Erc721::new(receipt.address()?, &alice.wallet);

//     let first_consecutive_token_id = U256::from(FIRST_CONSECUTIVE_ID);

//     // Transfer first consecutive token from Alice to Bob.
//     let _ = watch!(contract.transferFrom(
//         alice.address(),
//         bob.address(),
//         first_consecutive_token_id
//     ))?;

//     let Erc721::ownerOfReturn { ownerOf } =
//         contract.ownerOf(first_consecutive_token_id).call().await?;
//     assert_eq!(ownerOf, bob.address());

//     // Check that balances changed.
//     let Erc721::balanceOfReturn { balance: alice_balance } =
//         contract.balanceOf(alice.address()).call().await?;
//     assert_eq!(alice_balance, uint!(1000_U256) - uint!(1_U256));
//     let Erc721::balanceOfReturn { balance: bob_balance } =
//         contract.balanceOf(bob.address()).call().await?;
//     assert_eq!(bob_balance, uint!(1000_U256) + uint!(1_U256));

//     // Test non-consecutive mint.
//     let token_id = random_token_id();
//     let _ = watch!(contract.mint(alice.address(), token_id))?;
//     let Erc721::balanceOfReturn { balance: alice_balance } =
//         contract.balanceOf(alice.address()).call().await?;
//     assert_eq!(alice_balance, uint!(1000_U256));

//     // Test transfer of the token that wasn't minted consecutive.
//     let _ = watch!(contract.transferFrom(
//         alice.address(),
//         bob.address(),
//         token_id
//     ))?;
//     let Erc721::balanceOfReturn { balance: alice_balance } =
//         contract.balanceOf(alice.address()).call().await?;
//     assert_eq!(alice_balance, uint!(1000_U256) - uint!(1_U256));
//     Ok(())
// }
