//! Implementation of the ERC-20 token standard.
//!
//! We have followed general OpenZeppelin Contracts guidelines: functions
//! revert instead of returning `false` on failure. This behavior is
//! nonetheless conventional and does not conflict with the expectations of
//! ERC-20 applications.
use stylus_proc::SolidityError;

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use stylus_sdk::{
    evm, msg,
    stylus_proc::{external, sol_storage},
};

sol! {
    /// Emitted when `value` tokens are moved from one account (`from`) to
    /// another (`to`).
    ///
    /// Note that `value` may be zero.
    event Transfer(address indexed from, address indexed to, uint256 value);
    /// Emitted when the allowance of a `spender` for an `owner` is set by a
    /// call to `approve`. `value` is the new allowance.
    event Approval(address indexed owner, address indexed spender, uint256 value);
}

sol! {
    /// Indicates an error related to the current `balance` of `sender`. Used
    /// in transfers.
    ///
    /// * `sender` - Address whose tokens are being transferred.
    /// * `balance` - Current balance for the interacting account.
    /// * `needed` - Minimum amount required to perform a transfer.
    error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
    /// Indicates a failure with the token `sender`. Used in transfers.
    ///
    /// * `sender` - Address whose tokens are being transferred.
    error ERC20InvalidSender(address sender);
    /// Indicates a failure with the token `receiver`. Used in transfers.
    ///
    /// * `receiver` - Address to which the tokens are being transferred.
    error ERC20InvalidReceiver(address receiver);
    /// Indicates a failure with the `spender`’s `allowance`. Used in
    /// transfers.
    ///
    /// * `spender` - Address that may be allowed to operate on tokens without
    /// being their owner.
    /// * `allowance` - Amount of tokens a `spender` is allowed to operate
    /// with.
    /// * `needed` - Minimum amount required to perform a transfer.
    error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals.
    ///
    /// * `approver` - Address initiating an approval operation.
    error ERC20InvalidApprover(address approver);
    /// Indicates a failure with the `spender` to be approved. Used in
    /// approvals.
    ///
    /// * `spender` - Address that may be allowed to operate on tokens without
    /// being their owner.
    error ERC20InvalidSpender(address spender);
}

/// An ERC-20 error defined as described in [ERC-6093].
///
/// [ERC-6093]: https://eips.ethereum.org/EIPS/eip-6093
#[derive(SolidityError)]
pub enum Error {
    /// Indicates an error related to the current balance of `sender`. Used in
    /// transfers.
    InsufficientBalance(ERC20InsufficientBalance),
    /// Indicates a failure with the token `sender`. Used in transfers.
    InvalidSender(ERC20InvalidSender),
    /// Indicates a failure with the token `receiver`. Used in transfers.
    InvalidReceiver(ERC20InvalidReceiver),
    /// Indicates a failure with the `spender`’s `allowance`. Used in
    /// transfers.
    InsufficientAllowance(ERC20InsufficientAllowance),
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals.
    InvalidApprover(ERC20InvalidApprover),
    /// Indicates a failure with the `spender` to be approved. Used in
    /// approvals.
    InvalidSpender(ERC20InvalidSpender),
}

sol_storage! {
    /// State of an ERC20 token.
    pub struct ERC20 {
        /// Maps users to balances.
        mapping(address => uint256) _balances;
        /// Maps users to a mapping of each spender's allowance.
        mapping(address => mapping(address => uint256)) _allowances;
        /// The total supply of the token.
        uint256 _total_supply;
    }
}

#[external]
impl ERC20 {
    /// Returns the number of tokens in existence.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn total_supply(&self) -> U256 {
        self._total_supply.get()
    }

    /// Returns the number of tokens owned by `account`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - Account to get balance from.
    pub fn balance_of(&self, account: Address) -> U256 {
        self._balances.get(account)
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
    /// [`Error::InvalidReceiver`] is returned.
    /// * If the caller doesn't have a balance of at least `value`, then the
    /// error [`Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, Error> {
        let from = msg::sender();
        if to == Address::ZERO {
            return Err(Error::InvalidReceiver(ERC20InvalidReceiver {
                receiver: Address::ZERO,
            }));
        }

        self._transfer(from, to, value)?;
        evm::log(Transfer { from, to, value });
        Ok(true)
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
    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self._allowances.get(owner).get(spender)
    }

    /// Sets a `value` number of tokens as the allowance of `spender` over the
    /// caller's tokens.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// WARNING: Beware that changing an allowance with this method brings the
    /// risk that someone may use both the old and the new allowance by
    /// unfortunate transaction ordering. One possible solution to mitigate
    /// this race condition is to first reduce the spender's allowance to 0 and
    /// set the desired value afterwards:
    /// <https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729>
    ///
    /// # Arguments
    ///
    /// * `&mutself` - Write access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    ///
    /// # Errors
    ///
    /// If the `spender` address is `Address::ZERO`, then the error
    /// [`Error::InvalidSpender`] is returned.
    ///
    /// # Events
    ///
    /// Emits an [`Approval`] event.
    pub fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, Error> {
        let owner = msg::sender();
        if spender == Address::ZERO {
            return Err(Error::InvalidSpender(ERC20InvalidSpender {
                spender: Address::ZERO,
            }));
        }

        self._allowances.setter(owner).insert(spender, value);
        evm::log(Approval { owner, spender, value });
        Ok(true)
    }

    /// Moves a `value` number of tokens from `from` to `to` using the
    /// allowance mechanism. `value` is then deducted from the caller's
    /// allowance.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// NOTE: If `value` is the maximum `uint256`, the allowance is not updated
    /// on `transferFrom`. This is semantically equivalent to an infinite
    /// approval.
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
    /// [`Error::InvalidSender`] is returned.
    /// * If the `to` address is `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// * If not enough allowance is available, then the error
    /// [`Error::InsufficientAllowance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Error> {
        if from == Address::ZERO {
            return Err(Error::InvalidSender(ERC20InvalidSender {
                sender: Address::ZERO,
            }));
        }
        if to == Address::ZERO {
            return Err(Error::InvalidReceiver(ERC20InvalidReceiver {
                receiver: Address::ZERO,
            }));
        }

        let spender = msg::sender();
        self._spend_allowance(from, spender, value)?;
        self._transfer(from, to, value)?;

        Ok(true)
    }
}

impl ERC20 {
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
    /// If the `from` address doesn't have enough tokens, then the error
    /// [`Error::InsufficientBalance`] is returned.
    fn _transfer(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Error> {
        let from_balance = self._balances.get(from);
        if from_balance < value {
            return Err(Error::InsufficientBalance(ERC20InsufficientBalance {
                sender: from,
                balance: from_balance,
                needed: value,
            }));
        }

        let from_balance = from_balance - value;
        self._balances.insert(from, from_balance);
        let to_balance = self._balances.get(to);
        self._balances.insert(to, to_balance + value);
        Ok(())
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
    ) -> Result<(), Error> {
        let current_allowance = self._allowances.get(owner).get(spender);
        if current_allowance != U256::MAX {
            if current_allowance < value {
                return Err(Error::InsufficientAllowance(
                    ERC20InsufficientAllowance {
                        spender,
                        allowance: current_allowance,
                        needed: value,
                    },
                ));
            }

            self._allowances
                .setter(owner)
                .insert(spender, current_allowance - value);
        }

        Ok(())
    }
}
