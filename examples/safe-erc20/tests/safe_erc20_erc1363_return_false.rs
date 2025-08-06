#![cfg(feature = "e2e")]

use abi::SafeErc20;
use alloy::primitives::uint;
use alloy_primitives::{Bytes, U256};
use e2e::{send, watch, Account, Revert};
use mock::{
    erc1363_receiver, erc1363_return_false,
    erc1363_return_false::ERC1363ReturnFalseMock, erc1363_spender,
};

mod abi;
mod mock;

const DATA: Bytes = Bytes::from_static(b"0x12345678");

#[e2e::test]
async fn reverts_on_transfer_and_call_relaxed(
    alice: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);

    let value = uint!(10_U256);

    let erc20_address = erc1363_return_false::deploy(&alice.wallet).await?;
    let erc20_alice = ERC1363ReturnFalseMock::new(erc20_address, &alice.wallet);

    // Deploy ERC1363Receiver mock
    let receiver_address = erc1363_receiver::deploy(&alice.wallet).await?;

    // Mint tokens to the SafeERC20 contract
    watch!(erc20_alice.mint(safe_erc20_addr, value))?;

    // Use the relaxed helper method
    let err = send!(safe_erc20_alice.transferAndCallRelaxed(
        erc20_address,
        receiver_address,
        value,
        DATA,
    ))
    .expect_err("should revert with SafeERC20FailedOperation");

    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: erc20_address,
    }));

    Ok(())
}

#[e2e::test]
async fn reverts_on_transfer_from_and_call_relaxed(
    alice: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);

    let value = uint!(10_U256);

    let erc20_address = erc1363_return_false::deploy(&alice.wallet).await?;
    let erc20_alice = ERC1363ReturnFalseMock::new(erc20_address, &alice.wallet);

    // Deploy ERC1363Receiver mock
    let receiver_address = erc1363_receiver::deploy(&alice.wallet).await?;

    // Mint tokens to alice and approve SafeERC20 contract
    watch!(erc20_alice.mint(alice.address(), value))?;
    watch!(erc20_alice.approve(safe_erc20_addr, U256::MAX))?;

    // Use the relaxed helper method
    let err = send!(safe_erc20_alice.transferFromAndCallRelaxed(
        erc20_address,
        alice.address(),
        receiver_address,
        value,
        DATA,
    ))
    .expect_err("should revert with SafeERC20FailedOperation");

    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: erc20_address,
    }));

    Ok(())
}

#[e2e::test]
async fn reverts_on_approve_and_call_relaxed(
    alice: Account,
) -> eyre::Result<()> {
    let safe_erc20_addr = alice.as_deployer().deploy().await?.contract_address;
    let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);

    let value = uint!(10_U256);

    let erc20_address = erc1363_return_false::deploy(&alice.wallet).await?;

    // Deploy ERC1363Spender mock
    let spender_address = erc1363_spender::deploy(&alice.wallet).await?;

    // Use the relaxed helper method
    let err = send!(safe_erc20_alice.approveAndCallRelaxed(
        erc20_address,
        spender_address,
        value,
        DATA
    ))
    .expect_err("should revert with SafeERC20FailedOperation");

    assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
        token: erc20_address,
    }));

    Ok(())
}
