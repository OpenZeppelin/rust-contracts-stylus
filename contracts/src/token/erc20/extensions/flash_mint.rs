//! Implementation of the ERC-3156 Flash loans extension, as defined in
//! [ERC-3156].
//!
//! Adds the [`IErc3156FlashLender::flash_loan`] method, which provides flash
//! loan support at the token level. By default there is no fee, but this can be
//! changed by overriding [`IErc3156FlashLender::flash_loan`].
//!
//! NOTE: When this extension is used along with the
//! [`crate::token::erc20::extensions::Capped`] extension,
//! [`IErc3156FlashLender::max_flash_loan`] will not correctly reflect the
//! maximum that can be flash minted. We recommend overriding
//! [`IErc3156FlashLender::max_flash_loan`] so that it correctly reflects the
//! supply cap.
//!
//! [ERC-3156]: https://eips.ethereum.org/EIPS/eip-3156

// TODO: once ERC20Votes is implemented, include it in the comment above next to
// ERC20Capped.

use alloy_primitives::{Address, U256};
use stylus_sdk::{
    abi::Bytes,
    call::Call,
    contract, msg,
    prelude::*,
    storage::{StorageAddress, StorageU256},
};

use crate::token::erc20::{self, Erc20, IErc20};

const BORROWER_CALLBACK_VALUE: [u8; 32] = keccak_const::Keccak256::new()
    .update("ERC3156FlashBorrower.onFlashLoan".as_bytes())
    .finalize();

pub use sol::*;
mod sol {
    #![cfg_attr(coverage_nightly, coverage(off))]
    use alloy_sol_macro::sol;

    sol! {
        /// Indicate that the loan token is not supported or valid.
        ///
        /// * `token` - Address of the unsupported token.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC3156UnsupportedToken(address token);

        /// Indicate an error related to the loan amount exceeding the maximum.
        ///
        /// * `max_loan` - Maximum loan amount.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC3156ExceededMaxLoan(uint256 max_loan);

        /// Indicate that the receiver of a flashloan is not a valid [`IERC3156FlashBorrower::on_flash_loan`] implementer.
        ///
        /// * `receiver` - Address to which tokens are being transferred.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC3156InvalidReceiver(address receiver);
    }
}

/// An [`Erc20FlashMint`] extension error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicate that the loan token is not supported or valid.
    UnsupportedToken(ERC3156UnsupportedToken),
    /// Indicate an error related to the loan amount exceeding the maximum.
    ExceededMaxLoan(ERC3156ExceededMaxLoan),
    /// Indicate that the receiver of a flashloan is not a valid
    /// [`IERC3156FlashBorrower::on_flash_loan`] implementer.
    InvalidReceiver(ERC3156InvalidReceiver),
    /// Error type from [`Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error),
}

pub use borrower::IERC3156FlashBorrower;
mod borrower {
    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]
    use stylus_sdk::stylus_proc::sol_interface;

    sol_interface! {
        /// Interface of the ERC-3156 FlashBorrower, as defined in [ERC-3156].
        ///
        /// [ERC-3156]: https://eips.ethereum.org/EIPS/eip-3156
        interface IERC3156FlashBorrower {
            /// Receives a flash loan.
            ///
            /// To indicate successful handling of the flash loan, this function should return
            /// the `keccak256` hash of "ERC3156FlashBorrower.onFlashLoan".
            ///
            /// # Arguments
            ///
            /// * `initiator` - The initiator of the flash loan.
            /// * `token` - The token to be flash loaned.
            /// * `amount` - The amount of tokens lent.
            /// * `fee` - The additional amount of tokens to repay.
            /// * `data` - Arbitrary data structure, intended to contain user-defined parameters.
            #[allow(missing_docs)]
            function onFlashLoan(
                address initiator,
                address token,
                uint256 amount,
                uint256 fee,
                bytes calldata data
            ) external returns (bytes32);
        }
    }
}

/// State of the [`Erc20FlashMint`] Contract.
#[storage]
pub struct Erc20FlashMint {
    /// Fee applied when doing flash loans.
    pub flash_fee_amount: StorageU256,
    /// Receiver address of the flash fee.
    pub flash_fee_receiver_address: StorageAddress,
}

