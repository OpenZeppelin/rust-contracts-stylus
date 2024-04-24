//! ERC20 Pausable extension TODO
use crate::{
    erc20::{IERC20Virtual, IERC20},
    utils::pausable::IPausable,
};

/// This macro provides an implementation of the ERC-20 Pausable extension.
///
/// It adds the `cap` and `set_cap` functions, and expects the token
/// to contain `erc20` as a field that implements IERC20Capped trait.
#[macro_export]
macro_rules! erc20_pausable_impl {
    () => {
        pub(crate) fn paused(&self) -> bool {
            self.erc20.paused()
        }

        pub(crate) fn pause(&mut self) -> Result<(), alloc::vec::Vec<u8>> {
            self.erc20.pause().map_err(|e| e.into())
        }

        pub(crate) fn unpause(&mut self) -> Result<(), alloc::vec::Vec<u8>> {
            self.erc20.unpause().map_err(|e| e.into())
        }

        pub(crate) fn when_not_paused(
            &self,
        ) -> Result<(), alloc::vec::Vec<u8>> {
            self.erc20.when_not_paused().map_err(|e| e.into())
        }

        pub(crate) fn when_paused(&self) -> Result<(), alloc::vec::Vec<u8>> {
            self.erc20.when_paused().map_err(|e| e.into())
        }
    };
}

/// TODO docs
pub trait IERC20Pausable: IERC20Virtual + IERC20 + IPausable {}

#[cfg(all(test, feature = "std"))]
mod tests {
    // use alloy_primitives::{address, Address, U256};
    // use stylus_sdk::msg;

    // use crate::{
    //     erc20::{
    //         self,
    //         extensions::pausable::ERC20Pausable,
    //         ierc20::{IERC20Storage, IERC20Virtual, IERC20},
    //         Error as ERC20Error, ERC20,
    //     },
    //     utils::pausable::{self, Error as PausableError, IPausable, Pausable},
    // };
    // impl Default for TestToken {
    //     fn default() -> Self {
    //         Self { erc20: ERC20::default(), pausable: Pausable::default() }
    //     }
    // }

    // type TestToken = ERC20Pausable<ERC20, Pausable>;
    // #[grip::test]
    // fn reads_balance(contract: TestToken) {
    //     let balance = contract.balance_of(Address::ZERO);
    //     assert_eq!(U256::ZERO, balance);

    //     let owner = msg::sender();
    //     let one = U256::from(1);
    //     contract._set_balance(owner, one);
    //     let balance = contract.balance_of(owner);
    //     assert_eq!(one, balance);
    // }

    // #[grip::test]
    // fn update_mint(contract: TestToken) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    //     let one = U256::from(1);

