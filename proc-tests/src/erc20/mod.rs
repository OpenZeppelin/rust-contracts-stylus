use alloy_primitives::{Address, U256};
use contracts::{
    erc20::{
        extensions::burnable::IERC20Burnable, IERC20Storage, ERC20, IERC20,
    },
    utils::{
        capped::{Capped, ICapped},
        pausable::{IPausable, Pausable},
    },
};
use erc20_proc::{
    ICapped, IERC20Burnable, IERC20Capped, IERC20Pausable, IERC20Storage,
    IERC20Virtual, IPausable, IERC20,
};
use stylus_sdk::{msg, prelude::sol_storage};

sol_storage! {
    #[derive(IERC20Storage, IERC20, IERC20Virtual, IERC20Burnable, IPausable, ICapped, Default)]
    struct BurnableCappedPausableERC20 {
        CappedPausableERC20 erc20;
    }

    #[derive(IERC20Storage, IERC20, IPausable, IERC20Capped)]
    struct CappedPausableERC20 {
        PausableERC20 erc20;
        Capped capped;
    }

    #[derive(IERC20Storage, IERC20, IERC20Pausable)]
    struct PausableERC20 {
        ERC20 erc20;
        Pausable pausable
    }
}

type TestToken = BurnableCappedPausableERC20;

use alloy_primitives::address;
use contracts::erc20::IERC20Virtual;
use stylus_sdk::storage::{StorageBool, StorageMap, StorageType, StorageU256};

impl Default for PausableERC20 {
    fn default() -> Self {
        let root = U256::ZERO;
        let erc20 = ERC20 {
            _balances: unsafe { StorageMap::new(root, 0) },
            _allowances: unsafe { StorageMap::new(root + U256::from(32), 0) },
            _total_supply: unsafe {
                StorageU256::new(root + U256::from(64), 0)
            },
        };

        let pausable =
            Pausable { _paused: unsafe { StorageBool::new(U256::ZERO, 0) } };

        Self { erc20, pausable }
    }
}

impl Default for CappedPausableERC20 {
    fn default() -> Self {
        let capped =
            Capped { _cap: unsafe { StorageU256::new(U256::from(128), 0) } };
        Self { erc20: PausableERC20::default(), capped }
    }
}

#[grip::test]
fn reads_balance(contract: TestToken) {
    let balance = contract.balance_of(Address::ZERO);
    assert_eq!(U256::ZERO, balance);

    let owner = msg::sender();
    let one = U256::from(1);
    contract._set_balance(owner, one);
    let balance = contract.balance_of(owner);
    assert_eq!(one, balance);
}