/// Interface of the ERC-3156 Flash Lender, as defined in [ERC-3156].
///
/// [ERC-3156]: https://eips.ethereum.org/EIPS/eip-3156
pub trait IErc3156FlashLender {
    /// The error type associated to this trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the maximum amount of tokens available for loan.
    ///
    /// NOTE: This function does not consider any form of supply cap, so in case
    /// it's used in a token with a cap like
    /// [`crate::token::erc20::extensions::Capped`], make sure to override this
    /// function to integrate the cap instead of [`U256::MAX`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token` - The address of the token that is requested.
    /// * `erc20` - Read access to an [`Erc20`] contract.
    fn max_flash_loan(&self, token: Address, erc20: &Erc20) -> U256;

    /// Returns the fee applied when doing flash loans.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token` - The token to be flash loaned.
    /// * `amount` - The amount of tokens to be loaned.
    ///
    /// # Errors
    ///
    /// If the token is not supported, then the error
    /// [`Error::UnsupportedToken`] is returned.
    fn flash_fee(
        &self,
        token: Address,
        amount: U256,
    ) -> Result<U256, Self::Error>;

    /// Performs a flash loan.
    ///
    /// New tokens are minted and sent to the `receiver`, who is required to
    /// implement the [`IERC3156FlashBorrower`] interface. By the end of the
    /// flash loan, the receiver is expected to own value + fee tokens and have
    /// them approved back to the token contract itself so they can be burned.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `receiver` - The receiver of the flash loan. Should implement the
    ///   [`IERC3156FlashBorrower::on_flash_loan`] interface.
    /// * `token` - The token to be flash loaned. Only [`contract::address()`]
    ///   is supported.
    /// * `value` - The amount of tokens to be loaned.
    /// * `data` - Arbitrary data that is passed to the receiver.
    /// * `erc20` - Write access to an [`Erc20`] contract.
    ///
    /// # Errors
    ///
    /// If the `amount` is greater than the value returned by
    /// [`IErc3156FlashLender::max_flash_loan`], then the error
    /// [`Error::ExceededMaxLoan`] is returned. If `token` is not supported,
    /// then the error [`Error::UnsupportedToken`] is returned.
    /// If the `token` address is not a contract, then the error
    /// [`Error::InvalidReceiver`] is returned. If the contract fails to
    /// execute the call, then the error [`Error::InvalidReceiver`] is returned.
    /// If the receiver does not return [`BORROWER_CALLBACK_VALUE`], then the
    /// error [`Error::InvalidReceiver`] is returned.
    ///
    /// # Events
    ///
    /// Emits an [`erc20::Transfer`] event.
    /// Emits an [`erc20::Approval`] event.
    ///
    /// # Panics
    ///
    /// If the new (temporary) total supply exceeds `U256::MAX`.
    /// If the sum of the loan amount and fee exceeds the maximum value of
    /// `U256::MAX`.
    fn flash_loan(
        &mut self,
        receiver: Address,
        token: Address,
        amount: U256,
        data: Bytes,
        erc20: &mut Erc20,
    ) -> Result<bool, Self::Error>;
}

impl IErc3156FlashLender for Erc20FlashMint {
    type Error = Error;

    fn max_flash_loan(&self, token: Address, erc20: &Erc20) -> U256 {
        if token == contract::address() {
            return U256::MAX - erc20.total_supply();
        }
        U256::MIN
    }

    fn flash_fee(
        &self,
        token: Address,
        _amount: U256,
    ) -> Result<U256, Self::Error> {
        if token != contract::address() {
            return Err(Error::UnsupportedToken(ERC3156UnsupportedToken {
                token,
            }));
        }
        Ok(self.flash_fee_amount.get())
    }

    fn flash_loan(
        &mut self,
        receiver: Address,
        token: Address,
        amount: U256,
        data: Bytes,
        erc20: &mut Erc20,
    ) -> Result<bool, Self::Error> {
        let max_loan = self.max_flash_loan(token, erc20);
        if amount > max_loan {
            return Err(Error::ExceededMaxLoan(ERC3156ExceededMaxLoan {
                max_loan,
            }));
        }

        let fee = self.flash_fee(token, amount)?;
        if !Address::has_code(&receiver) {
            return Err(Error::InvalidReceiver(ERC3156InvalidReceiver {
                receiver,
            }));
        }
        erc20._mint(receiver, amount)?;
        let loan_receiver = IERC3156FlashBorrower::new(receiver);
        let loan_return = loan_receiver
            .on_flash_loan(
                Call::new(),
                msg::sender(),
                token,
                amount,
                fee,
                data.to_vec().into(),
            )
            .map_err(|_| {
                Error::InvalidReceiver(ERC3156InvalidReceiver { receiver })
            })?;
        if loan_return != BORROWER_CALLBACK_VALUE {
            return Err(Error::InvalidReceiver(ERC3156InvalidReceiver {
                receiver,
            }));
        }

        let allowance = amount
            .checked_add(fee)
            .expect("allowance should not exceed `U256::MAX`");
        erc20._spend_allowance(receiver, contract::address(), allowance)?;

        let flash_fee_receiver = self.flash_fee_receiver_address.get();

        if fee.is_zero() || flash_fee_receiver.is_zero() {
            // SAFETY: overflow already checked when calculating allowance
            erc20._burn(receiver, amount + fee)?;
        } else {
            erc20._burn(receiver, amount)?;
            erc20._transfer(receiver, flash_fee_receiver, fee)?;
        }

        Ok(true)
    }
}

