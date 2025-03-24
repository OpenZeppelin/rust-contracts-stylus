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
use alloy_sol_macro::sol;
use stylus_sdk::{
    call::{Call, MethodError},
    contract, msg,
    prelude::*,
    storage::{StorageAddress, StorageU8},
};

use crate::token::erc20::{
    self,
    utils::{safe_erc20, IErc20 as IErc20Solidity, ISafeErc20, SafeErc20},
    Erc20, IErc20,
};

sol! {
    /// Indicates that the address is not a valid ERC-20 token.
    ///
    /// * `token` - Address of the invalid ERC-20 token.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20InvalidUnderlying(address token);

    /// Indicates that the address is not a valid sender address.
    ///
    /// * `sender` - Address of the invalid sender.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20InvalidSender(address sender);

    /// Indicates that the address is not a valid receiver addresss.
    ///
    /// * `receiver` - Address of the invalid receiver.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20InvalidReceiver(address receiver);

}

/// An [`Erc20Wrapper`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error),

    /// Error type from [`SafeErc20`] contract [`safe_erc20::Error`].
    SafeErc20(safe_erc20::Error),

    /// The Sender Address is not valid.
    InvalidSender(ERC20InvalidSender),

    /// The Receiver Address is not valid.
    InvalidReceiver(ERC20InvalidReceiver),

    /// The underlying token couldn't be wrapped.
    InvalidUnderlying(ERC20InvalidUnderlying),
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of an [`Erc20Wrapper`] token.
#[storage]
pub struct Erc20Wrapper {
    /// Token Address of the  underline token
    pub(crate) underlying: StorageAddress,
    /// Token decimals.
    pub(crate) underlying_decimals: StorageU8,
    /// [`SafeErc20`] contract.
    safe_erc20: SafeErc20,
}

/// ERC-20 Wrapper Standard Interface
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
    fn underlying(&self) -> Address;

    /// Allow a user to deposit underlying tokens and mint the corresponding
    /// number of wrapped tokens.
    ///
    /// Arguments:
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to deposit tokens to.
    /// * `value` - The amount of tokens to deposit.
    /// * `erc20` - Write access to an [`Erc20`] contract.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If the `msg::sender()`'s address is the
    ///   `contract:address()`.
    /// * [`Error::InvalidReceiver`] - If the `account` address is a
    ///   `contract:address()`.
    /// * [`Error::SafeErc20`] - If caller lacks sufficient balance or hasn't
    ///   approved enough tokens to the [`Erc20Wrapper`] contract.
    /// * [`Error::Erc20`] - If an error occurrs during [`Erc20::_mint`]
    ///   operation.
    ///
    /// # Panics
    ///
    /// * If [`Erc20::_mint`] operation panics.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn deposit_for(&mut self, account: Address, value: U256) -> Result<bool, wrapper::Error> {
    ///     self.erc20_wrapper.deposit_for(account, value, &mut self.erc20)
    /// }
    /// ```
    fn deposit_for(
        &mut self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
    ) -> Result<bool, Self::Error>;

    /// Allow a user to burn a number of wrapped tokens and withdraw the
    /// corresponding number of underlying tokens.
    ///
    /// Arguments:
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to withdraw tokens from.
    /// * `value` - The amount of tokens to withdraw.
    /// * `erc20` - Write access to an [`Erc20`] contract.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If the `account`'s address is a
    ///   `contract:address()`.
    /// * [`Error::Erc20`] - If an error occurrs during [`Erc20::_burn`]
    ///   operation.
    /// * [`Error::SafeErc20`] - If the [`Erc20Wrapper`] contract lacks
    ///   sufficient balance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn withdraw_to(&mut self, account: Address, value: U256,) -> Result<bool, wrapper::Error> {
    ///    self.erc20_wrapper.withdraw_to(account, value, &mut self.erc20)
    /// }
    /// ```
    fn withdraw_to(
        &mut self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
    ) -> Result<bool, Self::Error>;
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc20Wrapper {}

impl IErc20Wrapper for Erc20Wrapper {
    type Error = Error;

    fn decimals(&self) -> U8 {
        self.underlying_decimals.get()
    }

    fn underlying(&self) -> Address {
        self.underlying.get()
    }

    fn deposit_for(
        &mut self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
    ) -> Result<bool, Self::Error> {
        let contract_address = contract::address();
        let sender = msg::sender();

        if sender == contract_address {
            return Err(ERC20InvalidSender { sender }.into());
        }

        if account == contract_address {
            return Err(ERC20InvalidReceiver { receiver: account }.into());
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

    fn withdraw_to(
        &mut self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
    ) -> Result<bool, Self::Error> {
        if account == contract::address() {
            return Err(ERC20InvalidReceiver { receiver: account }.into());
        }

        erc20._burn(msg::sender(), value)?;

        self.safe_erc20.safe_transfer(self.underlying(), account, value)?;

        Ok(true)
    }
}

impl Erc20Wrapper {
    /// Mint wrapped token to cover any underlying tokens that would have been
    /// transferred by mistake or acquired from rebasing mechanisms.
    ///
    /// Internal function that can be exposed with access control if desired.
    ///
    /// Arguments:
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to mint tokens to.
    /// * `erc20` - Write access to an [`Erc20`] contract.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidUnderlying`]  - If the external call for
    ///   [`IErc20::balance_of`] fails.
    /// * [`Error::Erc20`] - If an error occurrs during [`Erc20::_mint`]
    ///   operation.
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

        let underline_token = IErc20Solidity::new(self.underlying());

        let underlying_balance = underline_token
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

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::uint;
    use motsu::prelude::*;

    use super::*;

    #[storage]
    struct Erc20WrapperTestExample {
        wrapper: Erc20Wrapper,
        erc20: Erc20,
    }

    #[public]
    impl Erc20WrapperTestExample {
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

        fn recover(&mut self, account: Address) -> Result<U256, Error> {
            self.wrapper._recover(account, &mut self.erc20)
        }
    }

    unsafe impl TopLevelStorage for Erc20WrapperTestExample {}

    #[motsu::test]
    fn decimals_works(
        contract: Contract<Erc20WrapperTestExample>,
        alice: Address,
    ) {
        let decimals = uint!(18_U8);
        contract.init(alice, |contract| {
            contract.wrapper.underlying_decimals.set(decimals);
        });

        assert_eq!(contract.sender(alice).decimals(), decimals);
    }

    #[motsu::test]
    fn underlying_works(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let erc20_address = erc20_contract.address();

        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc20_address);
        });

