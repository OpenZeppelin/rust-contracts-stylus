//! Implementation of the ERC-20 token standard.
//!
//! We have followed general `OpenZeppelin` Contracts guidelines: functions
//! revert instead of returning `false` on failure. This behavior is
//! nonetheless conventional and does not conflict with the expectations of
//! [`Erc20`] applications.
use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::sol;
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    call::MethodError,
    evm, msg,
    stylus_proc::{public, sol_storage, SolidityError},
};

use crate::utils::introspection::erc165::{Erc165, IErc165};

pub mod extensions;
pub mod utils;

sol! {
    /// Emitted when `value` tokens are moved from one account (`from`) to
    /// another (`to`).
    ///
    /// Note that `value` may be zero.
    #[allow(missing_docs)]
    event Transfer(address indexed from, address indexed to, uint256 value);
    /// Emitted when the allowance of a `spender` for an `owner` is set by a
    /// call to `approve`. `value` is the new allowance.
    #[allow(missing_docs)]
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
    #[allow(missing_docs)]
    error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
    /// Indicates a failure with the token `sender`. Used in transfers.
    ///
    /// * `sender` - Address whose tokens are being transferred.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20InvalidSender(address sender);
    /// Indicates a failure with the token `receiver`. Used in transfers.
    ///
    /// * `receiver` - Address to which the tokens are being transferred.
    #[derive(Debug)]
    #[allow(missing_docs)]
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
    #[allow(missing_docs)]
    error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
    /// Indicates a failure with the `spender` to be approved. Used in
    /// approvals.
    ///
    /// * `spender` - Address that may be allowed to operate on tokens without
    /// being their owner.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20InvalidSpender(address spender);

    /// Indicates a failure with the `approver` of a token to be approved. Used in approvals.
    /// approver Address initiating an approval operation.
    ///
    /// * `approver` - Address initiating an approval operation.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20InvalidApprover(address approver);

}

/// An [`Erc20`] error defined as described in [ERC-6093].
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
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals. approver Address initiating an approval operation.
    InvalidApprover(ERC20InvalidApprover),
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

sol_storage! {
    /// State of an `Erc20` token.
    pub struct Erc20 {
        /// Maps users to balances.
        mapping(address => uint256) _balances;
        /// Maps users to a mapping of each spender's allowance.
        mapping(address => mapping(address => uint256)) _allowances;
        /// The total supply of the token.
        uint256 _total_supply;
    }
}

/// Required interface of an [`Erc20`] compliant contract.
#[interface_id]
pub trait IErc20 {
    /// The error type associated to this ERC-20 trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the number of tokens in existence.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn total_supply(&self) -> U256;

    /// Returns the number of tokens owned by `account`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - Account to get balance from.
    fn balance_of(&self, account: Address) -> U256;

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
    fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error>;

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
    fn allowance(&self, owner: Address, spender: Address) -> U256;

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
    ) -> Result<bool, Self::Error>;

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
    ) -> Result<bool, Self::Error>;
}

#[public]
impl IErc20 for Erc20 {
    type Error = Error;

    fn total_supply(&self) -> U256 {
        self._total_supply.get()
    }

    fn balance_of(&self, account: Address) -> U256 {
        self._balances.get(account)
    }

    fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        let from = msg::sender();
        self._transfer(from, to, value)?;
        Ok(true)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self._allowances.get(owner).get(spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        let owner = msg::sender();
        self._approve(owner, spender, value, true)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        let spender = msg::sender();
        self._spend_allowance(from, spender, value)?;
        self._transfer(from, to, value)?;
        Ok(true)
    }
}

impl IErc165 for Erc20 {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IErc20>::INTERFACE_ID == u32::from_be_bytes(*interface_id)
            || Erc165::supports_interface(interface_id)
    }
}

