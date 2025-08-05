#![cfg(feature = "e2e")]

use abi::SafeErc20;
use alloy::primitives::{uint, Bytes};
use e2e::{send, watch, Account, Revert};
use mock::{
    erc1363_force_approve, erc1363_force_approve::ERC1363ForceApproveMock,
    erc1363_spender,
};

mod abi;
mod mock;

const DATA: Bytes = Bytes::from_static(b"0x12345678");

mod without_initial_approval {
    use super::*;

    #[e2e::test]
    async fn approve_and_call_relaxed_works_when_recipient_is_eoa(
        alice: Account,
        spender: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let spender_addr = spender.address();

        let value = uint!(10_U256);

        let erc20_address =
            erc1363_force_approve::deploy(&alice.wallet).await?;
        let erc20_alice =
            ERC1363ForceApproveMock::new(erc20_address, &alice.wallet);

        watch!(safe_erc20_alice.approveAndCallRelaxed(
            erc20_address,
            spender_addr,
            value,
            DATA
        ))?;

        let allowance = erc20_alice
            .allowance(safe_erc20_addr, spender_addr)
            .call()
            .await?
            ._0;
        assert_eq!(allowance, value);

        Ok(())
    }

    #[e2e::test]
    async fn approve_and_call_relaxed_works_when_recipient_is_contract(
        alice: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);

        let value = uint!(10_U256);

        let erc20_address =
            erc1363_force_approve::deploy(&alice.wallet).await?;
        let erc20_alice =
            ERC1363ForceApproveMock::new(erc20_address, &alice.wallet);

        // Deploy ERC1363Spender mock
        let spender_address = erc1363_spender::deploy(&alice.wallet).await?;

        watch!(safe_erc20_alice.approveAndCallRelaxed(
            erc20_address,
            spender_address,
            value,
            DATA
        ))?;

        let allowance = erc20_alice
            .allowance(safe_erc20_addr, spender_address)
            .call()
            .await?
            ._0;
        assert_eq!(allowance, value);

        Ok(())
    }
}

mod with_initial_approval {
    use super::*;

    #[e2e::test]
    async fn approve_and_call_relaxed_works_when_recipient_is_eoa(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let bob_addr = bob.address();

        let initial_allowance = uint!(100_U256);
        let value = uint!(10_U256);

        let erc20_address =
            erc1363_force_approve::deploy(&alice.wallet).await?;
        let erc20_alice =
            ERC1363ForceApproveMock::new(erc20_address, &alice.wallet);

        // Set initial approval
        watch!(erc20_alice.forceApprove(
            safe_erc20_addr,
            bob_addr,
            initial_allowance
        ))?;

        watch!(safe_erc20_alice.approveAndCallRelaxed(
            erc20_address,
            bob_addr,
            value,
            DATA
        ))?;

        let allowance =
            erc20_alice.allowance(safe_erc20_addr, bob_addr).call().await?._0;
        assert_eq!(allowance, value);

        Ok(())
    }

    #[e2e::test]
    async fn approve_and_call_relaxed_reverts_when_recipient_is_contract(
        alice: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);

        let initial_allowance = uint!(100_U256);
        let value = uint!(10_U256);

        let erc20_address =
            erc1363_force_approve::deploy(&alice.wallet).await?;
        let erc20_alice =
            ERC1363ForceApproveMock::new(erc20_address, &alice.wallet);

        // Deploy ERC1363Spender mock
        let spender_address = erc1363_spender::deploy(&alice.wallet).await?;

        // Set initial approval
        watch!(erc20_alice.forceApprove(
            safe_erc20_addr,
            spender_address,
            initial_allowance
        ))?;

        let err = send!(safe_erc20_alice.approveAndCallRelaxed(
            erc20_address,
            spender_address,
            value,
            DATA
        ))
        .expect_err("should revert with 'USDT approval failure'");

        // TODO: once SafeErc20 is returning Vec<u8> for all errors, update this
        // to `"USDT approval failure"`
        assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
            token: erc20_address,
        }));

        Ok(())
    }
}