        assert_eq!(contract.sender(alice).underlying(), erc20_address);
    }

    #[motsu::test]
    fn deposit_for_reverts_when_invalid_asset(
        contract: Contract<Erc20WrapperTestExample>,
        alice: Address,
    ) {
        let invalid_asset = alice;
        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(invalid_asset);
        });

        let err = contract
            .sender(alice)
            .deposit_for(invalid_asset, uint!(10_U256))
            .motsu_expect_err("should return Error::SafeErc20");

        assert!(matches!(
            err,
            Error::SafeErc20(safe_erc20::Error::SafeErc20FailedOperation(
                safe_erc20::SafeErc20FailedOperation { token }
            )) if token == invalid_asset
        ));
    }

    #[motsu::test]
    fn deposit_for_reverts_when_invalid_sender(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let invalid_sender = contract.address();

        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc20_contract.address());
        });

        let err = contract
            .sender(invalid_sender)
            .deposit_for(alice, uint!(10_U256))
            .motsu_expect_err("should return Error::InvalidSender");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC20InvalidSender { sender }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn deposit_for_reverts_when_invalid_receiver(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let invalid_receiver = contract.address();

        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc20_contract.address());
        });

        let err = contract
            .sender(alice)
            .deposit_for(invalid_receiver, uint!(10_U256))
            .motsu_expect_err("should return Error::InvalidReceiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC20InvalidReceiver { receiver }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn deposit_for_reverts_when_insufficient_allowance(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let amount = uint!(10_U256);

        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc20_contract.address());
        });

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
            Error::SafeErc20(safe_erc20::Error::SafeErc20FailedOperation(
                safe_erc20::SafeErc20FailedOperation { token }
            )) if token == erc20_contract.address()
        ));
    }

    #[motsu::test]
    fn deposit_for_reverts_when_insufficient_balance(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let amount = uint!(10_U256);

        let exceeding_value = amount + uint!(1_U256);

        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc20_contract.address());
        });

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
            Error::SafeErc20(safe_erc20::Error::SafeErc20FailedOperation(
                safe_erc20::SafeErc20FailedOperation { token }
            )) if token == erc20_contract.address()
        ));
    }

    #[motsu::test]
    fn deposit_for_works(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let amount = uint!(10_U256);

        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc20_contract.address());
        });

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
        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc20_contract.address());
        });

        let err = contract
            .sender(alice)
            .withdraw_to(invalid_receiver, uint!(10_U256))
            .motsu_expect_err("should return Error::InvalidReceiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC20InvalidReceiver { receiver }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn withdraw_to_reverts_when_insufficient_balance(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let amount = uint!(10_U256);

        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc20_contract.address());
        });

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

        let exceeding_value = amount + uint!(1_U256);

        let err = contract
            .sender(alice)
            .withdraw_to(alice, exceeding_value)
            .motsu_expect_err("should return Error::SafeErc20");

        assert!(matches!(
            err,
            Error::Erc20(erc20::Error::InsufficientBalance(
                erc20::ERC20InsufficientBalance {
                    sender,
                    balance,
                    needed
                }
            )) if sender == alice && balance == amount && needed == exceeding_value
        ));
    }

    #[motsu::test]
    fn withdraw_to_works(
        contract: Contract<Erc20WrapperTestExample>,
        erc20_contract: Contract<Erc20>,
        alice: Address,
    ) {
        let amount = uint!(10_U256);

        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc20_contract.address());
        });

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

    // TODO: Should be a test for the `Error::InvalidUnderlying` error,
    // but impossible with current motsu limitations.
    #[motsu::test]
    #[ignore]
    fn recover_reverts_when_invalid_underlying(
        contract: Contract<Erc20WrapperTestExample>,
        invalid_underlying: Contract<crate::access::ownable::Ownable>,
        alice: Address,
    ) {
        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(invalid_underlying.address());
        });

        let err = contract
            .sender(alice)
            .recover(alice)
            .motsu_expect_err("should return Error::InvalidUnderlying");

        assert!(matches!(
            err, Error::InvalidUnderlying(ERC20InvalidUnderlying { token }) if token == invalid_underlying.address()
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

        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc20_contract.address());
        });

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

        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc20_contract.address());
        });

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

        contract.init(alice, |contract| {
            contract.wrapper.underlying.set(erc20_contract.address());
        });

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
        let unexpected_delta = uint!(1_U256);
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
}