impl Erc20 {
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
    ) -> Result<bool, Error> {
        if owner.is_zero() {
            return Err(Error::InvalidApprover(ERC20InvalidApprover {
                approver: Address::ZERO,
            }));
        }

        if spender.is_zero() {
            return Err(Error::InvalidSpender(ERC20InvalidSpender {
                spender: Address::ZERO,
            }));
        }

        self._allowances.setter(owner).insert(spender, value);
        if emit_event {
            evm::log(Approval { owner, spender, value });
        }
        Ok(true)
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
    ) -> Result<(), Error> {
        if account.is_zero() {
            return Err(Error::InvalidReceiver(ERC20InvalidReceiver {
                receiver: Address::ZERO,
            }));
        }
        self._update(Address::ZERO, account, value)
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
    ) -> Result<(), Error> {
        if from.is_zero() {
            // Mint operation. Overflow check required: the rest of the code
            // assumes that `_total_supply` never overflows.
            let total_supply = self
                .total_supply()
                .checked_add(value)
                .expect("should not exceed `U256::MAX` for `_total_supply`");
            self._total_supply.set(total_supply);
        } else {
            let from_balance = self._balances.get(from);
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
            // `value` <= `from_balance` <= `_total_supply`.
            self._balances.setter(from).set(from_balance - value);
        }

        if to.is_zero() {
            let total_supply = self.total_supply();
            // Overflow not possible:
            // `value` <= `_total_supply` or
            // `value` <= `from_balance` <= `_total_supply`.
            self._total_supply.set(total_supply - value);
        } else {
            let balance_to = self._balances.get(to);
            // Overflow not possible:
            // `balance_to` + `value` is at most `total_supply`,
            // which fits into a `U256`.
            self._balances.setter(to).set(balance_to + value);
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
    pub fn _burn(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Error> {
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
    pub fn _spend_allowance(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Error> {
        let current_allowance = self.allowance(owner, spender);
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

            self._approve(owner, spender, current_allowance - value, false)?;
        }

        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, uint, Address, U256};
    use stylus_sdk::msg;

    use super::{Erc20, Error, IErc20};
    use crate::utils::introspection::erc165::IErc165;

    #[motsu::test]
    fn reads_balance(contract: Erc20) {
        let balance = contract.balance_of(Address::ZERO);
        assert_eq!(U256::ZERO, balance);

        let owner = msg::sender();
        let one = uint!(1_U256);
        contract._balances.setter(owner).set(one);
        let balance = contract.balance_of(owner);
        assert_eq!(one, balance);
    }

    #[motsu::test]
    fn update_mint(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = uint!(1_U256);

        // Store initial balance & supply.
        let initial_balance = contract.balance_of(alice);
        let initial_supply = contract.total_supply();

        // Mint action should work.
        let result = contract._update(Address::ZERO, alice, one);
        assert!(result.is_ok());

        // Check updated balance & supply.
        assert_eq!(initial_balance + one, contract.balance_of(alice));
        assert_eq!(initial_supply + one, contract.total_supply());
    }

    #[motsu::test]
    #[should_panic = "should not exceed `U256::MAX` for `_total_supply`"]
    fn update_mint_errors_arithmetic_overflow(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = uint!(1_U256);
        assert_eq!(U256::ZERO, contract.balance_of(alice));
        assert_eq!(U256::ZERO, contract.total_supply());

        // Initialize state for the test case:
        // Alice's balance as `U256::MAX`.
        contract
            ._update(Address::ZERO, alice, U256::MAX)
            .expect("should mint tokens");
        // Mint action should NOT work:
        // overflow on `_total_supply`.
        let _result = contract._update(Address::ZERO, alice, one);
    }

    #[motsu::test]
    fn mint_works(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = uint!(1_U256);

        // Store initial balance & supply.
        let initial_balance = contract.balance_of(alice);
        let initial_supply = contract.total_supply();

        // Mint action should work.
        let result = contract._mint(alice, one);
        assert!(result.is_ok());

        // Check updated balance & supply.
        assert_eq!(initial_balance + one, contract.balance_of(alice));
        assert_eq!(initial_supply + one, contract.total_supply());
    }

    #[motsu::test]
    fn mint_errors_invalid_receiver(contract: Erc20) {
        let receiver = Address::ZERO;
        let one = uint!(1_U256);

        // Store initial balance & supply.
        let initial_balance = contract.balance_of(receiver);
        let initial_supply = contract.total_supply();

        // Mint action should work.
        let result = contract._mint(receiver, one);
        assert!(matches!(result, Err(Error::InvalidReceiver(_))));

        // Check updated balance & supply.
        assert_eq!(initial_balance, contract.balance_of(receiver));
        assert_eq!(initial_supply, contract.total_supply());
    }

    #[motsu::test]
    #[should_panic = "should not exceed `U256::MAX` for `_total_supply`"]
    fn mint_errors_arithmetic_overflow(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = uint!(1_U256);
        assert_eq!(U256::ZERO, contract.balance_of(alice));
        assert_eq!(U256::ZERO, contract.total_supply());

        // Initialize state for the test case:
        // Alice's balance as `U256::MAX`.
        contract
            ._update(Address::ZERO, alice, U256::MAX)
            .expect("should mint tokens");
        // Mint action should NOT work -- overflow on `_total_supply`.
        let _result = contract._mint(alice, one);
    }

    #[motsu::test]
    fn update_burn(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = uint!(1_U256);
        let two = uint!(2_U256);

        // Initialize state for the test case:
        // Alice's balance as `two`.
        contract
            ._update(Address::ZERO, alice, two)
            .expect("should mint tokens");

        // Store initial balance & supply.
        let initial_balance = contract.balance_of(alice);
        let initial_supply = contract.total_supply();

        // Burn action should work.
        let result = contract._update(alice, Address::ZERO, one);
        assert!(result.is_ok());

        // Check updated balance & supply.
        assert_eq!(initial_balance - one, contract.balance_of(alice));
        assert_eq!(initial_supply - one, contract.total_supply());
    }

    #[motsu::test]
    fn update_burn_errors_insufficient_balance(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = uint!(1_U256);
        let two = uint!(2_U256);

        // Initialize state for the test case:
        // Alice's balance as `one`.
        contract
            ._update(Address::ZERO, alice, one)
            .expect("should mint tokens");

        // Store initial balance & supply.
        let initial_balance = contract.balance_of(alice);
        let initial_supply = contract.total_supply();

        // Burn action should NOT work - `InsufficientBalance`.
        let result = contract._update(alice, Address::ZERO, two);
        assert!(matches!(result, Err(Error::InsufficientBalance(_))));

        // Check proper state (before revert).
        assert_eq!(initial_balance, contract.balance_of(alice));
        assert_eq!(initial_supply, contract.total_supply());
    }

    #[motsu::test]
    fn update_transfer(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
        let one = uint!(1_U256);

        // Initialize state for the test case:
        //  Alice's & Bob's balance as `one`.
        contract
            ._update(Address::ZERO, alice, one)
            .expect("should mint tokens");
        contract._update(Address::ZERO, bob, one).expect("should mint tokens");

        // Store initial balance & supply.
        let initial_alice_balance = contract.balance_of(alice);
        let initial_bob_balance = contract.balance_of(bob);
        let initial_supply = contract.total_supply();

        // Transfer action should work.
        let result = contract._update(alice, bob, one);
        assert!(result.is_ok());

        // Check updated balance & supply.
        assert_eq!(initial_alice_balance - one, contract.balance_of(alice));
        assert_eq!(initial_bob_balance + one, contract.balance_of(bob));
        assert_eq!(initial_supply, contract.total_supply());
    }

    #[motsu::test]
    fn update_transfer_errors_insufficient_balance(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
        let one = uint!(1_U256);

        // Initialize state for the test case:
        // Alice's & Bob's balance as `one`.
        contract
            ._update(Address::ZERO, alice, one)
            .expect("should mint tokens");
        contract._update(Address::ZERO, bob, one).expect("should mint tokens");

        // Store initial balance & supply.
        let initial_alice_balance = contract.balance_of(alice);
        let initial_bob_balance = contract.balance_of(bob);
        let initial_supply = contract.total_supply();

        // Transfer action should NOT work - `InsufficientBalance`.
        let result = contract._update(alice, bob, one + one);
        assert!(matches!(result, Err(Error::InsufficientBalance(_))));

        // Check proper state (before revert).
        assert_eq!(initial_alice_balance, contract.balance_of(alice));
        assert_eq!(initial_bob_balance, contract.balance_of(bob));
        assert_eq!(initial_supply, contract.total_supply());
    }

    #[motsu::test]
    fn transfers(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

        // Alice approves `msg::sender`.
        let one = uint!(1_U256);
        contract._allowances.setter(alice).setter(msg::sender()).set(one);

        // Mint some tokens for Alice.
        let two = uint!(2_U256);
        contract._update(Address::ZERO, alice, two).unwrap();
        assert_eq!(two, contract.balance_of(alice));

        contract.transfer_from(alice, bob, one).unwrap();

        assert_eq!(one, contract.balance_of(alice));
        assert_eq!(one, contract.balance_of(bob));
    }

    #[motsu::test]
    fn transfers_from(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
        let sender = msg::sender();

        // Alice approves `msg::sender`.
        let one = uint!(1_U256);
        contract._allowances.setter(alice).setter(sender).set(one);

        // Mint some tokens for Alice.
        let two = uint!(2_U256);
        contract._update(Address::ZERO, alice, two).unwrap();
        assert_eq!(two, contract.balance_of(alice));

        contract.transfer_from(alice, bob, one).unwrap();

        assert_eq!(one, contract.balance_of(alice));
        assert_eq!(one, contract.balance_of(bob));
        assert_eq!(U256::ZERO, contract.allowance(alice, sender));
    }

    #[motsu::test]
    fn transfer_from_errors_when_insufficient_balance(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

        // Alice approves `msg::sender`.
        let one = uint!(1_U256);
        contract._allowances.setter(alice).setter(msg::sender()).set(one);
        assert_eq!(U256::ZERO, contract.balance_of(alice));

        let one = uint!(1_U256);
        let result = contract.transfer_from(alice, bob, one);
        assert!(matches!(result, Err(Error::InsufficientBalance(_))));
    }

    #[motsu::test]
    fn transfer_from_errors_when_invalid_approver(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = uint!(1_U256);
        contract
            ._allowances
            .setter(Address::ZERO)
            .setter(msg::sender())
            .set(one);
        let result = contract.transfer_from(Address::ZERO, alice, one);
        assert!(matches!(result, Err(Error::InvalidApprover(_))));
    }

    #[motsu::test]
    fn transfer_from_errors_when_invalid_receiver(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = uint!(1_U256);
        contract._allowances.setter(alice).setter(msg::sender()).set(one);
        let result = contract.transfer_from(alice, Address::ZERO, one);
        assert!(matches!(result, Err(Error::InvalidReceiver(_))));
    }

    #[motsu::test]
    fn transfer_from_errors_when_insufficient_allowance(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

        // Mint some tokens for Alice.
        let one = uint!(1_U256);
        contract._update(Address::ZERO, alice, one).unwrap();
        assert_eq!(one, contract.balance_of(alice));

        let result = contract.transfer_from(alice, bob, one);
        assert!(matches!(result, Err(Error::InsufficientAllowance(_))));
    }

    #[motsu::test]
    fn reads_allowance(contract: Erc20) {
        let owner = msg::sender();
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

        let allowance = contract.allowance(owner, alice);
        assert_eq!(U256::ZERO, allowance);

        let one = uint!(1_U256);
        contract._allowances.setter(owner).setter(alice).set(one);
        let allowance = contract.allowance(owner, alice);
        assert_eq!(one, allowance);
    }

    #[motsu::test]
    fn approves(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

        // `msg::sender` approves Alice.
        let one = uint!(1_U256);
        contract.approve(alice, one).unwrap();
        assert_eq!(one, contract._allowances.get(msg::sender()).get(alice));
    }

    #[motsu::test]
    fn approve_errors_when_invalid_spender(contract: Erc20) {
        // `msg::sender` approves `Address::ZERO`.
        let one = uint!(1_U256);
        let result = contract.approve(Address::ZERO, one);
        assert!(matches!(result, Err(Error::InvalidSpender(_))));
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc20 as IErc20>::INTERFACE_ID;
        let expected = 0x36372b07;
        assert_eq!(actual, expected);

        let actual = <Erc20 as IErc165>::INTERFACE_ID;
        let expected = 0x01ffc9a7;
        assert_eq!(actual, expected);
    }
}
