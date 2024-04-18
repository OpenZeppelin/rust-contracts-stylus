//! Implementation of the ERC-20 token standard.
//!
//! We have followed general OpenZeppelin Contracts guidelines: functions
//! revert instead of returning `false` on failure. This behavior is
//! nonetheless conventional and does not conflict with the expectations of
//! ERC-20 applications.
use alloy_primitives::{Address, U256};

pub mod extensions;

pub mod ierc20;
pub use ierc20::{Error, IERC20Internal, IERC20};
use stylus_sdk::prelude::{external, sol_storage};

/// This macro provides an implementation of the ERC-20 token.
///
/// It adds the `burn` and `burn_from` functions, and expects the token
/// to contain `ERC20 erc20` as a field. See [`crate::ERC20`].
#[macro_export]
macro_rules! erc20_impl {
    () => {
        /// Returns the number of tokens in existence.
        ///
        /// # Arguments
        ///
        /// * `&self` - Read access to the contract's state.
        pub(crate) fn total_supply(&self) -> alloy_primitives::U256 {
            self.erc20.total_supply()
        }

        /// Returns the number of tokens owned by `account`.
        ///
        /// # Arguments
        ///
        /// * `&self` - Read access to the contract's state.
        /// * `account` - Account to get balance from.
        pub(crate) fn balance_of(
            &self,
            account: alloy_primitives::Address,
        ) -> alloy_primitives::U256 {
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
        /// [`Error::InvalidReceiver`] is returned.
        /// * If the caller doesn't have a balance of at least `value`, then the
        /// error [`Error::InsufficientBalance`] is returned.
        ///
        /// # Events
        ///
        /// Emits a [`Transfer`] event.
        pub(crate) fn transfer(
            &mut self,
            to: alloy_primitives::Address,
            value: alloy_primitives::U256,
        ) -> Result<bool, alloc::vec::Vec<u8>> {
            self.erc20.transfer(to, value).map_err(|e| e.into())
        }

        /// Returns the remaining number of tokens that `spender` will be
        /// allowed to spend on behalf of `owner` through `transfer_from`. This
        /// is zero by default.
        ///
        /// This value changes when `approve` or `transfer_from` are called.
        ///
        /// # Arguments
        ///
        /// * `&self` - Read access to the contract's state.
        /// * `owner` - Account that owns the tokens.
        /// * `spender` - Account that will spend the tokens.
        pub(crate) fn allowance(
            &self,
            owner: alloy_primitives::Address,
            spender: alloy_primitives::Address,
        ) -> alloy_primitives::U256 {
            self.erc20.allowance(owner, spender)
        }

        /// Sets a `value` number of tokens as the allowance of `spender` over
        /// the caller's tokens.
        ///
        /// Returns a boolean value indicating whether the operation succeeded.
        ///
        /// WARNING: Beware that changing an allowance with this method brings
        /// the risk that someone may use both the old and the new allowance by
        /// unfortunate transaction ordering. One possible solution to mitigate
        /// this race condition is to first reduce the spender's allowance to 0
        /// and set the desired value afterwards:
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
        pub(crate) fn approve(
            &mut self,
            spender: alloy_primitives::Address,
            value: alloy_primitives::U256,
        ) -> Result<bool, alloc::vec::Vec<u8>> {
            self.erc20.approve(spender, value).map_err(|e| e.into())
        }

        /// Moves a `value` number of tokens from `from` to `to` using the
        /// allowance mechanism. `value` is then deducted from the caller's
        /// allowance.
        ///
        /// Returns a boolean value indicating whether the operation succeeded.
        ///
        /// NOTE: If `value` is the maximum `uint256`, the allowance is not
        /// updated on `transferFrom`. This is semantically equivalent to an
        /// infinite approval.
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
        pub(crate) fn transfer_from(
            &mut self,
            from: alloy_primitives::Address,
            to: alloy_primitives::Address,
            value: alloy_primitives::U256,
        ) -> Result<bool, alloc::vec::Vec<u8>> {
            self.erc20.transfer_from(from, to, value).map_err(|e| e.into())
        }
    };
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

impl crate::erc20::ierc20::IERC20Storage for ERC20 {
    fn _get_total_supply(&self) -> U256 {
        self._total_supply.get()
    }

    fn _set_total_supply(&mut self, total_supply: U256) {
        self._total_supply.set(total_supply)
    }

    fn _get_balance(&self, account: Address) -> U256 {
        self._balances.get(account)
    }

    fn _set_balance(&mut self, account: Address, balance: U256) {
        self._balances.setter(account).set(balance);
    }

    fn _get_allowance(&self, owner: Address, spender: Address) -> U256 {
        self._allowances.get(owner).get(spender)
    }

    fn _set_allowance(
        &mut self,
        owner: Address,
        spender: Address,
        allowance: U256,
    ) {
        self._allowances.setter(owner).insert(spender, allowance);
    }
}
impl crate::erc20::ierc20::IERC20Internal for ERC20 {}

#[external]
impl IERC20 for ERC20 {}

impl ERC20 {}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, Address, U256};
    use stylus_sdk::{
        msg,
        storage::{StorageMap, StorageType, StorageU256},
    };

    use crate::erc20::{
        ierc20::{IERC20Internal, IERC20Storage, IERC20},
        Error, ERC20,
    };

    impl Default for ERC20 {
        fn default() -> Self {
            let root = U256::ZERO;
            ERC20 {
                _balances: unsafe { StorageMap::new(root, 0) },
                _allowances: unsafe {
                    StorageMap::new(root + U256::from(32), 0)
                },
                _total_supply: unsafe {
                    StorageU256::new(root + U256::from(64), 0)
                },
            }
        }
    }

    #[grip::test]
    fn reads_balance(contract: ERC20) {
        let balance = contract.balance_of(Address::ZERO);
        assert_eq!(U256::ZERO, balance);

        let owner = msg::sender();
        let one = U256::from(1);
        contract._set_balance(owner, one);
        let balance = contract.balance_of(owner);
        assert_eq!(one, balance);
    }

    #[grip::test]
    fn update_mint(contract: ERC20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = U256::from(1);

        // Store initial balance & supply
        let initial_balance = contract.balance_of(alice);
        let initial_supply = contract.total_supply();

        // Mint action should work
        let result = contract._update(Address::ZERO, alice, one);
        assert!(result.is_ok());

        // Check updated balance & supply
        assert_eq!(initial_balance + one, contract.balance_of(alice));
        assert_eq!(initial_supply + one, contract.total_supply());
    }

    #[grip::test]
    #[should_panic]
    fn update_mint_errors_arithmetic_overflow(contract: ERC20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = U256::from(1);
        assert_eq!(U256::ZERO, contract.balance_of(alice));
        assert_eq!(U256::ZERO, contract.total_supply());

        // Initialize state for the test case -- Alice's balance as U256::MAX
        contract
            ._update(Address::ZERO, alice, U256::MAX)
            .expect("ERC20::_update should work");
        // Mint action should NOT work -- overflow on `_total_supply`.
        let _result = contract._update(Address::ZERO, alice, one);
    }

    #[grip::test]
    fn update_burn(contract: ERC20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = U256::from(1);
        let two = U256::from(2);

        // Initialize state for the test case -- Alice's balance as `two`
        contract
            ._update(Address::ZERO, alice, two)
            .expect("ERC20::_update should work");

        // Store initial balance & supply
        let initial_balance = contract.balance_of(alice);
        let initial_supply = contract.total_supply();

        // Burn action should work
        let result = contract._update(alice, Address::ZERO, one);
        assert!(result.is_ok());

        // Check updated balance & supply
        assert_eq!(initial_balance - one, contract.balance_of(alice));
        assert_eq!(initial_supply - one, contract.total_supply());
    }

    #[grip::test]
    fn update_burn_errors_insufficient_balance(contract: ERC20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = U256::from(1);
        let two = U256::from(2);

        // Initialize state for the test case -- Alice's balance as `one`
        contract
            ._update(Address::ZERO, alice, one)
            .expect("ERC20::_update should work");

        // Store initial balance & supply
        let initial_balance = contract.balance_of(alice);
        let initial_supply = contract.total_supply();

        // Burn action should NOT work -- `InsufficientBalance`
        let result = contract._update(alice, Address::ZERO, two);
        assert!(matches!(result, Err(Error::InsufficientBalance(_))));

        // Check proper state (before revert)
        assert_eq!(initial_balance, contract.balance_of(alice));
        assert_eq!(initial_supply, contract.total_supply());
    }

    #[grip::test]
    fn update_transfer(contract: ERC20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
        let one = U256::from(1);

        // Initialize state for the test case -- Alice's & Bob's balance as
        // `one`
        contract
            ._update(Address::ZERO, alice, one)
            .expect("ERC20::_update should work");
        contract
            ._update(Address::ZERO, bob, one)
            .expect("ERC20::_update should work");

        // Store initial balance & supply
        let initial_alice_balance = contract.balance_of(alice);
        let initial_bob_balance = contract.balance_of(bob);
        let initial_supply = contract.total_supply();

        // Transfer action should work
        let result = contract._update(alice, bob, one);
        assert!(result.is_ok());

        // Check updated balance & supply
        assert_eq!(initial_alice_balance - one, contract.balance_of(alice));
        assert_eq!(initial_bob_balance + one, contract.balance_of(bob));
        assert_eq!(initial_supply, contract.total_supply());
    }

    #[grip::test]
    fn update_transfer_errors_insufficient_balance(contract: ERC20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
        let one = U256::from(1);

        // Initialize state for the test case -- Alice's & Bob's balance as
        // `one`
        contract
            ._update(Address::ZERO, alice, one)
            .expect("ERC20::_update should work");
        contract
            ._update(Address::ZERO, bob, one)
            .expect("ERC20::_update should work");

        // Store initial balance & supply
        let initial_alice_balance = contract.balance_of(alice);
        let initial_bob_balance = contract.balance_of(bob);
        let initial_supply = contract.total_supply();

        // Transfer action should NOT work -- `InsufficientBalance`
        let result = contract._update(alice, bob, one + one);
        assert!(matches!(result, Err(Error::InsufficientBalance(_))));

        // Check proper state (before revert)
        assert_eq!(initial_alice_balance, contract.balance_of(alice));
        assert_eq!(initial_bob_balance, contract.balance_of(bob));
        assert_eq!(initial_supply, contract.total_supply());
    }

    #[grip::test]
    fn transfers(contract: ERC20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

        // Alice approves `msg::sender`.
        let one = U256::from(1);
        contract._set_allowance(alice, msg::sender(), one);

        // Mint some tokens for Alice.
        let two = U256::from(2);
        contract._update(Address::ZERO, alice, two).unwrap();
        assert_eq!(two, contract.balance_of(alice));

        contract.transfer_from(alice, bob, one).unwrap();

        assert_eq!(one, contract.balance_of(alice));
        assert_eq!(one, contract.balance_of(bob));
    }

    #[grip::test]
    fn transfers_from(contract: ERC20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
        let sender = msg::sender();

        // Alice approves `msg::sender`.
        let one = U256::from(1);
        contract._set_allowance(alice, sender, one);

        // Mint some tokens for Alice.
        let two = U256::from(2);
        contract._update(Address::ZERO, alice, two).unwrap();
        assert_eq!(two, contract.balance_of(alice));

        contract.transfer_from(alice, bob, one).unwrap();

        assert_eq!(one, contract.balance_of(alice));
        assert_eq!(one, contract.balance_of(bob));
        assert_eq!(U256::ZERO, contract.allowance(alice, sender));
    }

    #[grip::test]
    fn transfer_from_errors_when_insufficient_balance(contract: ERC20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

        // Alice approves `msg::sender`.
        let one = U256::from(1);
        contract._set_allowance(alice, msg::sender(), one);
        assert_eq!(U256::ZERO, contract.balance_of(alice));

        let one = U256::from(1);
        let result = contract.transfer_from(alice, bob, one);
        assert!(matches!(result, Err(Error::InsufficientBalance(_))));
    }

    #[grip::test]
    fn transfer_from_errors_when_invalid_sender(contract: ERC20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = U256::from(1);
        contract._set_allowance(Address::ZERO, msg::sender(), one);
        let result = contract.transfer_from(Address::ZERO, alice, one);
        assert!(matches!(result, Err(Error::InvalidSender(_))));
    }

    #[grip::test]
    fn transfer_from_errors_when_invalid_receiver(contract: ERC20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let one = U256::from(1);
        contract._set_allowance(alice, msg::sender(), one);
        let result = contract.transfer_from(alice, Address::ZERO, one);
        assert!(matches!(result, Err(Error::InvalidReceiver(_))));
    }

    #[grip::test]
    fn transfer_from_errors_when_insufficient_allowance(contract: ERC20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

        // Mint some tokens for Alice.
        let one = U256::from(1);
        contract._update(Address::ZERO, alice, one).unwrap();
        assert_eq!(one, contract.balance_of(alice));

        let result = contract.transfer_from(alice, bob, one);
        assert!(matches!(result, Err(Error::InsufficientAllowance(_))));
    }

    #[grip::test]
    fn reads_allowance(contract: ERC20) {
        let owner = msg::sender();
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

        let allowance = contract.allowance(owner, alice);
        assert_eq!(U256::ZERO, allowance);

        let one = U256::from(1);
        contract._set_allowance(owner, alice, one);
        let allowance = contract.allowance(owner, alice);
        assert_eq!(one, allowance);
    }

    #[grip::test]
    fn approves(contract: ERC20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

        // `msg::sender` approves Alice.
        let one = U256::from(1);
        contract.approve(alice, one).unwrap();
        assert_eq!(one, contract._allowances.get(msg::sender()).get(alice));
    }

    #[grip::test]
    fn approve_errors_when_invalid_spender(contract: ERC20) {
        // `msg::sender` approves `Address::ZERO`.
        let one = U256::from(1);
        let result = contract.approve(Address::ZERO, one);
        assert!(matches!(result, Err(Error::InvalidSpender(_))));
    }
}
