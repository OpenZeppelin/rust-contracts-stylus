#![cfg(feature = "e2e")]

use alloy::primitives::uint;
use alloy_primitives::U256;
use e2e::{receipt, send, watch, Account, ReceiptExt, Revert};

use abi::SafeErc20;
use mock::{erc20, erc20::ERC20Mock};

mod abi;
mod mock;

mod transfers {
    use super::*;

    #[e2e::test]
    async fn doesnt_revert_on_transfer(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_mock_addr =
            alice.as_deployer().deploy().await?.address()?;
        let safe_erc20_mock_alice =
            SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
        let bob_addr = bob.address();

        let balance = uint!(10_U256);
        let value = uint!(1_U256);

        let erc20mock_address = erc20::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20Mock::new(erc20mock_address, &alice.wallet);

        let _ = watch!(erc20_alice.mint(safe_erc20_mock_addr, balance));

        let ERC20Mock::balanceOfReturn { _0: initial_safe_erc20_mock_balance } =
            erc20_alice.balanceOf(safe_erc20_mock_addr).call().await?;
        let ERC20Mock::balanceOfReturn { _0: initial_bob_balance } =
            erc20_alice.balanceOf(bob_addr).call().await?;
        assert_eq!(initial_safe_erc20_mock_balance, balance);
        assert_eq!(initial_bob_balance, U256::ZERO);

        let _ = receipt!(safe_erc20_mock_alice.safeTransfer(
            erc20mock_address,
            bob_addr,
            value
        ))?;

        let ERC20Mock::balanceOfReturn { _0: safe_erc20_mock_balance } =
            erc20_alice.balanceOf(safe_erc20_mock_addr).call().await?;
        let ERC20Mock::balanceOfReturn { _0: bob_balance } =
            erc20_alice.balanceOf(bob_addr).call().await?;

        assert_eq!(
            initial_safe_erc20_mock_balance - value,
            safe_erc20_mock_balance
        );
        assert_eq!(initial_bob_balance + value, bob_balance);

        Ok(())
    }

    #[e2e::test]
    async fn doesnt_revert_on_transfer_from(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_mock_addr =
            alice.as_deployer().deploy().await?.address()?;
        let safe_erc20_mock_alice =
            SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
        let alice_addr = alice.address();
        let bob_addr = bob.address();

        let balance = uint!(10_U256);
        let value = uint!(1_U256);

        let erc20_address = erc20::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

        let _ = watch!(erc20_alice.mint(alice_addr, balance));
        let _ = watch!(erc20_alice.approve(safe_erc20_mock_addr, value));

        let ERC20Mock::balanceOfReturn { _0: initial_alice_balance } =
            erc20_alice.balanceOf(alice_addr).call().await?;
        let ERC20Mock::balanceOfReturn { _0: initial_bob_balance } =
            erc20_alice.balanceOf(bob_addr).call().await?;
        assert_eq!(initial_alice_balance, balance);
        assert_eq!(initial_bob_balance, U256::ZERO);

        let _ = receipt!(safe_erc20_mock_alice.safeTransferFrom(
            erc20_address,
            alice_addr,
            bob_addr,
            value
        ))?;

        let ERC20Mock::balanceOfReturn { _0: alice_balance } =
            erc20_alice.balanceOf(alice_addr).call().await?;
        let ERC20Mock::balanceOfReturn { _0: bob_balance } =
            erc20_alice.balanceOf(bob_addr).call().await?;

        assert_eq!(initial_alice_balance - value, alice_balance);
        assert_eq!(initial_bob_balance + value, bob_balance);

        Ok(())
    }
}

mod approvals {
    mod with_zero_allowance {
        use super::super::*;

