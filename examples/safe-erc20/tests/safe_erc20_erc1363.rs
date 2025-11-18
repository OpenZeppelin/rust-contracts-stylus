#![cfg(feature = "e2e")]

use abi::{Erc1363Receiver, Erc1363Spender, Erc20, SafeErc20};
use alloy::primitives::uint;
use alloy_primitives::{Bytes, U256};
use e2e::{receipt, send, watch, Account, EventExt, Revert, RustPanic};
use mock::{erc1363, erc1363::ERC1363Mock, erc1363_receiver, erc1363_spender};

mod abi;
mod mock;

const DATA: Bytes = Bytes::from_static(b"0x12345678");

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

        let erc20_address = erc1363::deploy(&alice.wallet).await?;
        let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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
    async fn reverts_on_transfer_with_internal_error(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let bob_addr = bob.address();

        let value = U256::ONE;

        let erc20_address = erc1363::deploy(&alice.wallet).await?;
        let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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

        let erc20_address = erc1363::deploy(&alice.wallet).await?;
        let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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
            value
        }));

        let alice_balance = erc20_alice.balanceOf(alice_addr).call().await?._0;
        let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

        assert_eq!(initial_alice_balance - value, alice_balance);
        assert_eq!(initial_bob_balance + value, bob_balance);

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

        let erc20_address = erc1363::deploy(&alice.wallet).await?;
        let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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

        let erc20_address = erc1363::deploy(&alice.wallet).await?;
        let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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

        let erc20_address = erc1363::deploy(&alice.wallet).await?;
        let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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

            let erc20_address = erc1363::deploy(&alice.wallet).await?;
            let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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

            let erc20_address = erc1363::deploy(&alice.wallet).await?;
            let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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

            let erc20_address = erc1363::deploy(&alice.wallet).await?;
            let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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

            let erc20_address = erc1363::deploy(&alice.wallet).await?;
            let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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
            .expect_err("should not exceed U256::MAX");

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

            let erc20_address = erc1363::deploy(&alice.wallet).await?;
            let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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

            let erc20_address = erc1363::deploy(&alice.wallet).await?;
            let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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

            let erc20_address = erc1363::deploy(&alice.wallet).await?;
            let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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

            let erc20_address = erc1363::deploy(&alice.wallet).await?;
            let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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

            let erc20_address = erc1363::deploy(&alice.wallet).await?;
            let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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

            let erc20_address = erc1363::deploy(&alice.wallet).await?;
            let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

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

mod transfer_and_call {
    use super::*;

    #[e2e::test]
    async fn can_transfer_and_call_to_eoa_using_helper(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let bob_addr = bob.address();

        let value = uint!(10_U256);

        let erc20_address = erc1363::deploy(&alice.wallet).await?;
        let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

        // Mint tokens to the SafeERC20 contract
        watch!(erc20_alice.mint(safe_erc20_addr, value))?;

        // Use the relaxed helper method
        let receipt = receipt!(safe_erc20_alice.transferAndCallRelaxed(
            erc20_address,
            bob_addr,
            value,
            DATA
        ))?;

        assert!(receipt.emits(Erc20::Transfer {
            from: safe_erc20_addr,
            to: bob_addr,
            value
        }));

        // Verify balances
        let safe_erc20_balance =
            erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
        let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

        assert_eq!(safe_erc20_balance, U256::ZERO);
        assert_eq!(bob_balance, value);

        Ok(())
    }

    #[e2e::test]
    async fn can_transfer_and_call_to_erc1363_receiver_using_helper(
        alice: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);

        let value = uint!(10_U256);

        let erc20_address = erc1363::deploy(&alice.wallet).await?;
        let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

        // Deploy ERC1363Receiver mock
        let receiver_address = erc1363_receiver::deploy(&alice.wallet).await?;

        // Mint tokens to the SafeERC20 contract
        watch!(erc20_alice.mint(safe_erc20_addr, value))?;

        // Use the relaxed helper method to call ERC1363Receiver
        let receipt = receipt!(safe_erc20_alice.transferAndCallRelaxed(
            erc20_address,
            receiver_address,
            value,
            DATA
        ))?;

        assert!(receipt.emits(Erc20::Transfer {
            from: safe_erc20_addr,
            to: receiver_address,
            value
        }));

        // The ERC1363Receiver should emit a Received event
        assert!(receipt.emits(Erc1363Receiver::Received {
            operator: safe_erc20_addr,
            from: safe_erc20_addr,
            value,
            data: DATA
        }));

        // Verify balances
        let safe_erc20_balance =
            erc20_alice.balanceOf(safe_erc20_addr).call().await?._0;
        let receiver_balance =
            erc20_alice.balanceOf(receiver_address).call().await?._0;

        assert_eq!(safe_erc20_balance, U256::ZERO);
        assert_eq!(receiver_balance, value);

        Ok(())
    }
}

mod transfer_from_and_call {
    use super::*;

