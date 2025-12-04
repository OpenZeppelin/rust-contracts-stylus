//! Extension of the ERC-20 token contract to support token wrapping.
//!
//! Users can deposit and withdraw "underlying tokens" and receive a matching
//! number of "wrapped tokens". This is useful in conjunction with other
//! modules.
//!
//! WARNING: Any mechanism in which the underlying token changes the
//! [`IErc20::balance_of`] of an account without an explicit transfer may
//! desynchronize this contract's supply and its underlying balance. Please
//! exercise caution when wrapping tokens that may undercollateralize the
//! wrapper (i.e. wrapper's total supply is higher than its underlying balance).
//! See [`Erc20Wrapper::_recover`] for recovering value accrued to the wrapper.

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, U256, U8};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    call::{Call, MethodError},
    contract, msg,
    prelude::*,
    storage::StorageAddress,
};

use crate::token::erc20::{
    self,
    abi::{Erc20Interface, Erc20MetadataInterface},
    utils::{safe_erc20, ISafeErc20, SafeErc20},
    Erc20, IErc20,
};

/// Default number of decimals for an [ERC-20] token.
///
/// [ERC-20]: <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.3.0/contracts/token/ERC20/ERC20.sol>
const DEFAULT_DECIMALS: u8 = 18;

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Indicates that the address is not a valid ERC-20 token.
        ///
        /// * `token` - Address of the invalid ERC-20 token.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC20InvalidUnderlying(address token);
    }
}