        #[e2e::test]
        async fn doesnt_revert_when_force_approving_a_non_zero_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_mock_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_mock_alice =
                SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_mock_addr,
                spender_addr,
                U256::ZERO
            ));

            let value = uint!(100_U256);

            let _ = receipt!(safe_erc20_mock_alice.forceApprove(
                erc20_address,
                spender_addr,
                value
            ))?;

            let ERC20Mock::allowanceReturn { _0: spender_allowance } =
                erc20_alice
                    .allowance(safe_erc20_mock_addr, spender_addr)
                    .call()
                    .await?;
            assert_eq!(spender_allowance, value);

            Ok(())
        }

        #[e2e::test]
        async fn doesnt_revert_when_force_approving_a_zero_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_mock_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_mock_alice =
                SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_mock_addr,
                spender_addr,
                U256::ZERO
            ));

            let _ = receipt!(safe_erc20_mock_alice.forceApprove(
                erc20_address,
                spender_addr,
                U256::ZERO
            ))?;

            let ERC20Mock::allowanceReturn { _0: spender_allowance } =
                erc20_alice
                    .allowance(safe_erc20_mock_addr, spender_addr)
                    .call()
                    .await?;
            assert_eq!(spender_allowance, U256::ZERO);

            Ok(())
        }

        #[e2e::test]
        async fn doesnt_revert_when_increasing_the_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_mock_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_mock_alice =
                SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_mock_addr,
                spender_addr,
                U256::ZERO
            ));

            let value = uint!(10_U256);

            let _ = receipt!(safe_erc20_mock_alice.safeIncreaseAllowance(
                erc20_address,
                spender_addr,
                value
            ))?;

            let ERC20Mock::allowanceReturn { _0: spender_allowance } =
                erc20_alice
                    .allowance(safe_erc20_mock_addr, spender_addr)
                    .call()
                    .await?;
            assert_eq!(spender_allowance, value);

            Ok(())
        }

        #[e2e::test]
        async fn reverts_when_decreasing_the_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_mock_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_mock_alice =
                SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_mock_addr,
                spender_addr,
                U256::ZERO
            ));

            let value = uint!(10_U256);

            let err = send!(safe_erc20_mock_alice.safeDecreaseAllowance(
                erc20_address,
                spender_addr,
                value
            ))
            .expect_err("should not be able to succeed on 'decreaseAllowance'");
            assert!(err.reverted_with(
                SafeErc20::SafeErc20FailedDecreaseAllowance {
                    spender: spender_addr,
                    currentAllowance: U256::ZERO,
                    requestedDecrease: value
                }
            ));

            Ok(())
        }
    }

    mod with_non_zero_allowance {
        use super::super::*;

        #[e2e::test]
        async fn doesnt_revert_when_force_approving_a_non_zero_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_mock_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_mock_alice =
                SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_mock_addr,
                spender_addr,
                allowance
            ));

            let value = uint!(20_U256);

            let _ = receipt!(safe_erc20_mock_alice.forceApprove(
                erc20_address,
                spender_addr,
                value
            ))?;

            let ERC20Mock::allowanceReturn { _0: spender_allowance } =
                erc20_alice
                    .allowance(safe_erc20_mock_addr, spender_addr)
                    .call()
                    .await?;
            assert_eq!(spender_allowance, value);

            Ok(())
        }

        #[e2e::test]
        async fn doesnt_revert_when_force_approving_a_zero_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_mock_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_mock_alice =
                SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_mock_addr,
                spender_addr,
                allowance
            ));

            let _ = receipt!(safe_erc20_mock_alice.forceApprove(
                erc20_address,
                spender_addr,
                U256::ZERO
            ))?;

            let ERC20Mock::allowanceReturn { _0: spender_allowance } =
                erc20_alice
                    .allowance(safe_erc20_mock_addr, spender_addr)
                    .call()
                    .await?;
            assert_eq!(spender_allowance, U256::ZERO);

            Ok(())
        }

        #[e2e::test]
        async fn doesnt_revert_when_increasing_the_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_mock_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_mock_alice =
                SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_mock_addr,
                spender_addr,
                allowance
            ));

            let value = uint!(10_U256);

            let _ = receipt!(safe_erc20_mock_alice.safeIncreaseAllowance(
                erc20_address,
                spender_addr,
                value
            ))?;

            let ERC20Mock::allowanceReturn { _0: spender_allowance } =
                erc20_alice
                    .allowance(safe_erc20_mock_addr, spender_addr)
                    .call()
                    .await?;
            assert_eq!(spender_allowance, allowance + value);

            Ok(())
        }

        #[e2e::test]
        async fn doesnt_revert_when_decreasing_the_allowance_to_a_positive_value(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_mock_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_mock_alice =
                SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_mock_addr,
                spender_addr,
                allowance
            ));

            let value = uint!(50_U256);

            let _ = receipt!(safe_erc20_mock_alice.safeDecreaseAllowance(
                erc20_address,
                spender_addr,
                value
            ))?;

            let ERC20Mock::allowanceReturn { _0: spender_allowance } =
                erc20_alice
                    .allowance(safe_erc20_mock_addr, spender_addr)
                    .call()
                    .await?;
            assert_eq!(spender_allowance, allowance - value);

            Ok(())
        }

        #[e2e::test]
        async fn reverts_when_decreasing_the_allowance_to_a_negative_value(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_mock_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_mock_alice =
                SafeErc20::new(safe_erc20_mock_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_mock_addr,
                spender_addr,
                allowance
            ));

            let value = uint!(200_U256);

            let err = send!(safe_erc20_mock_alice.safeDecreaseAllowance(
                erc20_address,
                spender_addr,
                value
            ))
            .expect_err("should not be able to succeed on 'decreaseAllowance'");
            assert!(err.reverted_with(
                SafeErc20::SafeErc20FailedDecreaseAllowance {
                    spender: spender_addr,
                    currentAllowance: allowance,
                    requestedDecrease: value
                }
            ));

            Ok(())
        }
    }
}
