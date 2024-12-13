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

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    abi::Bytes,
    call::Call,
    contract, msg,
    prelude::*,
    storage::{StorageAddress, StorageU256},
};

use crate::{
    token::erc20::{self, Erc20, IErc20},
    utils::introspection::erc165::{Erc165, IErc165},
};

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
    /// ERC-20 contract storage.
    pub erc20: Erc20,
    /// Fee applied when doing flash loans.
    pub flash_fee_amount: StorageU256,
    /// Receiver address of the flash fee.
    pub flash_fee_receiver_address: StorageAddress,
}

/// Interface of the ERC-3156 Flash Lender, as defined in [ERC-3156].
///
/// [ERC-3156]: https://eips.ethereum.org/EIPS/eip-3156
#[interface_id]
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
    /// * `token` - The address of the token that is requested.
    fn max_flash_loan(&self, token: Address) -> U256;

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
    fn flash_loan(
        &mut self,
        receiver: Address,
        token: Address,
        amount: U256,
        data: Bytes,
    ) -> Result<bool, Self::Error>;
}

impl IErc3156FlashLender for Erc20FlashMint {
    type Error = Error;

    fn max_flash_loan(&self, token: Address) -> U256 {
        if token == contract::address() {
            return U256::MAX - self.total_supply();
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
    ) -> Result<bool, Self::Error> {
        let max_loan = self.max_flash_loan(token);
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
        self._mint(receiver, amount)?;

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

        // let flash_fee_receiver = self.flash_fee_receiver_address.get();
        // self._spend_allowance(receiver, contract::address(), amount + fee)?;
        // if fee.is_zero() || flash_fee_receiver.is_zero() {
        //     self._burn(receiver, amount + fee)?;
        // } else {
        //     self._burn(receiver, amount)?;
        //     self._transfer(receiver, flash_fee_receiver, fee)?;
        // }

        Ok(true)
    }
}

#[public]
impl IErc20 for Erc20FlashMint {
    type Error = crate::token::erc20::Error;

    /// Returns the number of tokens in existence.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn total_supply(&self) -> U256 {
        self.erc20.total_supply()
    }

    /// Returns the number of tokens owned by `account`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - Account to get balance from.
    fn balance_of(&self, account: Address) -> U256 {
        self.erc20.balance_of(account)
    }

    /// Moves a `value` amount of tokens from the caller's account to `to`.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    ///
    /// # Errors
    ///
    /// * If the `to` address is `Address::ZERO`, then the error
    /// [`crate::token::erc20::Error::InvalidReceiver`] is returned.
    /// * If the caller doesn't have a balance of at least `value`, then the
    /// error [`crate::token::erc20::Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`crate::token::erc20::Transfer`] event.
    fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.transfer(to, value)
    }

    /// Returns the remaining number of tokens that `spender` will be allowed
    /// to spend on behalf of `owner` through `transfer_from`. This is zero by
    /// default.
    ///
    /// This value changes when `approve` or `transfer_from` are called.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }

    /// Sets a `value` number of tokens as the allowance of `spender` over the
    /// caller's tokens.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// WARNING: Beware that changing an allowance with this method brings the
    /// risk that someone may use both the old and the new allowance by
    /// unfortunate transaction ordering. One possible solution to mitigate
    /// this race condition is to first reduce the `spender`'s allowance to 0
    /// and set the desired value afterwards:
    /// <https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729>
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    /// * `value` - The number of tokens being allowed to transfer by `spender`.
    ///
    /// # Errors
    ///
    /// If the `spender` address is `Address::ZERO`, then the error
    /// [`crate::token::erc20::Error::InvalidSpender`] is returned.
    ///
    /// # Events
    ///
    /// Emits an [`crate::token::erc20::Approval`] event.
    fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.approve(spender, value)
    }

    /// Moves a `value` number of tokens from `from` to `to` using the
    /// allowance mechanism. `value` is then deducted from the caller's
    /// allowance.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// NOTE: If `value` is the maximum `U256::MAX`, the allowance is not
    /// updated on `transfer_from`. This is semantically equivalent to
    /// an infinite approval.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    ///
    /// # Errors
    ///
    /// * If the `from` address is `Address::ZERO`, then the error
    /// [`crate::token::erc20::Error::InvalidSender`] is returned.
    /// * If the `to` address is `Address::ZERO`, then the error
    /// [`crate::token::erc20::Error::InvalidReceiver`] is returned.
    /// * If not enough allowance is available, then the error
    /// [`crate::token::erc20::Error::InsufficientAllowance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`crate::token::erc20::Transfer`] event.
    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.transfer_from(from, to, value)
    }
}

