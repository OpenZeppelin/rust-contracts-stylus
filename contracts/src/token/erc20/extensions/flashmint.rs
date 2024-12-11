//! Optional Flashloan extension of the ERC-20 standard.
//! using the IERC3156FlashBorrower interface to borrow tokens.

use alloy_primitives::{b256, Address, B256, U256};
use alloy_sol_types::sol;
use stylus_sdk::{abi::Bytes, call::Call, contract, msg, prelude::*};

use crate::token::erc20::{self, Erc20, IErc20};

sol! {
    /// Indicate an error related to an unsupported loan token.
    /// This occurs when the specified token cannot be used for loans.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC3156UnsupportedToken(address token);

    /// Indicate an error related to the loan amount exceeds the maximum.
    /// The requested amount is higher than the allowed loan for this token max_loan.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC3156ExceededMaxLoan(uint256 max_loan);

    /// Indicate  an  error related to an invalid flash loan receiver.
    /// The receiver does not implement the required `onFlashLoan` function.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC3156InvalidReceiver(address receiver);

}

/// Extension of [`Erc20`] that allows token holders to destroy both
/// their own tokens and those that they have an allowance for,
/// in a way that can be recognized off-chain (via event analysis).
pub trait IERC3156FlashLender {
    /// The error type associated to this ERC-20 Burnable trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the maximum amount of tokens that can be borrowed
    /// from this contract in a flash loan.
    ///
    /// For tokens that are not supported, this function returns
    /// `U256::MIN`.
    ///
    /// * `token` - The address of the ERC-20 token that will be loaned.
    fn max_flash_loan(&self, token: Address) -> U256;

    /// Calculates the fee for a flash loan.
    ///
    /// The fee is a fixed percentage of the borrowed amount.
    ///
    /// If the token is not supported, the function returns an
    /// `UnsupportedToken` error.
    ///
    /// * `token` - The address of the ERC-20 token that will be loaned.
    /// * `amount` - The amount of tokens that will be loaned.
    fn flash_fee(
        &self,
        token: Address,
        amount: U256,
    ) -> Result<U256, Self::Error>;

    /// Executes a flash loan.
    ///
    /// This function is part of the ERC-3156 (Flash Loans) standard.
    ///
    /// * `receiver` - The contract that will receive the flash loan.
    /// * `token` - The ERC-20 token that will be loaned.
    /// * `amount` - The amount of tokens that will be loaned.
    /// * `data` - Arbitrary data that can be passed to the receiver contract.
    ///
    /// The function must return `true` if the flash loan was successful,
    /// and revert otherwise.
    fn flash_loan(
        &mut self,
        receiver: Address,
        token: Address,
        amount: U256,
        data: Bytes,
    ) -> Result<bool, Self::Error>;
}

/// A FlashlMint    error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicate an error related to an unsupported loan token.
    /// This occurs when the specified token cannot be used for loans.
    UnsupportedToken(ERC3156UnsupportedToken),

    /// Indicate an error related to the loan amount exceeds the maximum.
    /// The requested amount is higher than the allowed loan for this token
    /// max_loan.
    ExceededMaxLoan(ERC3156ExceededMaxLoan),

    /// Indicate  an  error related to an invalid flash loan receiver.
    /// The receiver does not implement the required `onFlashLoan` function.
    InvalidReceiver(ERC3156InvalidReceiver),

    /// Error type from [`Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error),
}

pub use borrower::IERC3156FlashBorrower;

#[allow(missing_docs)]
mod borrower {
    use stylus_sdk::stylus_proc::sol_interface;

