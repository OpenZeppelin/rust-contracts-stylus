use ethers::prelude::*;
use eyre::{bail, Result};

use crate::infrastructure::{erc20::*, *};

#[tokio::test]
async fn mint() -> Result<()> {
    let infra = Infrastructure::<Erc20>::new().await?;
    let one = U256::from(1);

    let initial_balance =
        infra.alice.balance_of(infra.alice.wallet.address()).ctx_call().await?;
    let initial_supply = infra.alice.total_supply().ctx_call().await?;

    let _ =
        infra.alice.mint(infra.alice.wallet.address(), one).ctx_send().await?;

    let new_balance =
        infra.alice.balance_of(infra.alice.wallet.address()).ctx_call().await?;
    let new_supply = infra.alice.total_supply().ctx_call().await?;

    assert_eq!(initial_balance + one, new_balance);
    assert_eq!(initial_supply + one, new_supply);
    Ok(())
}

// TODO: add rest of the tests for erc20 base implementation
