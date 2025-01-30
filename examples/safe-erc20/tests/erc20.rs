#![cfg(feature = "e2e")]

use abi::{Erc20, SafeErc20};
use alloy::primitives::uint;
use alloy_primitives::U256;
use e2e::{
    receipt, send, watch, Account, EventExt, Panic, PanicCode, ReceiptExt,
    Revert,
};
use mock::{erc20, erc20::ERC20Mock};

mod abi;
mod mock;

mod transfers {
    use super::*;

    #[e2e::test]
    async fn safe_transfer_success(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let bob_addr = bob.address();

        let balance = uint!(10_U256);
        let value = uint!(1_U256);

        let erc20_address = erc20::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

        let _ = watch!(erc20_alice.mint(safe_erc20_addr, balance));

        let initial_safe_erc20_balance =
            erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
        let initial_bob_balance =
            erc20_alice.balanceOf(bob_addr).call().await?._0;
        assert_eq!(initial_safe_erc20_balance, balance);
        assert_eq!(initial_bob_balance, U256::ZERO);

        let receipt = receipt!(safe_erc20_alice.safeTransfer(
            erc20_address,
            bob_addr,
            value
        ))?;

        assert!(receipt.emits(Erc20::Transfer {
            from: safe_erc20_addr,
            to: bob_addr,
            value
        }));

        let safe_erc20_balance =
            erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
        let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

        assert_eq!(initial_safe_erc20_balance - value, safe_erc20_balance);
        assert_eq!(initial_bob_balance + value, bob_balance);

        Ok(())
    }

    #[e2e::test]
    async fn safe_transfer_reverts_when_balance_insufficient(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let bob_addr = bob.address();

        let value = uint!(1_U256);

        let erc20_address = erc20::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

        let initial_safe_erc20_balance =
            erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
        let initial_bob_balance =
            erc20_alice.balanceOf(bob_addr).call().await?._0;

        let err = send!(safe_erc20_alice.safeTransfer(
            erc20_address,
            bob_addr,
            value
        ))
        .expect_err("should not transfer when insufficient balance");

        assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
            token: erc20_address
        }));

        let safe_erc20_balance =
            erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
        let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

        assert_eq!(initial_safe_erc20_balance, safe_erc20_balance);
        assert_eq!(initial_bob_balance, bob_balance);

        Ok(())
    }

    #[e2e::test]
    async fn safe_transfer_from_success(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let alice_addr = alice.address();
        let bob_addr = bob.address();

        let balance = uint!(10_U256);
        let value = uint!(1_U256);

        let erc20_address = erc20::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

        let _ = watch!(erc20_alice.mint(alice_addr, balance));
        let _ = watch!(erc20_alice.approve(safe_erc20_addr, value));

        let initial_alice_balance =
            erc20_alice.balanceOf(alice_addr).call().await?._0;
        let initial_bob_balance =
            erc20_alice.balanceOf(bob_addr).call().await?._0;
        assert_eq!(initial_alice_balance, balance);
        assert_eq!(initial_bob_balance, U256::ZERO);

        let receipt = receipt!(safe_erc20_alice.safeTransferFrom(
            erc20_address,
            alice_addr,
            bob_addr,
            value
        ))?;

        assert!(receipt.emits(Erc20::Transfer {
            from: alice_addr,
            to: bob_addr,
            value
        }));

        let alice_balance = erc20_alice.balanceOf(alice_addr).call().await?._0;
        let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

        assert_eq!(initial_alice_balance - value, alice_balance);
        assert_eq!(initial_bob_balance + value, bob_balance);

        Ok(())
    }

    #[e2e::test]
    async fn safe_transfer_from_reverts_when_balance_insufficient(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr = alice.as_deployer().deploy().await?.address()?;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let alice_addr = alice.address();
        let bob_addr = bob.address();

        let value = uint!(1_U256);

        let erc20_address = erc20::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

        let _ = watch!(erc20_alice.approve(safe_erc20_addr, value));

        let initial_alice_balance =
            erc20_alice.balanceOf(alice_addr).call().await?._0;
        let initial_bob_balance =
            erc20_alice.balanceOf(bob_addr).call().await?._0;

        let err = send!(safe_erc20_alice.safeTransferFrom(
            erc20_address,
            alice_addr,
            bob_addr,
            value
        ))
        .expect_err("should not transfer when insufficient balance");

        assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
            token: erc20_address
        }));

        let alice_balance = erc20_alice.balanceOf(alice_addr).call().await?._0;
        let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

        assert_eq!(initial_alice_balance, alice_balance);
        assert_eq!(initial_bob_balance, bob_balance);

        Ok(())
    }
}

mod approvals {
    mod with_zero_allowance {
        use super::super::*;

        #[e2e::test]
        async fn force_approve_success_with_non_zero_value(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_addr,
                spender_addr,
                U256::ZERO
            ));

            let value = uint!(100_U256);

            let receipt = receipt!(safe_erc20_alice.forceApprove(
                erc20_address,
                spender_addr,
                value
            ))?;

            assert!(receipt.emits(Erc20::Approval {
                owner: safe_erc20_addr,
                spender: spender_addr,
                value,
            }));

            let spender_allowance = erc20_alice
                .allowance(safe_erc20_addr, spender_addr)
                .call()
                .await?
                ._0;
            assert_eq!(spender_allowance, value);

