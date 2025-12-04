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

// TODO: once Erc20Votes is implemented, include it in the comment above next to
// Erc20Capped.

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, B256, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    abi::Bytes,
    call::{Call, MethodError},
    contract, msg,
    prelude::*,
    storage::{StorageAddress, StorageU256},
};

use crate::token::erc20::{self, Erc20, IErc20};

/// The expected value returned from
/// [`Erc3156FlashBorrowerInterface::on_flash_loan`].
pub const BORROWER_CALLBACK_VALUE: [u8; 32] = keccak_const::Keccak256::new()
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

        /// Indicate an error related to the loan value exceeding the maximum.
        ///
        /// * `max_loan` - Maximum loan value.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC3156ExceededMaxLoan(uint256 max_loan);

        /// Indicate that the receiver of a flashloan is not a valid [`Erc3156FlashBorrowerInterface::on_flash_loan`] implementer.
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
    /// Indicate an error related to the loan value exceeding the maximum.
    ExceededMaxLoan(ERC3156ExceededMaxLoan),
    /// Indicate that the receiver of a flashloan is not a valid
    /// [`Erc3156FlashBorrowerInterface::on_flash_loan`] implementer.
    ERC3156InvalidReceiver(ERC3156InvalidReceiver),
    /// Indicates an error related to the current balance of `sender`. Used in
    /// transfers.
    InsufficientBalance(erc20::ERC20InsufficientBalance),
    /// Indicates a failure with the token `sender`. Used in transfers.
    InvalidSender(erc20::ERC20InvalidSender),
    /// Indicates a failure with the token `receiver`. Used in transfers.
    InvalidReceiver(erc20::ERC20InvalidReceiver),
    /// Indicates a failure with the `spender`’s `allowance`. Used in
    /// transfers.
    InsufficientAllowance(erc20::ERC20InsufficientAllowance),
    /// Indicates a failure with the `spender` to be approved. Used in
    /// approvals.
    InvalidSpender(erc20::ERC20InvalidSpender),
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals. approver Address initiating an approval operation.
    InvalidApprover(erc20::ERC20InvalidApprover),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl From<erc20::Error> for Error {
    fn from(value: erc20::Error) -> Self {
        match value {
            erc20::Error::InsufficientBalance(e) => {
                Error::InsufficientBalance(e)
            }
            erc20::Error::InvalidSender(e) => Error::InvalidSender(e),
            erc20::Error::InvalidReceiver(e) => Error::InvalidReceiver(e),
            erc20::Error::InsufficientAllowance(e) => {
                Error::InsufficientAllowance(e)
            }
            erc20::Error::InvalidSpender(e) => Error::InvalidSpender(e),
            erc20::Error::InvalidApprover(e) => Error::InvalidApprover(e),
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

use crate::token::erc20::abi::Erc3156FlashBorrowerInterface;

/// State of an [`Erc20FlashMint`] Contract.
#[storage]
pub struct Erc20FlashMint {
    // TODO: Remove this field once function overriding is possible. For now we
    // keep this field `pub`, since this is used to simulate overriding.
    /// Fee applied when doing flash loans.
    pub flash_fee_value: StorageU256,
    // TODO: Remove this field once function overriding is possible. For now we
    // keep this field `pub`, since this is used to simulate overriding.
    /// Receiver address of the flash fee.
    pub flash_fee_receiver_address: StorageAddress,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc20FlashMint {}

/// Interface of the ERC-3156 Flash Borrower, as defined in
/// [ERC-3156](https://eips.ethereum.org/EIPS/eip-3156).
pub trait IErc3156FlashBorrower {
    /// Receive a flash loan.
    ///
    /// # Arguments
    ///
    /// * `initiator` - The initiator of the loan.
    /// * `token` - The loan currency.
    /// * `amount` - The amount of tokens lent.
    /// * `fee` - The additional amount of tokens to repay.
    /// * `data` - Arbitrary data structure, intended to contain user-defined
    ///   parameters.
    ///
    /// # Errors
    ///
    /// * May return a custom error.
    fn on_flash_loan(
        &mut self,
        initiator: Address,
        token: Address,
        amount: U256,
        fee: U256,
        data: Bytes,
    ) -> Result<B256, Vec<u8>>;
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
    /// * `&self` - Read access to the contract's state.
    /// * `token` - The address of the token that is requested.
    #[must_use]
    fn max_flash_loan(&self, token: Address) -> U256;

    /// Returns the fee applied when doing flash loans.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token` - The token to be flash loaned.
    /// * `value` - The amount of tokens to be loaned.
    ///
    /// # Errors
    ///
    /// * [`Error::UnsupportedToken`] - If the token is not supported.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn flash_fee(&self, token: Address, value: U256) -> Result<U256, flash_mint::Error> {
    ///     self.erc20_flash_mint.flash_fee(token, value)
    /// }
    /// ```
    fn flash_fee(
        &self,
        token: Address,
        value: U256,
    ) -> Result<U256, Self::Error>;

    /// Performs a flash loan.
    ///
    /// New tokens are minted and sent to the `receiver`, who is required to
    /// implement the [`Erc3156FlashBorrowerInterface`] interface. By the end of
    /// the flash loan, the receiver is expected to own value + fee tokens
    /// and have them approved back to the token contract itself so they can
    /// be burned.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `receiver` - The receiver of the flash loan. Should implement the
    ///   [`Erc3156FlashBorrowerInterface::on_flash_loan`] interface.
    /// * `token` - The token to be flash loaned. Only [`contract::address()`]
    ///   is supported.
    /// * `value` - The amount of tokens to be loaned.
    /// * `data` - Arbitrary data that is passed to the receiver.
    ///
    /// # Errors
    ///
    /// * [`Error::ExceededMaxLoan`] - If the `value` is greater than the value
    ///   returned by [`IErc3156FlashLender::max_flash_loan`].
    /// * [`Error::UnsupportedToken`] - If `token` is not supported.
    /// * [`Error::InvalidReceiver`] - If the `token` address is not a contract
    ///   , the contract fails to execute the call, or the receiver does not
    ///   return [`BORROWER_CALLBACK_VALUE`].
    ///
    /// # Events
    ///
    /// * [`erc20::Transfer`].
    /// * [`erc20::Approval`].
    ///
    /// # Panics
    ///
    /// * If the new (temporary) total supply exceeds [`U256::MAX`].
    /// * If the sum of the loan value and fee exceeds the maximum value of
    ///   [`U256::MAX`].
    fn flash_loan(
        &mut self,
        receiver: Address,
        token: Address,
        value: U256,
        data: Bytes,
    ) -> Result<bool, Self::Error>;
}

impl Erc20FlashMint {
    /// See [`IErc3156FlashLender::max_flash_loan`].
    #[must_use]
    pub fn max_flash_loan(&self, token: Address, erc20: &erc20::Erc20) -> U256 {
        if token == contract::address() {
            U256::MAX - erc20.total_supply()
        } else {
            U256::MIN
        }
    }

    /// See [`IErc3156FlashLender::flash_fee`].
    #[allow(clippy::missing_errors_doc)]
    pub fn flash_fee(
        &self,
        token: Address,
        _value: U256,
    ) -> Result<U256, Error> {
        if token == contract::address() {
            Ok(self.flash_fee_value.get())
        } else {
            Err(Error::UnsupportedToken(ERC3156UnsupportedToken { token }))
        }
    }

    // This function can reenter, but it doesn't pose a risk because it always
    // preserves the property that the amount minted at the beginning is always
    // recovered and burned at the end, or else the entire function will revert.
    /// See [`IErc3156FlashLender::flash_loan`].
    #[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
    pub fn flash_loan(
        &mut self,
        receiver: Address,
        token: Address,
        value: U256,
        data: &Bytes,
        erc20: &mut Erc20,
    ) -> Result<bool, Error> {
        let max_loan = self.max_flash_loan(token, erc20);
        if value > max_loan {
            return Err(Error::ExceededMaxLoan(ERC3156ExceededMaxLoan {
                max_loan,
            }));
        }

        let fee = self.flash_fee(token, value)?;
        if !Address::has_code(&receiver) {
            return Err(Error::ERC3156InvalidReceiver(
                ERC3156InvalidReceiver { receiver },
            ));
        }
        erc20._mint(receiver, value)?;
        let loan_receiver = Erc3156FlashBorrowerInterface::new(receiver);
        let loan_return = loan_receiver
            .on_flash_loan(
                Call::new_in(self),
                msg::sender(),
                token,
                value,
                fee,
                data.to_vec().into(),
            )
            .map_err(|_| {
                Error::ERC3156InvalidReceiver(ERC3156InvalidReceiver {
                    receiver,
                })
            })?;
        if loan_return != BORROWER_CALLBACK_VALUE {
            return Err(Error::ERC3156InvalidReceiver(
                ERC3156InvalidReceiver { receiver },
            ));
        }

        let allowance = value
            .checked_add(fee)
            .expect("allowance should not exceed `U256::MAX`");
        erc20._spend_allowance(receiver, contract::address(), allowance)?;

        let flash_fee_receiver = self.flash_fee_receiver_address.get();

        if fee.is_zero() || flash_fee_receiver.is_zero() {
            erc20._burn(receiver, allowance)?;
        } else {
            erc20._burn(receiver, value)?;
            erc20._transfer(receiver, flash_fee_receiver, fee)?;
        }

        Ok(true)
    }
}

// TODO: implement `IErc165` once `IErc3156FlashLender` is implemented for
// `Erc20FlashMint`.
// impl IErc165 for Erc20FlashMint {
//     fn supports_interface(&self, interface_id: B32) -> bool {
//         <Self as IErc3156FlashLender>::interface_id() == interface_id
//             || <Self as IErc165>::interface_id() == interface_id
//     }
// }

#[cfg(test)]
mod tests {
    use alloy_primitives::B256;
    use motsu::prelude::*;
    use stylus_sdk::{
        abi::Bytes,
        alloy_primitives::{aliases::B32, uint, Address, U256},
        prelude::*,
    };

    use super::*;
    use crate::token::erc20::abi::Erc20Interface;

    // --- Borrower mocks ---
    #[storage]
    struct WrongSelectorBorrower;

    unsafe impl TopLevelStorage for WrongSelectorBorrower {}

    // Provide a router for the concrete type as well
    #[public]
    #[implements(IErc3156FlashBorrower)]
    impl WrongSelectorBorrower {}

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[public]
    impl IErc3156FlashBorrower for WrongSelectorBorrower {
        fn on_flash_loan(
            &mut self,
            _initiator: Address,
            _token: Address,
            _amount: U256,
            _fee: U256,
            _data: Bytes,
        ) -> Result<B256, Vec<u8>> {
            // Return an incorrect selector to trigger the wrong-selector branch
            Ok(B256::ZERO)
        }
    }

    #[storage]
    struct CallbackOkNoApproveBorrower;

    unsafe impl TopLevelStorage for CallbackOkNoApproveBorrower {}

    // Provide a router for the concrete type as well
    #[public]
    #[implements(IErc3156FlashBorrower)]
    impl CallbackOkNoApproveBorrower {}

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[public]
    impl IErc3156FlashBorrower for CallbackOkNoApproveBorrower {
        fn on_flash_loan(
            &mut self,
            _initiator: Address,
            _token: Address,
            _amount: U256,
            _fee: U256,
            _data: Bytes,
        ) -> Result<B256, Vec<u8>> {
            // Signal success but do not set allowance, so spend_allowance fails
            Ok(super::BORROWER_CALLBACK_VALUE.into())
        }
    }

    #[storage]
    struct GoodBorrower;

    unsafe impl TopLevelStorage for GoodBorrower {}

    // Provide a router for the concrete type as well
    #[public]
    #[implements(IErc3156FlashBorrower)]
    impl GoodBorrower {}

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[public]
    impl IErc3156FlashBorrower for GoodBorrower {
        fn on_flash_loan(
            &mut self,
            _initiator: Address,
            token: Address,
            amount: U256,
            fee: U256,
            _data: Bytes,
        ) -> Result<B256, Vec<u8>> {
            // Approve the token contract itself to pull back amount + fee
            let allowance = amount
                .checked_add(fee)
                .expect("allowance should not exceed `U256::MAX`");
            let token_iface = Erc20Interface::new(token);
            let ok =
                token_iface.approve(Call::new_in(self), token, allowance)?;
            if !ok {
                return Err(b"approve returned false".to_vec());
            }
            Ok(super::BORROWER_CALLBACK_VALUE.into())
        }
    }

    #[storage]
    struct ErrorBorrower;

    unsafe impl TopLevelStorage for ErrorBorrower {}

    // Provide a router for the concrete type as well
    #[public]
    #[implements(IErc3156FlashBorrower)]
    impl ErrorBorrower {}

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[public]
    impl IErc3156FlashBorrower for ErrorBorrower {
        fn on_flash_loan(
            &mut self,
            _initiator: Address,
            _token: Address,
            _amount: U256,
            _fee: U256,
            _data: Bytes,
        ) -> Result<B256, Vec<u8>> {
            Err("Borrower error".into())
        }
    }

    #[storage]
    struct Erc20FlashMintTestExample {
        erc20_flash_mint: Erc20FlashMint,
        erc20: Erc20,
    }

    unsafe impl TopLevelStorage for Erc20FlashMintTestExample {}

    #[public]
    #[implements(IErc3156FlashLender<Error = Error>, IErc20<Error = Error>)]
    impl Erc20FlashMintTestExample {}

    #[public]
    impl IErc3156FlashLender for Erc20FlashMintTestExample {
        type Error = Error;

        fn max_flash_loan(&self, token: Address) -> U256 {
            self.erc20_flash_mint.max_flash_loan(token, &self.erc20)
        }

        fn flash_fee(
            &self,
            token: Address,
            value: U256,
        ) -> Result<U256, Self::Error> {
            self.erc20_flash_mint.flash_fee(token, value)
        }

        fn flash_loan(
            &mut self,
            receiver: Address,
            token: Address,
            value: U256,
            data: Bytes,
        ) -> Result<bool, Self::Error> {
            self.erc20_flash_mint.flash_loan(
                receiver,
                token,
                value,
                &data,
                &mut self.erc20,
            )
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[public]
    impl IErc20 for Erc20FlashMintTestExample {
        type Error = Error;

        fn total_supply(&self) -> U256 {
            self.erc20.total_supply()
        }

        fn balance_of(&self, account: Address) -> U256 {
            self.erc20.balance_of(account)
        }

        fn transfer(
            &mut self,
            to: Address,
            value: U256,
        ) -> Result<bool, Self::Error> {
            Ok(self.erc20.transfer(to, value)?)
        }

        fn allowance(&self, owner: Address, spender: Address) -> U256 {
            self.erc20.allowance(owner, spender)
        }

        fn approve(
            &mut self,
            spender: Address,
            value: U256,
        ) -> Result<bool, Self::Error> {
            Ok(self.erc20.approve(spender, value)?)
        }

        fn transfer_from(
            &mut self,
            from: Address,
            to: Address,
            value: U256,
        ) -> Result<bool, Self::Error> {
            Ok(self.erc20.transfer_from(from, to, value)?)
        }
    }

    #[motsu::test]
    fn max_flash_loan_token_match(
        contract: Contract<Erc20FlashMintTestExample>,
        alice: Address,
    ) {
        let max_flash_loan =
            contract.sender(alice).max_flash_loan(contract.address());
        assert_eq!(max_flash_loan, U256::MAX);
    }

    #[motsu::test]
    fn max_flash_loan_token_mismatch(
        contract: Contract<Erc20FlashMintTestExample>,
        alice: Address,
    ) {
        let max_flash_loan = contract.sender(alice).max_flash_loan(alice);
        assert_eq!(max_flash_loan, U256::MIN);
    }

    #[motsu::test]
    fn max_flash_loan_when_token_minted(
        contract: Contract<Erc20FlashMintTestExample>,
        alice: Address,
    ) {
        let initial_supply = uint!(10000_U256);

        contract
            .sender(alice)
            .erc20
            ._mint(alice, initial_supply)
            .motsu_expect("should mint initial supply tokens");

        let max_flash_loan =
            contract.sender(alice).max_flash_loan(contract.address());

        assert_eq!(max_flash_loan, U256::MAX - initial_supply);
    }

    #[motsu::test]
    fn flash_fee(
        contract: Contract<Erc20FlashMintTestExample>,
        alice: Address,
    ) {
        let flash_fee_value = uint!(69_U256);
        contract
            .sender(alice)
            .erc20_flash_mint
            .flash_fee_value
            .set(flash_fee_value);

        let flash_fee = contract
            .sender(alice)
            .flash_fee(contract.address(), uint!(1000_U256))
            .motsu_expect("should return flash fee value");

        assert_eq!(flash_fee, flash_fee_value);
    }

    #[motsu::test]
    fn flash_fee_reverts_when_invalid_token(
        contract: Contract<Erc20FlashMintTestExample>,
        alice: Address,
    ) {
        let invalid_token = alice;

        let err = contract
            .sender(alice)
            .flash_fee(invalid_token, uint!(1000_U256))
            .motsu_expect_err("should return Error::UnsupportedToken");

        assert!(matches!(
            err,
            Error::UnsupportedToken(ERC3156UnsupportedToken { token })
                if token == invalid_token
        ));
    }

    #[motsu::test]
    fn flash_loan_reverts_when_receiver_is_invalid(
        contract: Contract<Erc20FlashMintTestExample>,
        alice: Address,
    ) {
        // A very hacky way of forcing the receiver to have code, but fail the
        // flash loan call during `_mint`. This is a workaround for our
        // current implementation which makes it impossible to meaningfully
        // override the `_mint` function to force it to fail.
        let invalid_receiver_with_code: Contract<Erc20FlashMintTestExample> =
            Contract::<Erc20FlashMintTestExample>::new_at(Address::ZERO);

        let err = contract
            .sender(alice)
            .flash_loan(
                invalid_receiver_with_code.address(),
                contract.address(),
                U256::MAX,
                vec![0, 1].into(),
            )
            .motsu_expect_err("should return Error::InvalidReceiver");

        assert!(
            matches!(err, Error::InvalidReceiver(erc20::ERC20InvalidReceiver { receiver }) if invalid_receiver_with_code.address() == receiver)
        );
    }

    #[motsu::test]
    fn flash_loan_reverts_when_exceeded_max_loan(
        contract: Contract<Erc20FlashMintTestExample>,
        alice: Address,
    ) {
        let initial_supply = uint!(10000_U256);

        contract
            .sender(alice)
            .erc20
            ._mint(alice, initial_supply)
            .motsu_expect("should mint initial supply tokens");

        let err = contract
            .sender(alice)
            .flash_loan(alice, contract.address(), U256::MAX, vec![0, 1].into())
            .motsu_expect_err("should return Error::ExceededMaxLoan");

        assert!(matches!(
            err,
            Error::ExceededMaxLoan(ERC3156ExceededMaxLoan { max_loan })
                if max_loan == U256::MAX - initial_supply
        ));
    }

    #[motsu::test]
    fn flash_loan_reverts_when_receiver_is_zero_address(
        contract: Contract<Erc20FlashMintTestExample>,
        alice: Address,
    ) {
        let invalid_reciver = Address::ZERO;
        let err = contract
            .sender(alice)
            .flash_loan(
                invalid_reciver,
                contract.address(),
                uint!(1000_U256),
                vec![0, 1].into(),
            )
            .motsu_expect_err("should return Error::InvalidReceiver");

        assert!(matches!(
            err,
            Error::ERC3156InvalidReceiver(ERC3156InvalidReceiver { receiver }) if receiver == invalid_reciver
        ));
    }

    #[motsu::test]
    fn flash_loan_reverts_when_invalid_receiver(
        contract: Contract<Erc20FlashMintTestExample>,
        alice: Address,
    ) {
        let invalid_receiver = alice;

        let err = contract
            .sender(alice)
            .flash_loan(
                invalid_receiver,
                contract.address(),
                uint!(1000_U256),
                vec![0, 1].into(),
            )
            .motsu_expect_err("should return Error::InvalidReceiver");

        assert!(matches!(
            err,
            Error::ERC3156InvalidReceiver(ERC3156InvalidReceiver { receiver })
                if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn flash_loan_reverts_when_callback_returns_wrong_selector(
        contract: Contract<Erc20FlashMintTestExample>,
        borrower: Contract<WrongSelectorBorrower>,
        alice: Address,
    ) {
        // Ensure fee could be anything; default zero is fine.
        let err = contract
            .sender(alice)
            .flash_loan(
                borrower.address(),
                contract.address(),
                uint!(123_U256),
                vec![].into(),
            )
            .motsu_expect_err("should revert due to wrong selector");

        assert!(matches!(
            err,
            Error::ERC3156InvalidReceiver(ERC3156InvalidReceiver { receiver })
                if receiver == borrower.address()
        ));
    }

    #[motsu::test]
    fn flash_loan_reverts_when_insufficient_allowance_after_callback(
        contract: Contract<Erc20FlashMintTestExample>,
        borrower: Contract<CallbackOkNoApproveBorrower>,
        alice: Address,
    ) {
        // Set a non-zero fee to require allowance for value + fee
        contract
            .sender(alice)
            .erc20_flash_mint
            .flash_fee_value
            .set(uint!(5_U256));

        let err = contract
            .sender(alice)
            .flash_loan(
                borrower.address(),
                contract.address(),
                uint!(100_U256),
                vec![0xAA].into(),
            )
            .motsu_expect_err("should revert due to insufficient allowance");

        assert!(
            matches!(err, Error::InsufficientAllowance(erc20::ERC20InsufficientAllowance { spender, allowance, needed }) if spender == contract.address() && allowance.is_zero() && needed == uint!(100_U256) + uint!(5_U256))
        );
    }

    #[motsu::test]
    fn flash_loan_reverts_borrower_reverts(
        contract: Contract<Erc20FlashMintTestExample>,
        borrower: Contract<ErrorBorrower>,
        alice: Address,
    ) {
        // Set a non-zero fee to require allowance for value + fee
        contract
            .sender(alice)
            .erc20_flash_mint
            .flash_fee_value
            .set(uint!(5_U256));

        let err = contract
            .sender(alice)
            .flash_loan(
                borrower.address(),
                contract.address(),
                uint!(100_U256),
                vec![0xAA].into(),
            )
            .motsu_expect_err("should revert due to insufficient allowance");

        assert!(matches!(err,
            Error::ERC3156InvalidReceiver(ERC3156InvalidReceiver { receiver }) if receiver == borrower.address()));
    }

    #[motsu::test]
    fn flash_loan_succeeds_when_fee_or_receiver_zero(
        contract: Contract<Erc20FlashMintTestExample>,
        borrower: Contract<GoodBorrower>,
        alice: Address,
        bob: Address,
    ) {
        let amount = uint!(100_U256);

        // Case 1: fee = 0, receiver != 0 -> burns amount
        contract.sender(alice).erc20_flash_mint.flash_fee_value.set(U256::ZERO);
        contract
            .sender(alice)
            .erc20_flash_mint
            .flash_fee_receiver_address
            .set(bob);

        let ok = contract
            .sender(alice)
            .flash_loan(
                borrower.address(),
                contract.address(),
                amount,
                vec![].into(),
            )
            .motsu_expect("flash loan should succeed when fee is zero");
        assert!(ok);
        let receiver_balance =
            contract.sender(alice).erc20.balance_of(borrower.address());
        let fee_receiver_balance = contract.sender(alice).erc20.balance_of(bob);
        let total_supply = contract.sender(alice).erc20.total_supply();
        assert_eq!(receiver_balance, U256::ZERO);
        assert_eq!(fee_receiver_balance, U256::ZERO);
        assert_eq!(total_supply, U256::ZERO);

        // Case 2: fee != 0, receiver = 0 -> burns amount + fee
        let fee = uint!(5_U256);
        contract.sender(alice).erc20_flash_mint.flash_fee_value.set(fee);
        contract
            .sender(alice)
            .erc20_flash_mint
            .flash_fee_receiver_address
            .set(Address::ZERO);

        // Pre-fund borrower with 'fee' tokens to allow burning amount + fee
        contract
            .sender(alice)
            .erc20
            ._mint(borrower.address(), fee)
            .motsu_expect("prefund fee to borrower");

        let ok = contract
            .sender(alice)
            .flash_loan(
                borrower.address(),
                contract.address(),
                amount,
                vec![0x01].into(),
            )
            .motsu_expect(
                "flash loan should succeed when fee receiver is zero",
            );
        assert!(ok);
        let receiver_balance =
            contract.sender(alice).erc20.balance_of(borrower.address());
        let fee_receiver_balance =
            contract.sender(alice).erc20.balance_of(Address::ZERO);
        let total_supply = contract.sender(alice).erc20.total_supply();
        assert_eq!(receiver_balance, U256::ZERO);
        assert_eq!(fee_receiver_balance, U256::ZERO);
        assert_eq!(total_supply, U256::ZERO);

        // Case 3: fee = 0, receiver = 0 -> burns amount
        contract.sender(alice).erc20_flash_mint.flash_fee_value.set(U256::ZERO);
        contract
            .sender(alice)
            .erc20_flash_mint
            .flash_fee_receiver_address
            .set(Address::ZERO);

        let ok = contract
            .sender(alice)
            .flash_loan(
                borrower.address(),
                contract.address(),
                amount,
                vec![0x02].into(),
            )
            .motsu_expect(
                "flash loan should succeed when both fee and receiver are zero",
            );
        assert!(ok);
        let receiver_balance =
            contract.sender(alice).erc20.balance_of(borrower.address());
        let total_supply = contract.sender(alice).erc20.total_supply();
        assert_eq!(receiver_balance, U256::ZERO);
        assert_eq!(total_supply, U256::ZERO);
    }

    #[motsu::test]
    fn flash_loan_succeeds_with_fee_and_fee_receiver(
        contract: Contract<Erc20FlashMintTestExample>,
        borrower: Contract<GoodBorrower>,
        alice: Address,
        bob: Address,
    ) {
        let amount = uint!(100_U256);
        let fee = uint!(7_U256);

        // Set non-zero fee and non-zero fee receiver
        contract.sender(alice).erc20_flash_mint.flash_fee_value.set(fee);
        contract
            .sender(alice)
            .erc20_flash_mint
            .flash_fee_receiver_address
            .set(bob);

        // Prefund borrower with 'fee' tokens so they can repay value + fee
        contract
            .sender(alice)
            .erc20
            ._mint(borrower.address(), fee)
            .motsu_expect("prefund fee to borrower");

        let ok = contract
            .sender(alice)
            .flash_loan(
                borrower.address(),
                contract.address(),
                amount,
                vec![0xAB].into(),
            )
            .motsu_expect(
                "flash loan should succeed with fee and fee receiver",
            );
        assert!(ok);

        let receiver_balance =
            contract.sender(alice).erc20.balance_of(borrower.address());
        let fee_receiver_balance = contract.sender(alice).erc20.balance_of(bob);
        let total_supply = contract.sender(alice).erc20.total_supply();

        assert_eq!(receiver_balance, U256::ZERO);
        assert_eq!(fee_receiver_balance, fee);
        // After burning `amount`, only `fee` remains in circulation at `bob`.
        assert_eq!(total_supply, fee);
    }

    #[motsu::test]
    fn interface_id() {
        let actual =
            <Erc20FlashMintTestExample as IErc3156FlashLender>::interface_id();
        let expected: B32 = 0xe4143091_u32.into();
        assert_eq!(actual, expected);
    }
}
