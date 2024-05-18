use ethers::prelude::*;
use eyre::Result;

use crate::infrastructure::{erc20::*, *};

#[tokio::test]
async fn mint() -> Result<()> {
    let Infrastructure { alice, bob } = Infrastructure::<Erc20>::new().await?;
    // TODO: have a nicer support for custom constructors
    let _ = alice
        .constructor("MyErc20".to_string(), "MRC".to_string(), U256::from(10))
        .ctx_send()
        .await?;
    let one = U256::from(1);

    let initial_balance =
        alice.balance_of(alice.wallet.address()).ctx_call().await?;
    let initial_supply = alice.total_supply().ctx_call().await?;

    let _ = alice.mint(alice.wallet.address(), one).ctx_send().await?;

    let new_balance =
        alice.balance_of(alice.wallet.address()).ctx_call().await?;
    let new_supply = alice.total_supply().ctx_call().await?;

    assert_eq!(initial_balance + one, new_balance);
    assert_eq!(initial_supply + one, new_supply);
    Ok(())
}

// TODO: add rest of the tests for erc20 base implementation