            Ok(())
        }

        #[e2e::test]
        async fn force_approve_success_with_zero_value(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_addr,
                spender_addr,
                U256::ZERO
            ));

            let receipt = receipt!(safe_erc20_alice.forceApprove(
                erc20_address,
                spender_addr,
                U256::ZERO
            ))?;

            assert!(receipt.emits(Erc20::Approval {
                owner: safe_erc20_addr,
                spender: spender_addr,
                value: U256::ZERO,
            }));

            let spender_allowance = erc20_alice
                .allowance(safe_erc20_addr, spender_addr)
                .call()
                .await?
                ._0;
            assert_eq!(spender_allowance, U256::ZERO);

            Ok(())
        }

        #[e2e::test]
        async fn safe_increase_allowance_success(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_addr,
                spender_addr,
                U256::ZERO
            ));

            let value = uint!(10_U256);

            let receipt = receipt!(safe_erc20_alice.safeIncreaseAllowance(
                erc20_address,
                spender_addr,
                value
            ))?;

            assert!(receipt.emits(Erc20::Approval {
                owner: safe_erc20_addr,
                spender: spender_addr,
                value,
            }));

            let spender_allowance = erc20_alice
                .allowance(safe_erc20_addr, spender_addr)
                .call()
                .await?
                ._0;
            assert_eq!(spender_allowance, value);

            Ok(())
        }

        #[e2e::test]
        async fn safe_increase_allowance_reverts_when_allowance_overflows(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_addr,
                spender_addr,
                U256::MAX
            ));

            let value = uint!(1_U256);

            let err = send!(safe_erc20_alice.safeIncreaseAllowance(
                erc20_address,
                spender_addr,
                value
            ))
            .expect_err("should not exceed U256::MAX");

            assert!(err.panicked_with(PanicCode::ArithmeticOverflow));

            Ok(())
        }

        #[e2e::test]
        async fn safe_decrease_allowance_reverts_when_insufficient_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_addr,
                spender_addr,
                U256::ZERO
            ));

            let value = uint!(10_U256);

            let err = send!(safe_erc20_alice.safeDecreaseAllowance(
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
        async fn force_approve_success_with_existing_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_addr,
                spender_addr,
                allowance
            ));

            let value = uint!(20_U256);

            let receipt = receipt!(safe_erc20_alice.forceApprove(
                erc20_address,
                spender_addr,
                value
            ))?;

            assert!(receipt.emits(Erc20::Approval {
                owner: safe_erc20_addr,
                spender: spender_addr,
                value,
            }));

            let spender_allowance = erc20_alice
                .allowance(safe_erc20_addr, spender_addr)
                .call()
                .await?
                ._0;
            assert_eq!(spender_allowance, value);

            Ok(())
        }

        #[e2e::test]
        async fn force_approve_success_with_existing_allowance_to_zero(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_addr,
                spender_addr,
                allowance
            ));

            let receipt = receipt!(safe_erc20_alice.forceApprove(
                erc20_address,
                spender_addr,
                U256::ZERO
            ))?;

            assert!(receipt.emits(Erc20::Approval {
                owner: safe_erc20_addr,
                spender: spender_addr,
                value: U256::ZERO,
            }));

            let spender_allowance = erc20_alice
                .allowance(safe_erc20_addr, spender_addr)
                .call()
                .await?
                ._0;
            assert_eq!(spender_allowance, U256::ZERO);

            Ok(())
        }

        #[e2e::test]
        async fn safe_increase_allowance_success_with_existing_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_addr,
                spender_addr,
                allowance
            ));

            let value = uint!(10_U256);

            let receipt = receipt!(safe_erc20_alice.safeIncreaseAllowance(
                erc20_address,
                spender_addr,
                value
            ))?;

            assert!(receipt.emits(Erc20::Approval {
                owner: safe_erc20_addr,
                spender: spender_addr,
                value: allowance + value,
            }));

            let spender_allowance = erc20_alice
                .allowance(safe_erc20_addr, spender_addr)
                .call()
                .await?
                ._0;
            assert_eq!(spender_allowance, allowance + value);

            Ok(())
        }

        #[e2e::test]
        async fn safe_decrease_allowance_success_to_positive(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_addr,
                spender_addr,
                allowance
            ));

            let value = uint!(50_U256);

            let receipt = receipt!(safe_erc20_alice.safeDecreaseAllowance(
                erc20_address,
                spender_addr,
                value
            ))?;

            assert!(receipt.emits(Erc20::Approval {
                owner: safe_erc20_addr,
                spender: spender_addr,
                value: allowance - value,
            }));

            let spender_allowance = erc20_alice
                .allowance(safe_erc20_addr, spender_addr)
                .call()
                .await?
                ._0;
            assert_eq!(spender_allowance, allowance - value);

            Ok(())
        }

        #[e2e::test]
        async fn safe_decrease_allowance_reverts_when_negative(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.address()?;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20::deploy(&alice.wallet).await?;
            let erc20_alice = ERC20Mock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            let _ = watch!(erc20_alice.regular_approve(
                safe_erc20_addr,
                spender_addr,
                allowance
            ));

            let value = uint!(200_U256);

            let err = send!(safe_erc20_alice.safeDecreaseAllowance(
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