#[grip::test]
fn mint(contract: TestToken) {
    let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    let one = U256::from(1);

    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Set cap
    contract.set_cap(U256::from(100)).expect("ICapped::set_cap should work");

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
fn mint_errors_when_paused(contract: TestToken) {
    let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    let one = U256::from(1);

    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Set `Paused` State
    contract.pause().expect("IPaused::pause should work");
    assert_eq!(contract.paused(), true);

    // Store initial balance & supply
    let initial_balance = contract.balance_of(alice);
    let initial_supply = contract.total_supply();

    // Set cap
    contract.set_cap(U256::from(100)).expect("ICapped::set_cap should work");

    // Mint action should not work in `Paused` state
    let result = contract._update(Address::ZERO, alice, one);
    assert!(matches!(
        result,
        Err(contracts::erc20::Error::ERC20PausableError(_))
    ));

    // Check updated balance & supply
    assert_eq!(initial_balance, contract.balance_of(alice));
    assert_eq!(initial_supply, contract.total_supply());
}

#[grip::test]
fn mint_errors_when_cap_exceeded(contract: TestToken) {
    let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    let one = U256::from(1);

    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Store initial balance & supply
    let initial_balance = contract.balance_of(alice);
    let initial_supply = contract.total_supply();

    // Set cap
    let cap = U256::from(100);
    contract.set_cap(cap).expect("ICapped::set_cap should work");

    // Mint action should not work in `Paused` state
    let result = contract._update(Address::ZERO, alice, one + cap);
    assert!(matches!(
        result,
        Err(contracts::erc20::Error::ERC20ExceededCap(_))
    ));

    // Check updated balance & supply
    assert_eq!(initial_balance, contract.balance_of(alice));
    assert_eq!(initial_supply, contract.total_supply());
}

#[grip::test]
fn burn(contract: TestToken) {
    let alice = msg::sender();
    let one = U256::from(1);

    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Set cap
    let cap = U256::from(100);
    contract.set_cap(cap).expect("ICapped::set_cap should work");

    // Initialize state for the test case -- Alice's balance as `cap`
    contract
        ._update(Address::ZERO, alice, cap)
        .expect("IERC20Virtual::_update should work");

    // Store initial balance & supply
    let initial_balance = contract.balance_of(alice);
    let initial_supply = contract.total_supply();

    // Burn action should work
    let result = contract.burn(one);
    assert!(result.is_ok());

    // Check updated balance & supply
    assert_eq!(initial_balance - one, contract.balance_of(alice));
    assert_eq!(initial_supply - one, contract.total_supply());
}

#[grip::test]
fn burn_and_mint(contract: TestToken) {
    let alice = msg::sender();
    let one = U256::from(1);

    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Set cap
    let cap = U256::from(100);
    contract.set_cap(cap).expect("ICapped::set_cap should work");

    // Initialize state for the test case -- Alice's balance as `cap`
    contract
        ._update(Address::ZERO, alice, cap)
        .expect("IERC20Virtual::_update should work");

    // Store initial balance & supply
    let initial_balance = contract.balance_of(alice);
    let initial_supply = contract.total_supply();

    // Burn action should work
    let result = contract.burn(one);
    assert!(result.is_ok());

    // Check updated balance & supply
    assert_eq!(initial_balance - one, contract.balance_of(alice));
    assert_eq!(initial_supply - one, contract.total_supply());

    // Mint burnt token
    let result = contract._update(Address::ZERO, alice, one);
    assert!(result.is_ok());

    // Check updated balance & supply
    assert_eq!(initial_balance, contract.balance_of(alice));
    assert_eq!(initial_supply, contract.total_supply());

    // Try to mint token exceeding cap
    let result = contract._update(Address::ZERO, alice, one);
    assert!(matches!(
        result,
        Err(contracts::erc20::Error::ERC20ExceededCap(_))
    ));

    // Check balance & supply
    assert_eq!(initial_balance, contract.balance_of(alice));
    assert_eq!(initial_supply, contract.total_supply());
}

#[grip::test]
fn burn_errors_when_paused(contract: TestToken) {
    let alice = msg::sender();
    let one = U256::from(1);
    let two = U256::from(2);

    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Set cap
    let cap = U256::from(100);
    contract.set_cap(cap).expect("ICapped::set_cap should work");

    // Initialize state for the test case -- Alice's balance as `two`
    contract
        ._update(Address::ZERO, alice, two)
        .expect("IERC20Virtual::_update should work");

    // Set `Paused` State
    contract.pause().expect("IPausable::pause should work");
    assert_eq!(contract.paused(), true);

    // Store initial balance & supply
    let initial_balance = contract.balance_of(alice);
    let initial_supply = contract.total_supply();

    // Burn action should work
    let result = contract.burn(one);
    assert!(matches!(
        result,
        Err(contracts::erc20::Error::ERC20PausableError(_))
    ));

    // Check updated balance & supply
    assert_eq!(initial_balance, contract.balance_of(alice));
    assert_eq!(initial_supply, contract.total_supply());
}

#[grip::test]
fn transfer(contract: TestToken) {
    let alice = msg::sender();
    let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
    let one = U256::from(1);

    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Set cap
    let cap = U256::from(100);
    contract.set_cap(cap).expect("ICapped::set_cap should work");

    // Initialize state for the test case -- Alice's & Bob's balance as
    // `one`
    contract
        ._update(Address::ZERO, alice, one)
        .expect("IERC20Virtual::_update should work");
    contract
        ._update(Address::ZERO, bob, one)
        .expect("IERC20Virtual::_update should work");

    // Store initial balance & supply
    let initial_alice_balance = contract.balance_of(alice);
    let initial_bob_balance = contract.balance_of(bob);
    let initial_supply = contract.total_supply();

    // Transfer action should work
    let result = contract.transfer(bob, one);
    assert!(result.is_ok());

    // Check updated balance & supply
    assert_eq!(initial_alice_balance - one, contract.balance_of(alice));
    assert_eq!(initial_bob_balance + one, contract.balance_of(bob));
    assert_eq!(initial_supply, contract.total_supply());
}

#[grip::test]
fn transfer_errors_when_paused(contract: TestToken) {
    let alice = msg::sender();
    let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
    let one = U256::from(1);

    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Set cap
    let cap = U256::from(100);
    contract.set_cap(cap).expect("ICapped::set_cap should work");

    // Initialize state for the test case -- Alice's & Bob's balance as
    // `one`
    contract
        ._update(Address::ZERO, alice, one)
        .expect("IERC20Virtual::_update should work");
    contract
        ._update(Address::ZERO, bob, one)
        .expect("IERC20Virtual::_update should work");

    // Set `Paused` State
    contract.pause().expect("IPausable::pause should work");
    assert_eq!(contract.paused(), true);

    // Store initial balance & supply
    let initial_alice_balance = contract.balance_of(alice);
    let initial_bob_balance = contract.balance_of(bob);
    let initial_supply = contract.total_supply();

    // Transfer action should work
    let result = contract.transfer(bob, one);
    assert!(matches!(
        result,
        Err(contracts::erc20::Error::ERC20PausableError(_))
    ));

    // Check updated balance & supply
    assert_eq!(initial_alice_balance, contract.balance_of(alice));
    assert_eq!(initial_bob_balance, contract.balance_of(bob));
    assert_eq!(initial_supply, contract.total_supply());
}

#[grip::test]
fn transfer_from(contract: TestToken) {
    let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Set cap
    let cap = U256::from(100);
    contract.set_cap(cap).expect("ICapped::set_cap should work");

    // Alice approves `msg::sender`.
    let one = U256::from(1);
    contract._set_allowance(alice, msg::sender(), one);

    // Mint some tokens for Alice.
    let two = U256::from(2);
    contract._update(Address::ZERO, alice, two).unwrap();
    assert_eq!(two, contract.balance_of(alice));

    contract
        .transfer_from(alice, bob, one)
        .expect("IERC20::transfer should work");

    assert_eq!(one, contract.balance_of(alice));
    assert_eq!(one, contract.balance_of(bob));
}

#[grip::test]
fn transfers_errors_when_paused(contract: TestToken) {
    let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Set cap
    let cap = U256::from(100);
    contract.set_cap(cap).expect("ICapped::set_cap should work");

    // Alice approves `msg::sender`.
    let one = U256::from(1);
    contract._set_allowance(alice, msg::sender(), one);

    // Mint some tokens for Alice.
    let two = U256::from(2);
    contract._update(Address::ZERO, alice, two).unwrap();
    assert_eq!(two, contract.balance_of(alice));

    // Set `Paused` State
    contract.pause().expect("IPausable::pause should work");
    assert_eq!(contract.paused(), true);

    let result = contract.transfer_from(alice, bob, one);
    assert!(matches!(
        result,
        Err(contracts::erc20::Error::ERC20PausableError(_))
    ));

    assert_eq!(two, contract.balance_of(alice));
    assert_eq!(U256::ZERO, contract.balance_of(bob));
}

#[grip::test]
fn reads_allowance(contract: TestToken) {
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
fn approves(contract: TestToken) {
    let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

    // `msg::sender` approves Alice.
    let one = U256::from(1);
    contract.approve(alice, one).unwrap();
    assert_eq!(one, contract._get_allowance(msg::sender(), alice));
}

#[grip::test]
fn approve_errors_when_invalid_spender(contract: TestToken) {
    // `msg::sender` approves `Address::ZERO`.
    let one = U256::from(1);
    let result = contract.approve(Address::ZERO, one);
    assert!(matches!(result, Err(contracts::erc20::Error::InvalidSpender(_))));
}

#[grip::test]
fn paused_works(contract: TestToken) {
    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Check for `Paused` state
    contract.erc20.erc20.pausable._paused.set(true);
    assert_eq!(contract.paused(), true);
}

#[grip::test]
fn when_not_paused_works(contract: TestToken) {
    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    let result = contract.when_not_paused();
    assert!(result.is_ok());
}

#[grip::test]
fn when_not_paused_errors_when_paused(contract: TestToken) {
    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Pause
    contract.pause().expect("IPausable::pause should work");
    assert_eq!(contract.paused(), true);

    let result = contract.when_not_paused();
    assert!(matches!(
        result,
        Err(contracts::utils::pausable::Error::EnforcedPause(_))
    ));
}

#[grip::test]
fn when_paused_works(contract: TestToken) {
    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    contract.pause().expect("IPausable::pause should work");

    let result = contract.when_paused();
    assert!(result.is_ok());
}

#[grip::test]
fn when_paused_errors_when_not_paused(contract: TestToken) {
    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    let result = contract.when_paused();
    assert!(matches!(
        result,
        Err(contracts::utils::pausable::Error::ExpectedPause(_))
    ));
}

#[grip::test]
fn pause_works(contract: TestToken) {
    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Pause the contract
    contract.pause().expect("IPausable::pause should work");
    assert_eq!(contract.paused(), true);
}

#[grip::test]
fn pause_errors_when_already_paused(contract: TestToken) {
    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Pause for the first time
    contract.pause().expect("IPausable::pause should work");
    assert_eq!(contract.paused(), true);

    // Try to pause the `Paused` contract
    let result = contract.pause();
    assert!(matches!(
        result,
        Err(contracts::utils::pausable::Error::EnforcedPause(_))
    ));
    assert_eq!(contract.paused(), true);
}

#[grip::test]
fn unpause_works(contract: TestToken) {
    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Pause the contract
    contract.pause().expect("IPausable::pause should work");
    assert_eq!(contract.paused(), true);

    // Unpause the `Paused` contract
    contract.unpause().expect("IPausable::unpause should work");
    assert_eq!(contract.paused(), false);
}

#[grip::test]
fn unpause_errors_when_already_unpaused(contract: TestToken) {
    // Check `Unpaused` State
    assert_eq!(contract.paused(), false);

    // Unpause the `Unpaused` contract
    let result = contract.unpause();
    assert!(matches!(
        result,
        Err(contracts::utils::pausable::Error::ExpectedPause(_))
    ));
    assert_eq!(contract.paused(), false);
}

#[grip::test]
fn cap_works(contract: TestToken) {
    // Check `cap` value
    let value = U256::from(2024);
    contract.erc20.capped._cap.set(value);
    assert_eq!(contract.cap(), value);

    let value = U256::from(1);
    contract.erc20.capped._cap.set(value);
    assert_eq!(contract.cap(), value);
}

#[grip::test]
fn set_cap_works(contract: TestToken) {
    let initial_value = U256::from(1);
    contract.erc20.capped._cap.set(initial_value);
    assert_eq!(contract.cap(), initial_value);

    // Set cap value
    let new_value = U256::from(2024);
    contract.set_cap(new_value).expect("ICapped::set_cap should work");
    assert_eq!(contract.cap(), new_value);
}

#[grip::test]
fn set_cap_when_invalid_cap(contract: TestToken) {
    let initial_value = U256::from(1);
    contract.erc20.capped._cap.set(initial_value);
    assert_eq!(contract.cap(), initial_value);

    // Try to set invalid cap value
    let result = contract.set_cap(U256::ZERO);
    assert!(matches!(
        result,
        Err(contracts::utils::capped::Error::InvalidCap(_))
    ));
    assert_eq!(contract.cap(), initial_value);
}
