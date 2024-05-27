use e2e_grip::prelude::*;

use crate::abi::erc20::*;

#[e2e_grip::test]
async fn mint(alice: User) -> Result<()> {
    let erc20 = &alice.deploys::<Erc20>().await?;
    // TODO: have a nicer support for custom constructors
    let _ = alice
        .uses(erc20)
        .constructor(
            "MyErc20".to_string(),
            "MRC".to_string(),
            U256::from(10),
            false,
        )
        .ctx_send()
        .await?;
    let one = U256::from(1);

    let initial_balance =
        alice.uses(erc20).balance_of(alice.address()).ctx_call().await?;
    let initial_supply = alice.uses(erc20).total_supply().ctx_call().await?;

    let _ = alice.uses(erc20).mint(alice.address(), one).ctx_send().await?;

    let new_balance =
        alice.uses(erc20).balance_of(alice.address()).ctx_call().await?;
    let new_supply = alice.uses(erc20).total_supply().ctx_call().await?;

    assert_eq!(initial_balance + one, new_balance);
    assert_eq!(initial_supply + one, new_supply);
    Ok(())
}

// TODO: add rest of the tests for erc20 base implementation