    #[e2e::test]
    async fn can_transfer_from_and_call_to_eoa_using_helper(
        alice: Account,
        bob: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let alice_addr = alice.address();
        let bob_addr = bob.address();

        let value = uint!(10_U256);

        let erc20_address = erc1363::deploy(&alice.wallet).await?;
        let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

        // Mint tokens to alice and approve SafeERC20 contract
        watch!(erc20_alice.mint(alice_addr, value))?;
        watch!(erc20_alice.approve(safe_erc20_addr, U256::MAX))?;

        // Use the relaxed helper method
        let receipt = receipt!(safe_erc20_alice.transferFromAndCallRelaxed(
            erc20_address,
            alice_addr,
            bob_addr,
            value,
            DATA
        ))?;

        assert!(receipt.emits(Erc20::Transfer {
            from: alice_addr,
            to: bob_addr,
            value
        }));

        // Verify balances
        let alice_balance = erc20_alice.balanceOf(alice_addr).call().await?._0;
        let bob_balance = erc20_alice.balanceOf(bob_addr).call().await?._0;

        assert_eq!(alice_balance, U256::ZERO);
        assert_eq!(bob_balance, value);

        Ok(())
    }

    #[e2e::test]
    async fn can_transfer_from_and_call_to_erc1363_receiver_using_helper(
        alice: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);
        let alice_addr = alice.address();

        let value = uint!(10_U256);

        let erc20_address = erc1363::deploy(&alice.wallet).await?;
        let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

        // Deploy ERC1363Receiver mock
        let receiver_address = erc1363_receiver::deploy(&alice.wallet).await?;

        // Mint tokens to alice and approve SafeERC20 contract
        watch!(erc20_alice.mint(alice_addr, value))?;
        watch!(erc20_alice.approve(safe_erc20_addr, U256::MAX))?;

        // Use the relaxed helper method to call ERC1363Receiver
        let receipt = receipt!(safe_erc20_alice.transferFromAndCallRelaxed(
            erc20_address,
            alice_addr,
            receiver_address,
            value,
            DATA
        ))?;

        assert!(receipt.emits(Erc20::Transfer {
            from: alice_addr,
            to: receiver_address,
            value
        }));

        // The ERC1363Receiver should emit a Received event
        assert!(receipt.emits(Erc1363Receiver::Received {
            operator: safe_erc20_addr,
            from: alice_addr,
            value,
            data: DATA
        }));

        // Verify balances
        let alice_balance = erc20_alice.balanceOf(alice_addr).call().await?._0;
        let receiver_balance =
            erc20_alice.balanceOf(receiver_address).call().await?._0;

        assert_eq!(alice_balance, U256::ZERO);
        assert_eq!(receiver_balance, value);

        Ok(())
    }
}

mod approve_and_call {
    use super::*;

    #[e2e::test]
    async fn can_approve_and_call_to_eoa_using_helper(
        alice: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);

        let value = uint!(10_U256);

        let erc20_address = erc1363::deploy(&alice.wallet).await?;
        let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

        let erc1363_spender = erc1363_spender::deploy(&alice.wallet).await?;

        // Use the relaxed helper method
        let receipt = receipt!(safe_erc20_alice.approveAndCallRelaxed(
            erc20_address,
            erc1363_spender,
            value,
            DATA
        ))?;

        assert!(receipt.emits(Erc20::Approval {
            owner: safe_erc20_addr,
            spender: erc1363_spender,
            value
        }));

        // Verify allowance
        let allowance = erc20_alice
            .allowance(safe_erc20_addr, erc1363_spender)
            .call()
            .await?
            ._0;
        assert_eq!(allowance, value);

        Ok(())
    }

    #[e2e::test]
    async fn can_approve_and_call_to_erc1363_spender_using_helper(
        alice: Account,
    ) -> eyre::Result<()> {
        let safe_erc20_addr =
            alice.as_deployer().deploy().await?.contract_address;
        let safe_erc20_alice = SafeErc20::new(safe_erc20_addr, &alice.wallet);

        let value = uint!(10_U256);

        let erc20_address = erc1363::deploy(&alice.wallet).await?;
        let erc20_alice = ERC1363Mock::new(erc20_address, &alice.wallet);

        // Deploy ERC1363Spender mock
        let spender_address = erc1363_spender::deploy(&alice.wallet).await?;

        // Use the relaxed helper method to call ERC1363Spender
        let receipt = receipt!(safe_erc20_alice.approveAndCallRelaxed(
            erc20_address,
            spender_address,
            value,
            DATA
        ))?;

        assert!(receipt.emits(Erc20::Approval {
            owner: safe_erc20_addr,
            spender: spender_address,
            value
        }));

        // The ERC1363Spender should emit an Approved event
        assert!(receipt.emits(Erc1363Spender::Approved {
            owner: safe_erc20_addr,
            value,
            data: DATA
        }));

        // Verify allowance
        let allowance = erc20_alice
            .allowance(safe_erc20_addr, spender_address)
            .call()
            .await?
            ._0;
        assert_eq!(allowance, value);

        Ok(())
    }
}
