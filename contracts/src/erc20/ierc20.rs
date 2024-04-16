//! Module doc TODO!

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use stylus_proc::SolidityError;
use stylus_sdk::{evm, msg};

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
    #[derive(Debug)]
    error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
    /// Indicates a failure with the token `sender`. Used in transfers.
    ///
    /// * `sender` - Address whose tokens are being transferred.
    #[derive(Debug)]
    error ERC20InvalidSender(address sender);
    /// Indicates a failure with the token `receiver`. Used in transfers.
    ///
    /// * `receiver` - Address to which the tokens are being transferred.
    #[derive(Debug)]
    error ERC20InvalidReceiver(address receiver);
    /// Indicates a failure with the `spender`’s `allowance`. Used in
    /// transfers.
    ///
    /// * `spender` - Address that may be allowed to operate on tokens without
    /// being their owner.
    /// * `allowance` - Amount of tokens a `spender` is allowed to operate
    /// with.
    /// * `needed` - Minimum amount required to perform a transfer.
    #[derive(Debug)]
    error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
    /// Indicates a failure with the `spender` to be approved. Used in
    /// approvals.
    ///
    /// * `spender` - Address that may be allowed to operate on tokens without
    /// being their owner.
    #[derive(Debug)]
    error ERC20InvalidSpender(address spender);

}

/// An ERC-20 error defined as described in [ERC-6093].
///
/// [ERC-6093]: https://eips.ethereum.org/EIPS/eip-6093
#[derive(SolidityError, Debug)]
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
    /// Indicates a failure with the `spender` to be approved. Used in
    /// approvals.
    InvalidSpender(ERC20InvalidSpender),
    /// TODO!!!
    PausableError(crate::utils::pausable::EnforcedPause),
}

pub trait IERC20Storage {
    fn _get_total_supply(&self) -> U256;
    fn _set_total_supply(&mut self, total_supply: U256);

    fn _get_balance(&self, account: Address) -> U256;
    fn _set_balance(&mut self, account: Address, balance: U256);

    fn _get_allowance(&self, owner: Address, spender: Address) -> U256;
    fn _set_allowance(
        &mut self,
        owner: Address,
        spender: Address,
        allowance: U256,
    );
}

pub trait IERC20Internal: IERC20Storage {
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
    /// [`Error::InvalidSender`] is returned.
    /// * If the `to` address is `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If the `from` address doesn't have enough tokens, then the error
    /// [`Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    fn _transfer(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Error> {
        if from.is_zero() {
            return Err(Error::InvalidSender(ERC20InvalidSender {
                sender: Address::ZERO,
            }));
        }
        if to.is_zero() {
            return Err(Error::InvalidReceiver(ERC20InvalidReceiver {
                receiver: Address::ZERO,
            }));
        }

        self._update(from, to, value)?;

        Ok(())
    }

    /// Transfers a `value` amount of tokens from `from` to `to`,
    /// or alternatively mints (or burns)
    /// if `from` (or `to`) is the zero address.
    ///
    /// All customizations to transfers, mints, and burns
    /// should be done by using this function.
    ///
    /// # Arguments
    ///
    /// * `from` - Owner's address.
    /// * `to` - Recipient's address.
    /// * `value` - Amount to be transferred.
    ///
    /// # Panics
    ///
    /// If `_total_supply` exceeds `U256::MAX`.
    /// It may happen during `mint` operation.
    ///
    /// # Errors
    ///
    /// If the `from` address doesn't have enough tokens, then the error
    /// [`Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    fn _update(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Error> {
        if from.is_zero() {
            // Mint operation. Overflow check required: the rest of the code
            // assumes that `_total_supply` never overflows.
            let total_supply = self
                ._get_total_supply()
                .checked_add(value)
                .expect("Should not exceed `U256::MAX` for `_total_supply`");
            self._set_total_supply(total_supply);
        } else {
            let from_balance = self._get_balance(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance(
                    ERC20InsufficientBalance {
                        sender: from,
                        balance: from_balance,
                        needed: value,
                    },
                ));
            }
            // Overflow not possible:
            // value <= from_balance <= _total_supply.
            self._set_balance(from, from_balance - value);
        }

        if to.is_zero() {
            let total_supply = self._get_total_supply();
            // Overflow not possible:
            // value <= _total_supply or value <= from_balance <= _total_supply.
            self._set_total_supply(total_supply - value);
        } else {
            let balance_to = self._get_balance(to);
            // Overflow not possible:
            // balance + value is at most total_supply, which fits into a U256.
            self._set_balance(to, balance_to + value);
        }

        evm::log(Transfer { from, to, value });

        Ok(())
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
    /// [`Error::InvalidSender`] is returned.
    /// If the `from` address doesn't have enough tokens, then the error
    /// [`Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    fn _burn(&mut self, account: Address, value: U256) -> Result<(), Error> {
        if account == Address::ZERO {
            return Err(Error::InvalidSender(ERC20InvalidSender {
                sender: Address::ZERO,
            }));
        }
        self._update(account, Address::ZERO, value)
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
    fn _spend_allowance(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Error> {
        let current_allowance = self._get_allowance(owner, spender);
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

            self._set_allowance(owner, spender, current_allowance - value);
        }

        Ok(())
    }
}

/// Trait containing ERC20 Interface TODO
pub trait IERC20: IERC20Internal {
    /// Returns the number of tokens in existence.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn total_supply(&self) -> U256 {
        self._get_total_supply()
    }

    /// Returns the number of tokens owned by `account`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - Account to get balance from.
    fn balance_of(&self, account: Address) -> U256 {
        self._get_balance(account)
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
    fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Error> {
        let from = msg::sender();
        self._transfer(from, to, value)?;
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
    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self._get_allowance(owner, spender)
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
    fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, Error> {
        let owner = msg::sender();
        if spender.is_zero() {
            return Err(Error::InvalidSpender(ERC20InvalidSpender {
                spender: Address::ZERO,
            }));
        }

        self._set_allowance(owner, spender, value);
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
    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Error> {
        let spender = msg::sender();
        self._spend_allowance(from, spender, value)?;
        self._transfer(from, to, value)?;
        Ok(true)
    }
}

/// TODO
#[macro_export]
macro_rules! ierc20_storage_impl {
    () => {
        fn _get_total_supply(&self) -> U256 {
            self.erc20._get_total_supply()
        }

        fn _set_total_supply(&mut self, total_supply: U256) {
            self.erc20._set_total_supply(total_supply)
        }

        fn _get_balance(&self, account: Address) -> U256 {
            self.erc20._get_balance(account)
        }

        fn _set_balance(&mut self, account: Address, balance: U256) {
            self.erc20._set_balance(account, balance)
        }

        fn _get_allowance(&self, owner: Address, spender: Address) -> U256 {
            self.erc20._get_allowance(owner, spender)
        }

        fn _set_allowance(
            &mut self,
            owner: Address,
            spender: Address,
            allowance: U256,
        ) {
            self.erc20._set_allowance(owner, spender, allowance)
        }
    };
}