    //     // Set `Unpaused` State
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);

    //     // Store initial balance & supply
    //     let initial_balance = contract.balance_of(alice);
    //     let initial_supply = contract.total_supply();

    //     // Mint action should work
    //     let result = contract._update(Address::ZERO, alice, one);
    //     assert!(result.is_ok());

    //     // Check updated balance & supply
    //     assert_eq!(initial_balance + one, contract.balance_of(alice));
    //     assert_eq!(initial_supply + one, contract.total_supply());
    // }

    // #[grip::test]
    // fn update_mint_errors_when_paused(contract: TestToken) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    //     let one = U256::from(1);

    //     // Set `Paused` State
    //     contract.pausable._paused.set(true);
    //     assert_eq!(contract.paused(), true);

    //     // Store initial balance & supply
    //     let initial_balance = contract.balance_of(alice);
    //     let initial_supply = contract.total_supply();

    //     // Mint action should not work in `Paused` state
    //     let result = contract._update(Address::ZERO, alice, one);
    //     assert!(matches!(result, Err(ERC20Error::PausableError(_))));

    //     // Check updated balance & supply
    //     assert_eq!(initial_balance, contract.balance_of(alice));
    //     assert_eq!(initial_supply, contract.total_supply());
    // }

    // #[grip::test]
    // fn update_burn(contract: TestToken) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    //     let one = U256::from(1);
    //     let two = U256::from(2);

    //     // Set `Unpaused` State
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);

    //     // Initialize state for the test case -- Alice's balance as `two`
    //     contract
    //         ._update(Address::ZERO, alice, two)
    //         .expect("ERC20::_update should work");

    //     // Store initial balance & supply
    //     let initial_balance = contract.balance_of(alice);
    //     let initial_supply = contract.total_supply();

    //     // Burn action should work
    //     let result = contract._update(alice, Address::ZERO, one);
    //     assert!(result.is_ok());

    //     // Check updated balance & supply
    //     assert_eq!(initial_balance - one, contract.balance_of(alice));
    //     assert_eq!(initial_supply - one, contract.total_supply());
    // }

    // #[grip::test]
    // fn update_burn_errors_when_paused(contract: TestToken) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    //     let one = U256::from(1);
    //     let two = U256::from(2);

    //     // Set `Unpaused` State
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);
    //     // Initialize state for the test case -- Alice's balance as `two`
    //     contract
    //         ._update(Address::ZERO, alice, two)
    //         .expect("ERC20::_update should work");

    //     // Set `Paused` State
    //     contract.pausable._paused.set(true);
    //     assert_eq!(contract.paused(), true);

    //     // Store initial balance & supply
    //     let initial_balance = contract.balance_of(alice);
    //     let initial_supply = contract.total_supply();

    //     // Burn action should work
    //     let result = contract._update(alice, Address::ZERO, one);
    //     assert!(matches!(result, Err(ERC20Error::PausableError(_))));

    //     // Check updated balance & supply
    //     assert_eq!(initial_balance, contract.balance_of(alice));
    //     assert_eq!(initial_supply, contract.total_supply());
    // }

    // #[grip::test]
    // fn update_transfer(contract: TestToken) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    //     let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
    //     let one = U256::from(1);

    //     // Set `Unpaused` State
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);

    //     // Initialize state for the test case -- Alice's & Bob's balance as
    //     // `one`
    //     contract
    //         ._update(Address::ZERO, alice, one)
    //         .expect("ERC20::_update should work");
    //     contract
    //         ._update(Address::ZERO, bob, one)
    //         .expect("ERC20::_update should work");

    //     // Store initial balance & supply
    //     let initial_alice_balance = contract.balance_of(alice);
    //     let initial_bob_balance = contract.balance_of(bob);
    //     let initial_supply = contract.total_supply();

    //     // Transfer action should work
    //     let result = contract._update(alice, bob, one);
    //     assert!(result.is_ok());

    //     // Check updated balance & supply
    //     assert_eq!(initial_alice_balance - one, contract.balance_of(alice));
    //     assert_eq!(initial_bob_balance + one, contract.balance_of(bob));
    //     assert_eq!(initial_supply, contract.total_supply());
    // }

    // #[grip::test]
    // fn update_transfer_errors_when_paused(contract: TestToken) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    //     let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
    //     let one = U256::from(1);

    //     // Set `Unpaused` State
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);

    //     // Initialize state for the test case -- Alice's & Bob's balance as
    //     // `one`
    //     contract
    //         ._update(Address::ZERO, alice, one)
    //         .expect("ERC20::_update should work");
    //     contract
    //         ._update(Address::ZERO, bob, one)
    //         .expect("ERC20::_update should work");

    //     // Set `Paused` State
    //     contract.pausable._paused.set(true);
    //     assert_eq!(contract.paused(), true);

    //     // Store initial balance & supply
    //     let initial_alice_balance = contract.balance_of(alice);
    //     let initial_bob_balance = contract.balance_of(bob);
    //     let initial_supply = contract.total_supply();

    //     // Transfer action should work
    //     let result = contract._update(alice, bob, one);
    //     assert!(matches!(result, Err(ERC20Error::PausableError(_))));

    //     // Check updated balance & supply
    //     assert_eq!(initial_alice_balance, contract.balance_of(alice));
    //     assert_eq!(initial_bob_balance, contract.balance_of(bob));
    //     assert_eq!(initial_supply, contract.total_supply());
    // }

    // #[grip::test]
    // fn transfers(contract: TestToken) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    //     let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

    //     // Set `Unpaused` State
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);

    //     // Alice approves `msg::sender`.
    //     let one = U256::from(1);
    //     contract._set_allowance(alice, msg::sender(), one);

    //     // Mint some tokens for Alice.
    //     let two = U256::from(2);
    //     contract._update(Address::ZERO, alice, two).unwrap();
    //     assert_eq!(two, contract.balance_of(alice));

    //     contract
    //         .transfer_from(alice, bob, one)
    //         .expect("ERC20::transfer should work");

    //     assert_eq!(one, contract.balance_of(alice));
    //     assert_eq!(one, contract.balance_of(bob));
    // }

    // #[grip::test]
    // fn transfers_errors_when_paused(contract: TestToken) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    //     let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

    //     // Set `Unpaused` State
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);

    //     // Alice approves `msg::sender`.
    //     let one = U256::from(1);
    //     contract._set_allowance(alice, msg::sender(), one);

    //     // Mint some tokens for Alice.
    //     let two = U256::from(2);
    //     contract._update(Address::ZERO, alice, two).unwrap();
    //     assert_eq!(two, contract.balance_of(alice));

    //     // Set `Paused` State
    //     contract.pausable._paused.set(true);
    //     assert_eq!(contract.paused(), true);

    //     let result = contract.transfer_from(alice, bob, one);
    //     assert!(matches!(result, Err(ERC20Error::PausableError(_))));

    //     assert_eq!(two, contract.balance_of(alice));
    //     assert_eq!(U256::ZERO, contract.balance_of(bob));
    // }

    // #[grip::test]
    // fn transfers_from(contract: TestToken) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    //     let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
    //     let sender = msg::sender();

    //     // Set `Unpaused` State
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);

    //     // Alice approves `msg::sender`.
    //     let one = U256::from(1);
    //     contract._set_allowance(alice, sender, one);

    //     // Mint some tokens for Alice.
    //     let two = U256::from(2);
    //     contract._update(Address::ZERO, alice, two).unwrap();
    //     assert_eq!(two, contract.balance_of(alice));

    //     contract.transfer_from(alice, bob, one).unwrap();

    //     assert_eq!(one, contract.balance_of(alice));
    //     assert_eq!(one, contract.balance_of(bob));
    //     assert_eq!(U256::ZERO, contract.allowance(alice, sender));
    // }

    // #[grip::test]
    // fn transfers_from_errors_when_paused(contract: TestToken) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    //     let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
    //     let sender = msg::sender();

    //     // Set `Unpaused` State
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);

    //     // Alice approves `msg::sender`.
    //     let one = U256::from(1);
    //     contract._set_allowance(alice, sender, one);

    //     // Mint some tokens for Alice.
    //     let two = U256::from(2);
    //     contract._update(Address::ZERO, alice, two).unwrap();
    //     assert_eq!(two, contract.balance_of(alice));

    //     // Set `Paused` State
    //     contract.pausable._paused.set(true);
    //     assert_eq!(contract.paused(), true);

    //     let result = contract.transfer_from(alice, bob, one);
    //     assert!(matches!(result, Err(ERC20Error::PausableError(_))));

    //     assert_eq!(two, contract.balance_of(alice));
    //     assert_eq!(U256::ZERO, contract.balance_of(bob));
    // }

    // #[grip::test]
    // fn reads_allowance(contract: TestToken) {
    //     let owner = msg::sender();
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

    //     let allowance = contract.allowance(owner, alice);
    //     assert_eq!(U256::ZERO, allowance);

    //     let one = U256::from(1);
    //     contract._set_allowance(owner, alice, one);
    //     let allowance = contract.allowance(owner, alice);
    //     assert_eq!(one, allowance);
    // }

    // #[grip::test]
    // fn approves(contract: TestToken) {
    //     let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

    //     // `msg::sender` approves Alice.
    //     let one = U256::from(1);
    //     contract.approve(alice, one).unwrap();
    //     assert_eq!(one, contract._get_allowance(msg::sender(), alice));
    // }

    // #[grip::test]
    // fn approve_errors_when_invalid_spender(contract: TestToken) {
    //     // `msg::sender` approves `Address::ZERO`.
    //     let one = U256::from(1);
    //     let result = contract.approve(Address::ZERO, one);
    //     assert!(matches!(result, Err(ERC20Error::InvalidSpender(_))));
    // }

    // #[grip::test]
    // fn paused_works(contract: TestToken) {
    //     // Check for unpaused
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);
    //     // Check for paused
    //     contract.pausable._paused.set(true);
    //     assert_eq!(contract.paused(), true);
    // }

    // #[grip::test]
    // fn when_not_paused_works(contract: TestToken) {
    //     // Check for unpaused
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);

    //     let result = contract.when_not_paused();
    //     assert!(result.is_ok());
    // }

    // #[grip::test]
    // fn when_not_paused_errors_when_paused(contract: TestToken) {
    //     // Check for paused
    //     contract.pausable._paused.set(true);
    //     assert_eq!(contract.paused(), true);

    //     let result = contract.when_not_paused();
    //     assert!(matches!(result, Err(PausableError::EnforcedPause(_))));
    // }

    // #[grip::test]
    // fn when_paused_works(contract: TestToken) {
    //     // Check for unpaused
    //     contract.pausable._paused.set(true);
    //     assert_eq!(contract.paused(), true);

    //     let result = contract.when_paused();
    //     assert!(result.is_ok());
    // }

    // #[grip::test]
    // fn when_paused_errors_when_not_paused(contract: TestToken) {
    //     // Check for paused
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);

    //     let result = contract.when_paused();
    //     assert!(matches!(result, Err(PausableError::ExpectedPause(_))));
    // }

    // #[grip::test]
    // fn pause_works(contract: TestToken) {
    //     // Check for unpaused
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);

    //     // Pause the contract
    //     contract.pause().expect("Pause action must work in unpaused state");
    //     assert_eq!(contract.paused(), true);
    // }

    // #[grip::test]
    // fn pause_errors_when_already_paused(contract: TestToken) {
    //     // Check for paused
    //     contract.pausable._paused.set(true);
    //     assert_eq!(contract.paused(), true);

    //     // Pause the paused contract
    //     let result = contract.pause();
    //     assert!(matches!(result, Err(PausableError::EnforcedPause(_))));
    //     assert_eq!(contract.paused(), true);
    // }

    // #[grip::test]
    // fn unpause_works(contract: TestToken) {
    //     // Check for paused
    //     contract.pausable._paused.set(true);
    //     assert_eq!(contract.paused(), true);

    //     // Unpause the paused contract
    //     contract.unpause().expect("Unpause action must work in paused
    // state");     assert_eq!(contract.paused(), false);
    // }

    // #[grip::test]
    // fn unpause_errors_when_already_unpaused(contract: TestToken) {
    //     // Check for unpaused
    //     contract.pausable._paused.set(false);
    //     assert_eq!(contract.paused(), false);

    //     // Unpause the unpaused contract
    //     let result = contract.unpause();
    //     assert!(matches!(result, Err(PausableError::ExpectedPause(_))));
    //     assert_eq!(contract.paused(), false);
    // }
}