// TODO: uncomment once it's possible to mock contract address.
// NOTE: double check that the tests assert the correct and expected things.
// #[cfg(all(test, feature = "std"))]
// mod tests {

// use alloc::vec;

// use alloy_primitives::{address, uint, Address, U256};
// use stylus_sdk::msg;

// use crate::token::erc20::{
//     extensions::flash_mint::{Error, IErc3156FlashLender},
//     Erc20,
// };

// const ALICE: Address =
// address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
// const TOKEN_ADDRESS: Address =
//     address!("dce82b5f92c98f27f116f70491a487effdb6a2a9");
// const INVALID_TOKEN_ADDRESS: Address =
//     address!("dce82b5f92c98f27f116f70491a487effdb6a2aa");

// #[motsu::test]
// fn max_flash_loan_token_match(contract: Erc20) {
//     let max_flash_loan = contract.max_flash_loan(TOKEN_ADDRESS);
//     assert_eq!(max_flash_loan, U256::MAX);
// }

// #[motsu::test]
// fn max_flash_loan_token_mismatch(contract: Erc20) {
//     let max_flash_loan = contract.max_flash_loan(INVALID_TOKEN_ADDRESS);
//     assert_eq!(max_flash_loan, U256::MIN);
// }

// #[motsu::test]
// fn max_flash_loan_when_token_minted(contract: Erc20) {
//     contract._mint(msg::sender(), uint!(10000_U256)).unwrap();
//     let max_flash_loan = contract.max_flash_loan(TOKEN_ADDRESS);
//     assert_eq!(max_flash_loan, U256::MAX - uint!(10000_U256));
// }

// #[motsu::test]
// fn flash_fee(contract: Erc20) {
//     let flash_fee =
//         contract.flash_fee(TOKEN_ADDRESS, uint!(1000_U256)).unwrap();
//     assert_eq!(flash_fee, U256::MIN);
// }

// #[motsu::test]
// fn error_flash_fee_when_invalid_token(contract: Erc20) {
//     let result =
//         contract.flash_fee(INVALID_TOKEN_ADDRESS, uint!(1000_U256));
//     assert!(matches!(result, Err(Error::UnsupportedToken(_))));
// }

// #[motsu::test]
// fn error_flash_loan_when_exceeded_max_loan(contract: Erc20) {
//     let _ = contract._mint(msg::sender(), uint!(10000_U256));
//     let result = contract.flash_loan(
//         msg::sender(),
//         TOKEN_ADDRESS,
//         U256::MAX,
//         vec![0, 1].into(),
//     );
//     assert!(matches!(result, Err(Error::ExceededMaxLoan(_))));
// }

// #[motsu::test]
// fn flash_loan(contract: Erc20) {
//     let _ = contract._mint(msg::sender(), uint!(10000_U256));
// }

// #[motsu::test]
// fn error_flash_loan_when_zero_receiver_address(contract: Erc20) {
//     let invalid_reciver = Address::ZERO;
//     let result = contract.flash_loan(
//         invalid_reciver,
//         TOKEN_ADDRESS,
//         uint!(1000_U256),
//         vec![0, 1].into(),
//     );
//     assert_eq!(result.is_err(), true);
// }

// #[motsu::test]
// fn error_flash_loan_when_invalid_receiver(contract: Erc20) {
//     let result = contract.flash_loan(
//         ALICE,
//         TOKEN_ADDRESS,
//         uint!(1000_U256),
//         vec![0, 1].into(),
//     );
//     assert_eq!(result.is_err(), true);
// }
// }