impl IErc165 for Erc20FlashMint {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IErc20>::INTERFACE_ID == u32::from_be_bytes(*interface_id)
            || <Self as IErc3156FlashLender>::INTERFACE_ID
                == u32::from_be_bytes(*interface_id)
            || Erc165::supports_interface(interface_id)
    }
}

impl Erc20FlashMint {
    /// Sets a `value` number of tokens as the allowance of `spender` over the
    /// caller's tokens.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    /// * `emit_event` - Emit an [`Approval`] event flag.
    ///
    /// # Errors
    ///
    /// If the `spender` address is `Address::ZERO`, then the error
    /// [`Error::InvalidSpender`] is returned.
    ///
    /// # Events
    ///
    /// Emits an [`Approval`] event.
    fn _approve(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
        emit_event: bool,
    ) -> Result<bool, crate::token::erc20::Error> {
        self.erc20._approve(owner, spender, value, emit_event)
    }

    /// Internal implementation of transferring tokens between two accounts.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - The number of tokens to transfer.
    ///
    /// # Errors
    ///
    /// * If the `from` address is `Address::ZERO`, then the error
    ///   [`Error::InvalidSender`] is returned.
    /// * If the `to` address is `Address::ZERO`, then the error
    ///   [`Error::InvalidReceiver`] is returned.
    /// * If the `from` address doesn't have enough tokens, then the error
    ///   [`Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    fn _transfer(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), crate::token::erc20::Error> {
        self.erc20._transfer(from, to, value)
    }

    /// Creates a `value` amount of tokens and assigns them to `account`,
    /// by transferring it from `Address::ZERO`.
    ///
    /// Relies on the `_update` mechanism.
    ///
    /// # Panics
    ///
    /// If `_total_supply` exceeds `U256::MAX`.
    ///
    /// # Errors
    ///
    /// If the `account` address is `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn _mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), crate::token::erc20::Error> {
        self.erc20._mint(account, value)
    }

    /// Transfers a `value` amount of tokens from `from` to `to`, or
    /// alternatively mints (or burns) if `from` (or `to`) is the zero address.
    ///
    /// All customizations to transfers, mints, and burns should be done by
    /// using this function.
    ///
    /// # Arguments
    ///
    /// * `from` - Owner's address.
    /// * `to` - Recipient's address.
    /// * `value` - Amount to be transferred.
    ///
    /// # Panics
    ///
    /// If `_total_supply` exceeds `U256::MAX`. It may happen during `mint`
    /// operation.
    ///
    /// # Errors
    ///
    /// If the `from` address doesn't have enough tokens, then the error
    /// [`Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn _update(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), crate::token::erc20::Error> {
        self.erc20._update(from, to, value)
    }

    /// Destroys a `value` amount of tokens from `account`,
    /// lowering the total supply.
    ///
    /// Relies on the `update` mechanism.
    ///
    /// # Arguments
    ///
    /// * `account` - Owner's address.
    /// * `value` - Amount to be burnt.
    ///
    /// # Errors
    ///
    /// * If the `from` address is `Address::ZERO`, then the error
    ///   [`Error::InvalidSender`] is returned.
    /// * If the `from` address doesn't have enough tokens, then the error
    ///   [`Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn _burn(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), crate::token::erc20::Error> {
        self.erc20._burn(account, value)
    }

    /// Updates `owner`'s allowance for `spender` based on spent `value`.
    ///
    /// Does not update the allowance value in the case of infinite allowance.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Account to transfer tokens from.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - The number of tokens to transfer.
    ///
    /// # Errors
    ///
    /// If not enough allowance is available, then the error
    /// [`Error::InsufficientAllowance`] is returned.
    pub fn _spend_allowance(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), crate::token::erc20::Error> {
        self.erc20._spend_allowance(owner, spender, value)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {

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
}