/// An [`Erc20Wrapper`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
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
    /// An operation with an ERC-20 token failed.
    SafeErc20FailedOperation(safe_erc20::SafeErc20FailedOperation),
    /// Indicates a failed [`ISafeErc20::safe_decrease_allowance`] request.
    SafeErc20FailedDecreaseAllowance(
        safe_erc20::SafeErc20FailedDecreaseAllowance,
    ),
    /// The underlying token couldn't be wrapped.
    InvalidUnderlying(ERC20InvalidUnderlying),
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
impl From<safe_erc20::Error> for Error {
    fn from(value: safe_erc20::Error) -> Self {
        match value {
            safe_erc20::Error::SafeErc20FailedOperation(e) => {
                Error::SafeErc20FailedOperation(e)
            }
            safe_erc20::Error::SafeErc20FailedDecreaseAllowance(e) => {
                Error::SafeErc20FailedDecreaseAllowance(e)
            }
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of an [`Erc20Wrapper`] token.
#[storage]
pub struct Erc20Wrapper {
    /// Address of the underlying token.
    pub(crate) underlying: StorageAddress,
    /// [`SafeErc20`] contract.
    safe_erc20: SafeErc20,
}

/// ERC-20 Wrapper Standard Interface
#[interface_id]
pub trait IErc20Wrapper {
    /// The error type associated to the trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the number of decimals used to get its user representation.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn decimals(&self) -> U8 {
    ///     self.erc20_wrapper.decimals()
    /// }
    /// ```
    #[must_use]
    fn decimals(&self) -> U8;

    /// Returns the address of the underlying ERC-20 token that is being
    /// wrapped.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn underlying(&self) -> Address {
    ///     self.erc20_wrapper.underlying()
    /// }
    /// ```
    #[must_use]
    fn underlying(&self) -> Address;

    /// Allow a user to deposit underlying tokens and mint the corresponding
    /// number of wrapped tokens.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to deposit tokens to.
    /// * `value` - The amount of tokens to deposit.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If the `msg::sender()`'s address is the
    ///   `contract:address()`.
    /// * [`Error::InvalidReceiver`] - If the `account` address is a
    ///   `contract:address()`.
    /// * [`Error::SafeErc20FailedOperation`] - If caller lacks sufficient
    ///   balance or hasn't approved enough tokens to the [`Erc20Wrapper`]
    ///   contract.
    /// * [`Error::InvalidReceiver`] - If the `account` address is
    ///   [`Address::ZERO`].
    ///
    /// # Panics
    ///
    /// * If [`Erc20::_mint`] operation panics.
    fn deposit_for(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<bool, Self::Error>;

    /// Allow a user to burn a number of wrapped tokens and withdraw the
    /// corresponding number of underlying tokens.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to withdraw tokens from.
    /// * `value` - The amount of tokens to withdraw.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If the `account`'s address is a
    ///   `contract:address()`.
    /// * [`Error::InvalidSender`] - If the `from` address is [`Address::ZERO`].
    /// * [`Error::InsufficientBalance`] - If the `from` address doesn't have
    ///   enough tokens.
    /// * [`Error::SafeErc20FailedOperation`] - If the [`Erc20Wrapper`] contract
    ///   lacks sufficient balance.
    fn withdraw_to(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<bool, Self::Error>;
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc20Wrapper {}

impl Erc20Wrapper {
    /// See [`IErc20Wrapper::decimals`].
    #[must_use]
    pub fn decimals(&self) -> U8 {
        U8::from(
            Erc20MetadataInterface::new(self.underlying())
                .decimals(self)
                .unwrap_or(DEFAULT_DECIMALS),
        )
    }

    /// See [`IErc20Wrapper::underlying`].
    #[must_use]
    pub fn underlying(&self) -> Address {
        self.underlying.get()
    }

    /// See [`IErc20Wrapper::deposit_for`].
    #[allow(clippy::missing_errors_doc)]
    pub fn deposit_for(
        &mut self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
    ) -> Result<bool, Error> {
        let contract_address = contract::address();
        let sender = msg::sender();

        if sender == contract_address {
            return Err(erc20::ERC20InvalidSender { sender }.into());
        }

        if account == contract_address {
            return Err(
                erc20::ERC20InvalidReceiver { receiver: account }.into()
            );
        }

        self.safe_erc20.safe_transfer_from(
            self.underlying(),
            sender,
            contract_address,
            value,
        )?;

        erc20._mint(account, value)?;

        Ok(true)
    }

    /// See [`IErc20Wrapper::withdraw_to`].
    #[allow(clippy::missing_errors_doc)]
    pub fn withdraw_to(
        &mut self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
    ) -> Result<bool, Error> {
        if account == contract::address() {
            return Err(
                erc20::ERC20InvalidReceiver { receiver: account }.into()
            );
        }

        erc20._burn(msg::sender(), value)?;

        self.safe_erc20.safe_transfer(self.underlying(), account, value)?;

        Ok(true)
    }
}

#[public]
impl Erc20Wrapper {
    /// Constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `underlying_token` - The wrapped token.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidUnderlying`] - If underlying token is this contract.
    #[constructor]
    pub fn constructor(
        &mut self,
        underlying_token: Address,
    ) -> Result<(), Error> {
        if underlying_token == contract::address() {
            return Err(Error::InvalidUnderlying(ERC20InvalidUnderlying {
                token: underlying_token,
            }));
        }
        self.underlying.set(underlying_token);
        Ok(())
    }
}

impl Erc20Wrapper {
    /// Mint wrapped token to cover any underlying tokens that would have been
    /// transferred by mistake or acquired from rebasing mechanisms.
    ///
    /// Internal function that can be exposed with access control if desired.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to mint tokens to.
    /// * `erc20` - Write access to an [`Erc20`] contract.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidUnderlying`]  - If the external call for
    ///   [`IErc20::balance_of`] fails.
    /// * [`Error::InvalidReceiver`] - If the `account` address is
    ///   [`Address::ZERO`].
    ///
    /// # Panics
    ///
    /// * If the underlying balance is less than the [`IErc20::total_supply`].
    pub fn _recover(
        &mut self,
        account: Address,
        erc20: &mut Erc20,
    ) -> Result<U256, Error> {
        let contract_address = contract::address();

        let underlying_token = Erc20Interface::new(self.underlying());

        let underlying_balance = underlying_token
            .balance_of(Call::new_in(self), contract_address)
            .map_err(|_| ERC20InvalidUnderlying { token: contract_address })?;

        let value = underlying_balance
            .checked_sub(erc20.total_supply())
            .expect("underlying balance should be greater than the `IErc20::total_supply`");

        if value > U256::ZERO {
            erc20._mint(account, value)?;
        }

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{aliases::B32, uint};
    use motsu::prelude::*;

    use super::*;
    use crate::{
        token::erc20::extensions::IErc20Metadata,
        utils::introspection::erc165::IErc165,
    };

    const DUMMY_TEST_DECIMALS: u8 = 12;
    #[storage]
    struct DummyErc20Metadata {}

    #[public]
    #[implements(IErc20Metadata, IErc165)]
    impl DummyErc20Metadata {}

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[public]
    impl IErc20Metadata for DummyErc20Metadata {
        fn name(&self) -> String {
            "DummyErc20Metadata".into()
        }

        fn symbol(&self) -> String {
            "TTK".into()
        }

        fn decimals(&self) -> U8 {
            U8::from(DUMMY_TEST_DECIMALS)
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[public]
    impl IErc165 for DummyErc20Metadata {
        fn supports_interface(&self, _interface_id: B32) -> bool {
            // dummy implementation, required by [`IErc20Metadata`] trait.
            true
        }
    }

    unsafe impl TopLevelStorage for DummyErc20Metadata {}

    #[storage]
    struct Erc20WrapperTestExample {
        wrapper: Erc20Wrapper,
        erc20: Erc20,
    }

    #[public]
    #[implements(IErc20Wrapper<Error = Error>)]
    impl Erc20WrapperTestExample {
        #[constructor]
        fn constructor(
            &mut self,
            underlying_token: Address,
        ) -> Result<(), Error> {
            self.wrapper.constructor(underlying_token)
        }

        fn recover(&mut self, account: Address) -> Result<U256, Error> {
            self.wrapper._recover(account, &mut self.erc20)
        }
    }

    #[public]
    impl IErc20Wrapper for Erc20WrapperTestExample {
        type Error = Error;

        fn decimals(&self) -> U8 {
            self.wrapper.decimals()
        }

        fn underlying(&self) -> Address {
            self.wrapper.underlying()
        }

        fn deposit_for(
            &mut self,
            account: Address,
            value: U256,
        ) -> Result<bool, Error> {
            self.wrapper.deposit_for(account, value, &mut self.erc20)
        }

        fn withdraw_to(
            &mut self,
            account: Address,
            value: U256,
        ) -> Result<bool, Error> {
            self.wrapper.withdraw_to(account, value, &mut self.erc20)
        }
    }

    unsafe impl TopLevelStorage for Erc20WrapperTestExample {}

    #[motsu::test]
    fn decimals_works(
        contract: Contract<Erc20WrapperTestExample>,
        metadata: Contract<DummyErc20Metadata>,
        alice: Address,
    ) {
        contract
            .sender(alice)
            .constructor(metadata.address())
            .motsu_expect("should construct");
        assert_eq!(
            contract.sender(alice).decimals(),
            U8::from(DUMMY_TEST_DECIMALS)
        );
    }

    #[motsu::test]
    fn underlying_works(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let erc20_address = erc20_contract.address();

        contract
            .sender(alice)
            .constructor(erc20_address)
            .motsu_expect("should construct");

        assert_eq!(contract.sender(alice).underlying(), erc20_address);
    }

    #[motsu::test]
    fn constructor_reverts_when_invalid_asset(
        contract: Contract<Erc20WrapperTestExample>,
        alice: Address,
    ) {
        let invalid_asset = contract.address();

        let err = contract
            .sender(alice)
            .constructor(invalid_asset)
            .motsu_expect_err("should return Error::InvalidUnderlying");

        assert!(matches!(
            err,
            Error::InvalidUnderlying(ERC20InvalidUnderlying { token })
                if token == invalid_asset
        ));
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[motsu::test]
    #[ignore = "TODO: unignore once motsu fixes https://github.com/OpenZeppelin/stylus-test-helpers/issues/115."]
    fn deposit_for_reverts_when_invalid_asset(
        contract: Contract<Erc20WrapperTestExample>,
        alice: Address,
    ) {
        // assume an invalid underlying asset is somehow set in the contract
        let invalid_asset = alice;
        contract.sender(alice).wrapper.underlying.set(invalid_asset);

        let err = contract
            .sender(alice)
            .deposit_for(alice, uint!(10_U256))
            .motsu_expect_err("should return Error::SafeErc20FailedOperation");

        assert!(matches!(
            err,
            Error::SafeErc20FailedOperation(
                safe_erc20::SafeErc20FailedOperation { token }
            ) if token == invalid_asset
        ));
    }

    #[motsu::test]
    fn deposit_for_reverts_when_invalid_sender(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let invalid_sender = contract.address();

        contract
            .sender(alice)
            .constructor(erc20_contract.address())
            .motsu_expect("should construct");

        let err = contract
            .sender(invalid_sender)
            .deposit_for(alice, uint!(10_U256))
            .motsu_expect_err("should return Error::InvalidSender");

        assert!(matches!(
            err,
            Error::InvalidSender(erc20::ERC20InvalidSender { sender }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn deposit_for_reverts_when_invalid_receiver(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let invalid_receiver = contract.address();

        contract
            .sender(alice)
            .constructor(erc20_contract.address())
            .motsu_expect("should construct");

        let err = contract
            .sender(alice)
            .deposit_for(invalid_receiver, uint!(10_U256))
            .motsu_expect_err("should return Error::InvalidReceiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(erc20::ERC20InvalidReceiver { receiver }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn deposit_for_reverts_when_insufficient_allowance(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let amount = uint!(10_U256);

        contract
            .sender(alice)
            .constructor(erc20_contract.address())
            .motsu_expect("should construct");

        erc20_contract
            .sender(alice)
            ._mint(alice, amount)
            .motsu_expect("should mint");

        let err = contract
            .sender(alice)
            .deposit_for(alice, amount)
            .motsu_expect_err("should return Error::SafeErc20");

        assert!(matches!(
            err,
            Error::SafeErc20FailedOperation(
                safe_erc20::SafeErc20FailedOperation { token }
            ) if token == erc20_contract.address()
        ));
    }

    #[motsu::test]
    fn deposit_for_reverts_when_insufficient_balance(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let amount = uint!(10_U256);

        let exceeding_value = amount + U256::ONE;

        contract
            .sender(alice)
            .constructor(erc20_contract.address())
            .motsu_expect("should construct");

        erc20_contract
            .sender(alice)
            ._mint(alice, amount)
            .motsu_expect("should mint");

        erc20_contract
            .sender(alice)
            .approve(contract.address(), exceeding_value)
            .motsu_expect("should approve");

        let err = contract
            .sender(alice)
            .deposit_for(alice, exceeding_value)
            .motsu_expect_err("should return Error::SafeErc20");

        assert!(matches!(
            err,
            Error::SafeErc20FailedOperation(
                safe_erc20::SafeErc20FailedOperation { token }
            ) if token == erc20_contract.address()
        ));
    }

    #[motsu::test]
    fn deposit_for_works(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let amount = uint!(10_U256);

        contract
            .sender(alice)
            .constructor(erc20_contract.address())
            .motsu_expect("should construct");

        erc20_contract
            .sender(alice)
            ._mint(alice, amount)
            .motsu_expect("should mint");

        let initial_balance = erc20_contract.sender(alice).balance_of(alice);
        let initial_wrapped_balance =
            contract.sender(alice).erc20.balance_of(alice);

        let initial_contract_balance =
            erc20_contract.sender(alice).balance_of(contract.address());

        let initial_wrapped_supply =
            contract.sender(alice).erc20.total_supply();

        erc20_contract
            .sender(alice)
            .approve(contract.address(), amount)
            .motsu_expect("should approve");

        assert!(contract
            .sender(alice)
            .deposit_for(alice, amount)
            .motsu_expect("should deposit"));

        erc20_contract.assert_emitted(&erc20::Transfer {
            from: alice,
            to: contract.address(),
            value: amount,
        });

        contract.assert_emitted(&erc20::Transfer {
            from: Address::ZERO,
            to: alice,
            value: amount,
        });

        assert_eq!(
            erc20_contract.sender(alice).balance_of(alice),
            initial_balance - amount
        );

        assert_eq!(
            contract.sender(alice).erc20.balance_of(alice),
            initial_wrapped_balance + amount
        );

        assert_eq!(
            erc20_contract
                .sender(contract.address())
                .balance_of(contract.address()),
            initial_contract_balance + amount
        );

        assert_eq!(
            contract.sender(alice).erc20.total_supply(),
            initial_wrapped_supply + amount
        );
    }

    #[motsu::test]
    fn withdraw_to_reverts_when_invalid_receiver(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let invalid_receiver = contract.address();
        contract
            .sender(alice)
            .constructor(erc20_contract.address())
            .motsu_expect("should construct");

        let err = contract
            .sender(alice)
            .withdraw_to(invalid_receiver, uint!(10_U256))
            .motsu_expect_err("should return Error::InvalidReceiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(erc20::ERC20InvalidReceiver { receiver }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn withdraw_to_reverts_when_insufficient_balance(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let amount = uint!(10_U256);

        contract
            .sender(alice)
            .constructor(erc20_contract.address())
            .motsu_expect("should construct");

        erc20_contract
            .sender(alice)
            ._mint(alice, amount)
            .motsu_expect("should mint");

        erc20_contract
            .sender(alice)
            .approve(contract.address(), amount)
            .motsu_expect("should approve");

        contract
            .sender(alice)
            .deposit_for(alice, amount)
            .motsu_expect("should deposit");

        let exceeding_value = amount + U256::ONE;

        let err = contract
            .sender(alice)
            .withdraw_to(alice, exceeding_value)
            .motsu_expect_err("should return Error::SafeErc20");

        assert!(matches!(
            err,
            Error::InsufficientBalance(
                erc20::ERC20InsufficientBalance {
                    sender,
                    balance,
                    needed
                }
            ) if sender == alice && balance == amount && needed == exceeding_value
        ));
    }

    #[motsu::test]
    fn withdraw_to_works(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let amount = uint!(10_U256);

        contract
            .sender(alice)
            .constructor(erc20_contract.address())
            .motsu_expect("should construct");

        erc20_contract
            .sender(alice)
            ._mint(alice, amount)
            .motsu_expect("should mint");

        erc20_contract
            .sender(alice)
            .approve(contract.address(), amount)
            .motsu_expect("should approve");

        contract
            .sender(alice)
            .deposit_for(alice, amount)
            .motsu_expect("should deposit");

        let initial_balance = erc20_contract.sender(alice).balance_of(alice);
        let initial_wrapped_balance =
            contract.sender(alice).erc20.balance_of(alice);

        let initial_contract_balance =
            erc20_contract.sender(alice).balance_of(contract.address());

        let initial_wrapped_supply =
            contract.sender(alice).erc20.total_supply();

        assert!(contract
            .sender(alice)
            .withdraw_to(alice, amount)
            .motsu_expect("should withdraw"));

        contract.assert_emitted(&erc20::Transfer {
            from: alice,
            to: Address::ZERO,
            value: amount,
        });

        erc20_contract.assert_emitted(&erc20::Transfer {
            from: contract.address(),
            to: alice,
            value: amount,
        });

        assert_eq!(
            erc20_contract.sender(alice).balance_of(alice),
            initial_balance + amount
        );

        assert_eq!(
            contract.sender(alice).erc20.balance_of(alice),
            initial_wrapped_balance - amount
        );

        assert_eq!(
            erc20_contract
                .sender(contract.address())
                .balance_of(contract.address()),
            initial_contract_balance - amount
        );

        assert_eq!(
            contract.sender(alice).erc20.total_supply(),
            initial_wrapped_supply - amount
        );
    }

    #[storage]
    struct InvalidUnderlyingToken;

    unsafe impl TopLevelStorage for InvalidUnderlyingToken {}

    #[public]
    #[allow(clippy::unused_self)]
    impl InvalidUnderlyingToken {
        fn balance_of(&self, _account: Address) -> Result<U256, Vec<u8>> {
            Err("InvalidUnderlying".into())
        }
    }

    // TODO: update when Erc20Wrapper returns Vec<u8> on all errors: https://github.com/OpenZeppelin/rust-contracts-stylus/issues/800
    #[motsu::test]
    #[ignore = "TODO: un-ignore when motsu supports returning empty revert reasons, see: https://github.com/OpenZeppelin/stylus-test-helpers/issues/118"]
    fn recover_reverts_when_invalid_underlying(
        contract: Contract<Erc20WrapperTestExample>,
        invalid_underlying: Contract<InvalidUnderlyingToken>,
        alice: Address,
    ) {
        contract
            .sender(alice)
            .constructor(invalid_underlying.address())
            .motsu_unwrap();

        let err = contract
            .sender(alice)
            .recover(alice)
            .motsu_expect_err("should return Error::InvalidUnderlying");

        assert!(matches!(
            err, Error::InvalidUnderlying(ERC20InvalidUnderlying { token }) if token == contract.address()
        ));
    }

    #[motsu::test]
    #[should_panic = "underlying balance should be greater than the `IErc20::total_supply`"]
    fn recover_panics_when_underlying_balance_is_less_than_total_supply(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let amount = uint!(10_U256);

        contract
            .sender(alice)
            .constructor(erc20_contract.address())
            .motsu_expect("should construct");

        erc20_contract
            .sender(alice)
            ._mint(alice, amount)
            .motsu_expect("should mint");

        erc20_contract
            .sender(alice)
            .approve(contract.address(), amount)
            .motsu_expect("should approve");

        contract
            .sender(alice)
            .deposit_for(alice, amount)
            .motsu_expect("should deposit");

        // Unexpected mint.
        contract
            .sender(alice)
            .erc20
            ._mint(alice, amount)
            .motsu_expect("should mint");

        // This should panic.
        _ = contract.sender(alice).recover(alice);
    }

    #[motsu::test]
    fn recover_works_when_underlying_balance_is_equal_to_total_supply(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let amount = uint!(10_U256);

        contract
            .sender(alice)
            .constructor(erc20_contract.address())
            .motsu_expect("should construct");

        erc20_contract
            .sender(alice)
            ._mint(alice, amount)
            .motsu_expect("should mint");

        erc20_contract
            .sender(alice)
            .approve(contract.address(), amount)
            .motsu_expect("should approve");

        contract
            .sender(alice)
            .deposit_for(alice, amount)
            .motsu_expect("should deposit");

        assert_eq!(
            contract
                .sender(alice)
                .recover(alice)
                .motsu_expect("should recover"),
            U256::ZERO
        );
    }

    #[motsu::test]
    fn recover_works_when_underlying_balance_is_greater_than_total_supply(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let amount = uint!(10_U256);

        contract
            .sender(alice)
            .constructor(erc20_contract.address())
            .motsu_expect("should construct");

        erc20_contract
            .sender(alice)
            ._mint(alice, amount)
            .motsu_expect("should mint");

        erc20_contract
            .sender(alice)
            .approve(contract.address(), amount)
            .motsu_expect("should approve");

        contract
            .sender(alice)
            .deposit_for(alice, amount)
            .motsu_expect("should deposit");

        // Unexpected mint.
        let unexpected_delta = U256::ONE;
        erc20_contract
            .sender(alice)
            ._mint(contract.address(), unexpected_delta)
            .motsu_expect("should mint");

        assert_eq!(
            contract
                .sender(alice)
                .recover(alice)
                .motsu_expect("should recover"),
            unexpected_delta
        );

        contract.assert_emitted(&erc20::Transfer {
            from: Address::ZERO,
            to: alice,
            value: unexpected_delta,
        });
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc20WrapperTestExample as IErc20Wrapper>::interface_id();
        let expected: B32 = 0x511f913e_u32.into();
        assert_eq!(actual, expected);
    }
}
