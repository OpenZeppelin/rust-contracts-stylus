#![cfg(feature = "e2e")]

use alloy::primitives::uint;
use alloy_primitives::U256;
use e2e::{receipt, send, watch, Account, ReceiptExt, Revert};

use abi::SafeErc20;
use mock::{erc20, erc20::ERC20Mock};

mod abi;
mod mock;

#[e2e::test]
async fn safe_transfers(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = SafeErc20::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let balance = uint!(10_U256);
    let value = uint!(1_U256);

    let erc20_address = erc20::deploy(&alice.wallet).await?;
    let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

    let _ = watch!(erc20_alice.mint(alice_addr, balance));

    let ERC20Mock::balanceOfReturn { _0: initial_alice_balance } =
        erc20_alice.balanceOf(alice_addr).call().await?;
    let ERC20Mock::balanceOfReturn { _0: initial_bob_balance } =
        erc20_alice.balanceOf(bob_addr).call().await?;
    assert_eq!(initial_alice_balance, balance);
    assert_eq!(initial_bob_balance, U256::ZERO);

    let err =
        send!(contract_alice.safeTransfer(erc20_address, bob_addr, value))
            .expect_err("should err I guess");
    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: erc20_address,
    }));

    let ERC20Mock::balanceOfReturn { _0: alice_balance } =
        erc20_alice.balanceOf(alice_addr).call().await?;
    let ERC20Mock::balanceOfReturn { _0: bob_balance } =
        erc20_alice.balanceOf(bob_addr).call().await?;

    assert_eq!(initial_alice_balance - value, alice_balance);
    assert_eq!(initial_bob_balance + value, bob_balance);

    Ok(())
}