    sol_interface! {
        /// Interface that must be implemented by smart contracts
        /// in order to borrow ERC-3156 flashloan .
        interface IERC3156FlashBorrower {
            /// Handles the receipt of a flash loan.
            /// This function is called after the loan amount has been transferred to the borrower.
            ///
            /// To indicate successful handling of the flash loan, this function should return
            /// the `keccak256` hash of "ERC3156FlashBorrower.onFlashLoan".
            ///
            /// # Arguments
            ///
            /// * `initiator` - The address which initiated the flash loan.
            /// * `token` - The address of the token being loaned (loan currency).
            /// * `amount` - The amount of tokens lent in the flash loan.
            /// * `fee` - The additional fee to repay with the flash loan amount.
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

const RETURN_VALUE: [u8; 32] = keccak_const::Keccak256::new()
    .update("ERC3156FlashBorrower.onFlashLoan".as_bytes())
    .finalize();

impl IERC3156FlashLender for Erc20 {
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
        amount: U256,
    ) -> Result<U256, Self::Error> {
        if token != contract::address() {
            return Err(Error::UnsupportedToken(ERC3156UnsupportedToken {
                token,
            }));
        }
        Ok(self._flash_fee(token, amount))
    }

    fn flash_loan(
        &mut self,
        receiver: Address,
        token: Address,
        value: U256,
        data: Bytes,
    ) -> Result<bool, Self::Error> {
        let max_loan = self.max_flash_loan(token);
        if value > max_loan {
            return Err(Error::ExceededMaxLoan(ERC3156ExceededMaxLoan {
                max_loan,
            }));
        }

        let fee = self.flash_fee(token, value)?;
        self._mint(receiver, value)?;
        let loan_reciver = IERC3156FlashBorrower::new(receiver);
        if Address::has_code(&loan_reciver) {
            return Err(Error::InvalidReceiver(ERC3156InvalidReceiver {
                receiver,
            }));
        }
        let call = Call::new();
        let loan_return = loan_reciver.on_flash_loan(
            call,
            msg::sender(),
            token,
            value,
            fee,
            data.to_vec().into(),
        );
        if loan_return.is_err() || loan_return.ok() != Some(RETURN_VALUE.into())
        {
            return Err(Error::InvalidReceiver(ERC3156InvalidReceiver {
                receiver,
            }));
        }

        let flash_fee_receiver = self._flash_fee_receiver();
        self._spend_allowance(receiver, msg::sender(), value + fee)?;
        if fee.is_zero() || flash_fee_receiver.is_zero() {
            self._burn(receiver, value + fee)?;
        } else {
            self._burn(receiver, value)?;
            self._transfer(receiver, flash_fee_receiver, fee)?;
        }

        Ok(true)
    }
}

impl Erc20 {
    /// Calculates the fee for a flash loan.
    ///
    /// The fee is currently fixed at 0.
    ///
    /// * `token` - The ERC-20 token that will be loaned.
    /// * `value` - The amount of tokens that will be loaned.
    pub fn _flash_fee(&self, token: Address, value: U256) -> U256 {
        let _ = token;
        let _ = value;
        return U256::MIN;
    }

    /// Returns the address of the receiver contract that will receive the flash
    /// loan. The default implementation returns `Address::ZERO`.
    pub fn _flash_fee_receiver(&self) -> Address {
        Address::ZERO
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {

    use alloc::vec;

    use alloy_primitives::{address, uint, Address, U256};
    use stylus_sdk::msg;

    use crate::token::erc20::{
        extensions::flashmint::{Error, IERC3156FlashLender},
        Erc20,
    };

    const ALICE: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    const TOKEN_ADDRESS: Address =
        address!("dce82b5f92c98f27f116f70491a487effdb6a2a9");
    const INVALID_TOKEN_ADDRESS: Address =
        address!("dce82b5f92c98f27f116f70491a487effdb6a2aa");

    #[motsu::test]
    fn max_flash_loan_token_match(contract: Erc20) {
        let max_flash_loan = contract.max_flash_loan(TOKEN_ADDRESS);
        assert_eq!(max_flash_loan, U256::MAX);
    }

    #[motsu::test]
    fn max_flash_loan_token_mismatch(contract: Erc20) {
        let max_flash_loan = contract.max_flash_loan(INVALID_TOKEN_ADDRESS);
        assert_eq!(max_flash_loan, U256::MIN);
    }

    #[motsu::test]
    fn max_flash_loan_when_token_minted(contract: Erc20) {
        contract._mint(msg::sender(), uint!(10000_U256)).unwrap();
        let max_flash_loan = contract.max_flash_loan(TOKEN_ADDRESS);
        assert_eq!(max_flash_loan, U256::MAX - uint!(10000_U256));
    }

    #[motsu::test]
    fn flash_fee(contract: Erc20) {
        let flash_fee =
            contract.flash_fee(TOKEN_ADDRESS, uint!(1000_U256)).unwrap();
        assert_eq!(flash_fee, U256::MIN);
    }

    #[motsu::test]
    fn error_flash_fee_when_invalid_token(contract: Erc20) {
        let result =
            contract.flash_fee(INVALID_TOKEN_ADDRESS, uint!(1000_U256));
        assert!(matches!(result, Err(Error::UnsupportedToken(_))));
    }

    #[motsu::test]
    fn error_flash_loan_when_exceeded_max_loan(contract: Erc20) {
        let _ = contract._mint(msg::sender(), uint!(10000_U256));
        let result = contract.flash_loan(
            msg::sender(),
            TOKEN_ADDRESS,
            U256::MAX,
            vec![0, 1].into(),
        );
        assert!(matches!(result, Err(Error::ExceededMaxLoan(_))));
    }

    #[motsu::test]
    fn flash_loan(contract: Erc20) {
        let _ = contract._mint(msg::sender(), uint!(10000_U256));
    }

    #[motsu::test]
    fn error_flash_loan_when_zero_receiver_address(contract: Erc20) {
        let invalid_reciver = Address::ZERO;
        let result = contract.flash_loan(
            invalid_reciver,
            TOKEN_ADDRESS,
            uint!(1000_U256),
            vec![0, 1].into(),
        );
        assert_eq!(result.is_err(), true);
    }

    #[motsu::test]
    fn error_flash_loan_when_invalid_receiver(contract: Erc20) {
        let result = contract.flash_loan(
            ALICE,
            TOKEN_ADDRESS,
            uint!(1000_U256),
            vec![0, 1].into(),
        );
        assert_eq!(result.is_err(), true);
    }
}
