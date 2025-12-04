#![cfg(feature = "e2e")]

use abi::{Erc20, SafeErc20};
use alloy::primitives::uint;
use alloy_primitives::U256;
use e2e::{receipt, send, watch, Account, EventExt, Revert, RustPanic};
use mock::{erc20_no_return, erc20_no_return::ERC20NoReturnMock};

mod abi;
mod mock;

mod transfers {
    use super::*;

    #[e2e::test]
    async fn does_not_revert_on_transfer(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let bob_addr = bob.address();

        let balance = uint!(10_U256);
        let value = U256::ONE;

        let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20NoReturnMock::new(erc20_address, &alice.wallet);

        watch!(erc20_alice.mint(safe_erc20_addr, balance))?;

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
            value,
        }));

        let safe_erc20_balance =
            erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
        let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

        assert_eq!(initial_safe_erc20_balance - value, safe_erc20_balance);
        assert_eq!(initial_bob_balance + value, bob_balance);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_on_transfer_with_internal_error(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let bob_addr = bob.address();

        let value = U256::ONE;

        let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20NoReturnMock::new(erc20_address, &alice.wallet);

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
    async fn returns_true_on_try_safe_transfer(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let bob_addr = bob.address();

        let balance = uint!(10_U256);
        let value = U256::ONE;

        let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20NoReturnMock::new(erc20_address, &alice.wallet);

        watch!(erc20_alice.mint(safe_erc20_addr, balance))?;

        let initial_safe_erc20_balance =
            erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
        let initial_bob_balance =
            erc20_alice.balanceOf(bob_addr).call().await?._0;
        assert_eq!(initial_safe_erc20_balance, balance);
        assert_eq!(initial_bob_balance, U256::ZERO);

        let receipt = receipt!(safe_erc20_alice.trySafeTransfer(
            erc20_address,
            bob_addr,
            value
        ))?;

        assert!(receipt.emits(SafeErc20::True {}));

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
    async fn does_not_revert_on_transfer_from(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let alice_addr = alice.address();
        let bob_addr = bob.address();

        let balance = uint!(10_U256);
        let value = U256::ONE;

        let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20NoReturnMock::new(erc20_address, &alice.wallet);

        watch!(erc20_alice.mint(alice_addr, balance))?;
        watch!(erc20_alice.approve(safe_erc20_addr, value))?;

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
            value,
        }));

        let alice_balance = erc20_alice.balanceOf(alice_addr).call().await?._0;
        let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

        assert_eq!(initial_alice_balance - value, alice_balance);
        assert_eq!(initial_bob_balance + value, bob_balance);

        Ok(())
    }

    #[e2e::test]
    async fn reverts_on_transfer_from_internal_error(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let alice_addr = alice.address();
        let bob_addr = bob.address();

        let value = U256::ONE;

        let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20NoReturnMock::new(erc20_address, &alice.wallet);

        watch!(erc20_alice.approve(safe_erc20_addr, value))?;

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

    #[e2e::test]
    async fn returns_true_on_try_safe_transfer_from(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let alice_addr = alice.address();
        let bob_addr = bob.address();

        let balance = uint!(10_U256);
        let value = U256::ONE;

        let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
        let erc20_alice = ERC20NoReturnMock::new(erc20_address, &alice.wallet);

        watch!(erc20_alice.mint(alice_addr, balance))?;
        watch!(erc20_alice.approve(safe_erc20_addr, value))?;

        let initial_alice_balance =
            erc20_alice.balanceOf(alice_addr).call().await?._0;
        let initial_bob_balance =
            erc20_alice.balanceOf(bob_addr).call().await?._0;
        assert_eq!(initial_alice_balance, balance);
        assert_eq!(initial_bob_balance, U256::ZERO);

        let receipt = receipt!(safe_erc20_alice.trySafeTransferFrom(
            erc20_address,
            alice_addr,
            bob_addr,
            value
        ))?;

        assert!(receipt.emits(SafeErc20::True {}));

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
}

mod approvals {
    mod with_zero_allowance {
        use super::super::*;

        #[e2e::test]
        async fn does_not_revert_when_force_approving_a_non_zero_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.contract_address;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
            let erc20_alice =
                ERC20NoReturnMock::new(erc20_address, &alice.wallet);

            watch!(erc20_alice.forceApprove(
                safe_erc20_addr,
                spender_addr,
                U256::ZERO
            ))?;

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
        async fn does_not_revert_when_force_approving_a_zero_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.contract_address;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
            let erc20_alice =
                ERC20NoReturnMock::new(erc20_address, &alice.wallet);

            watch!(erc20_alice.forceApprove(
                safe_erc20_addr,
                spender_addr,
                U256::ZERO
            ))?;

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
        async fn does_not_revert_when_increasing_the_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.contract_address;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
            let erc20_alice =
                ERC20NoReturnMock::new(erc20_address, &alice.wallet);

            watch!(erc20_alice.forceApprove(
                safe_erc20_addr,
                spender_addr,
                U256::ZERO
            ))?;

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
        async fn panics_when_increasing_the_allowance_overflow(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.contract_address;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
            let erc20_alice =
                ERC20NoReturnMock::new(erc20_address, &alice.wallet);

            watch!(erc20_alice.forceApprove(
                safe_erc20_addr,
                spender_addr,
                U256::MAX
            ))?;

            let value = U256::ONE;

            let err = send!(safe_erc20_alice.safeIncreaseAllowance(
                erc20_address,
                spender_addr,
                value
            ))
            .expect_err("should exceed U256::MAX");

            assert!(err.panicked());

            Ok(())
        }

        #[e2e::test]
        async fn reverts_when_decreasing_the_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.contract_address;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
            let erc20_alice =
                ERC20NoReturnMock::new(erc20_address, &alice.wallet);

            watch!(erc20_alice.forceApprove(
                safe_erc20_addr,
                spender_addr,
                U256::ZERO
            ))?;

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
        async fn does_not_revert_when_force_approving_a_non_zero_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.contract_address;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
            let erc20_alice =
                ERC20NoReturnMock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            watch!(erc20_alice.forceApprove(
                safe_erc20_addr,
                spender_addr,
                allowance
            ))?;

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
        async fn does_not_revert_when_force_approving_a_zero_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.contract_address;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
            let erc20_alice =
                ERC20NoReturnMock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            watch!(erc20_alice.forceApprove(
                safe_erc20_addr,
                spender_addr,
                allowance
            ))?;

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
        async fn does_not_revert_when_increasing_the_allowance(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.contract_address;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
            let erc20_alice =
                ERC20NoReturnMock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            watch!(erc20_alice.forceApprove(
                safe_erc20_addr,
                spender_addr,
                allowance
            ))?;

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
        async fn does_not_revert_when_decreasing_the_allowance_to_a_positive_value(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.contract_address;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
            let erc20_alice =
                ERC20NoReturnMock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            watch!(erc20_alice.forceApprove(
                safe_erc20_addr,
                spender_addr,
                allowance
            ))?;

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
        async fn reverts_when_decreasing_the_allowance_to_a_negative_value(
            alice: Account,
        ) -> eyre::Result<()> {
            let safe_erc20_addr =
                alice.as_deployer().deploy().await?.contract_address;
            let safe_erc20_alice =
                SafeErc20::new(safe_erc20_addr, &alice.wallet);
            let spender_addr = alice.address();

            let erc20_address = erc20_no_return::deploy(&alice.wallet).await?;
            let erc20_alice =
                ERC20NoReturnMock::new(erc20_address, &alice.wallet);

            let allowance = uint!(100_U256);

            watch!(erc20_alice.forceApprove(
                safe_erc20_addr,
                spender_addr,
                allowance
            ))?;

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
