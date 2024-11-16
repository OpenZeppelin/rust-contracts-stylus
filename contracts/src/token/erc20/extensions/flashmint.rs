//! Optional Flashloan extension of the ERC-20 standard.
//! using the IERC3156FlashBorrower interface to borrow tokens.

use alloc::borrow;
use alloy_primitives::{b256, Address, B256,Bytes, U256};
use alloy_sol_macro_expander::expand::ty;
use alloy_sol_types::sol;
use stylus_sdk::{call::Call, contract, msg, prelude::*, abi::Bytes as AbiBytes};

use crate::token::erc20::{
    self, utils::borrower::IERC3156FlashBorrower, 
    Erc20, IErc20,
};

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
        data: AbiBytes,
    ) -> Result<bool, Self::Error>;
}

/// A Permit error.
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


sol_storage! {
    pub struct Erc20FlashMint {
       uint256 _flash_fee_amount;
       address _flash_fee_receiver_address;
       Erc20 erc20;
    }
}



const RETURN_VALUE: B256 =
    b256!("439148f0bbc682ca079e46d6e2c2f0c1e3b820f1a291b069d8882abf8cf18dd9");


#[public]
impl IERC3156FlashLender for  Erc20FlashMint {
    type Error = Error;
    fn max_flash_loan(&self, token: Address) -> U256 {
        if token == contract::address() {
            return U256::MAX - self.erc20.total_supply();
        }
        U256::MIN
    }

    fn flash_fee(&self, token: Address, amount: U256) -> Result<U256,Self::Error> {
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
        data: AbiBytes,
    ) -> Result<bool, Self::Error> {
        let max_loan = self.max_flash_loan(token);
        if value > max_loan {
            return Err(Error::ExceededMaxLoan(ERC3156ExceededMaxLoan {
                max_loan,
            }));
        }

        let fee = self.flash_fee(token, value)?;
        self.erc20._mint(receiver, value)?;
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
            Bytes::from(data.0),
        );
        if loan_return.is_err() {
            return Err(Error::InvalidReceiver(ERC3156InvalidReceiver {
                receiver,
            }));
        }
        if loan_return.ok() != Some(RETURN_VALUE) {
            return Err(Error::InvalidReceiver(ERC3156InvalidReceiver {
                receiver,
            }));
        }

        let flash_fee_receiver = self._flash_fee_receiver();
        self.erc20._spend_allowance(receiver, msg::sender(), value + fee)?;
        if fee.is_zero() || flash_fee_receiver.is_zero() {
            self.erc20._burn(receiver, value + fee)?;
        } else {
            self.erc20._burn(receiver, value)?;
            self.erc20._transfer(receiver, flash_fee_receiver, fee)?;
        }

        Ok(true)
    }
}

impl Erc20FlashMint {
    /// Calculates the fee for a flash loan.
    ///
    /// The fee is currently fixed at 0.
    ///
    /// * `token` - The ERC-20 token that will be loaned.
    /// * `value` - The amount of tokens that will be loaned.
    pub fn _flash_fee(&self, token: Address, value: U256) -> U256 {
        let _ = token;
        let _ = value;
        if self._flash_fee_amount.is_zero() {
            return  U256::MIN
        }
        self._flash_fee_amount.get()
    }

    /// Returns the address of the receiver contract that will receive the flash
    /// loan. The default implementation returns `Address::ZERO`.
    pub fn _flash_fee_receiver(&self) -> Address {
        if self._flash_fee_receiver_address.eq(&Address::ZERO) {
            return self._flash_fee_receiver_address.get();
        }
        Address::ZERO
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address,uint,  U256};
    use stylus_sdk::msg;

    use crate::token::erc20::{Erc20, IErc20};

    use super::Erc20FlashMint;
    use crate::token::erc20::extensions::flashmint::IERC3156FlashLender;


    #[motsu::test]
    fn max_flash_loan_token_match(contract: Erc20FlashMint) {
        let token =  address!("dce82b5f92c98f27f116f70491a487effdb6a2a9");
        let max_flash_loan =  contract.max_flash_loan(token);
        assert_eq!(max_flash_loan, U256::MAX);
    }
    
    #[motsu::test]
    fn max_flash_loan_token_mismatch(contract: Erc20FlashMint) {
        let token =  address!("dce82b5f92c98f27f116f70491a487effdb6a2a6");
        let max_flash_loan =  contract.max_flash_loan(token);
        assert_eq!(max_flash_loan, U256::MIN);
    }

    #[motsu::test]
    fn burns_errors_when_insufficient_balance(contract: Erc20FlashMint) {
        let zero = U256::ZERO;
        let one = uint!(1_U256);
        let sender = msg::sender();

        assert_eq!(zero, contract.erc20.balance_of(sender));

        // let result = contract.burn(one);
        // assert!(matches!(result, Err(Error::InsufficientBalance(_))));
    }

    // #[motsu::test]
    // fn burn_from(contract: Erc20) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    //     let sender = msg::sender();

    //     // Alice approves `msg::sender`.
    //     let one = uint!(1_U256);
    //     contract._allowances.setter(alice).setter(sender).set(one);

    //     // Mint some tokens for Alice.
    //     let two = uint!(2_U256);
    //     contract._update(Address::ZERO, alice, two).unwrap();
    //     assert_eq!(two, contract.balance_of(alice));
    //     assert_eq!(two, contract.total_supply());

    //     contract.burn_from(alice, one).unwrap();

    //     assert_eq!(one, contract.balance_of(alice));
    //     assert_eq!(one, contract.total_supply());
    //     assert_eq!(U256::ZERO, contract.allowance(alice, sender));
    // }

    // #[motsu::test]
    // fn burns_from_errors_when_insufficient_balance(contract: Erc20) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

    //     // Alice approves `msg::sender`.
    //     let zero = U256::ZERO;
    //     let one = uint!(1_U256);

    //     contract._allowances.setter(alice).setter(msg::sender()).set(one);
    //     assert_eq!(zero, contract.balance_of(alice));

    //     let one = uint!(1_U256);

    //     let result = contract.burn_from(alice, one);
    //     assert!(matches!(result, Err(Error::InsufficientBalance(_))));
    // }

    // #[motsu::test]
    // fn burns_from_errors_when_invalid_approver(contract: Erc20) {
    //     let one = uint!(1_U256);

    //     contract
    //         ._allowances
    //         .setter(Address::ZERO)
    //         .setter(msg::sender())
    //         .set(one);

    //     let result = contract.burn_from(Address::ZERO, one);
    //     assert!(matches!(result, Err(Error::InvalidApprover(_))));
    // }

    // #[motsu::test]
    // fn burns_from_errors_when_insufficient_allowance(contract: Erc20) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

    //     // Mint some tokens for Alice.
    //     let one = uint!(1_U256);
    //     contract._update(Address::ZERO, alice, one).unwrap();
    //     assert_eq!(one, contract.balance_of(alice));

    //     let result = contract.burn_from(alice, one);
    //     assert!(matches!(result, Err(Error::InsufficientAllowance(_))));
    // }
}
