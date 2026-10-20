//! Unit-test environment for OpenZeppelin Stylus contracts.
//!
//! The environment is a small simulated chain built on the Stylus SDK test
//! VM ([`TestVM`]): every deployed contract owns a `TestVM` that holds its
//! storage and mocks, a shared `TestVM` holds balances, code and the block
//! context, and the environment routes calls between contracts through their
//! `#[public]` routers. A call that returns `Err` can be reverted with
//! [`ResultExt`], and cross-contract calls revert what they changed on their
//! own.
//!
//! # Examples
//!
//! ```rust,ignore
//! use testing::prelude::*;
//!
//! #[testing::test]
//! fn transfers(token: Contract<Erc20>, alice: Address, bob: Address) {
//!     token.sender(alice)._mint(alice, U256::ONE).unwrap();
//!     token.sender(alice).transfer(bob, U256::ONE).unwrap();
//!     assert_eq!(token.sender(bob).balance_of(bob), U256::ONE);
//!     token.assert_emitted(&Transfer { from: alice, to: bob, value: U256::ONE });
//!
//!     let err = token.sender(bob).transfer(alice, U256::from(2)).unwrap_revert();
//!     assert!(matches!(err, Error::InsufficientBalance(_)));
//! }
//! ```
//!
//! The macro expands to a `#[test]` function that creates an [`Env`] and
//! builds every parameter from it: `Contract<T>` deploys `T` and `Address`
//! creates an account. Addresses are derived from the parameter names, so
//! they are stable across runs. The macro refers to this crate as `testing`,
//! so import it under that name.
#![deny(rustdoc::broken_intra_doc_links)]

mod account;
mod contract;
mod env;
mod host;
mod result;

pub use account::Account;
use alloy_primitives::{Address, U256};
pub use contract::{Contract, ContractCall};
pub use env::{Env, Log, CONTRACT_CODE};
pub use host::EnvHost;
pub use openzeppelin_stylus_testing_proc::test;
pub use result::ResultExt;
use stylus_sdk::{abi::Router, prelude::*, storage::StorageType};
pub use stylus_sdk::testing::TestVM;

/// Contract types whose `#[public]` methods can be called by other contracts.
///
/// Every `#[storage]` struct with a `#[public] impl` block implements it.
pub trait Routable:
    Router<Self, Storage = Self>
    + StorageType
    + TopLevelStorage
    + ValueDenier
    + ConstructorGuard
    + Sized
    + 'static
{
}

impl<T> Routable for T where
    T: Router<T, Storage = T>
        + StorageType
        + TopLevelStorage
        + ValueDenier
        + ConstructorGuard
        + 'static
{
}

/// Values that `#[testing::test]` builds from the environment.
pub trait FromEnv {
    /// Builds the value from `env`; `name` is the test parameter's name.
    fn from_env(env: &Env, name: &str) -> Self;
}

impl FromEnv for Address {
    fn from_env(env: &Env, name: &str) -> Self {
        env.account(name)
    }
}

impl FromEnv for Env {
    fn from_env(env: &Env, _name: &str) -> Self {
        env.clone()
    }
}

/// Funds an account of the current environment.
pub trait Funding {
    /// Credits `amount` wei to this account.
    fn fund(self, amount: U256);
}

impl Funding for Address {
    fn fund(self, amount: U256) {
        Env::current().fund(self, amount);
    }
}

/// Reads the balance of an account of the current environment.
pub trait Balance {
    /// Returns the balance of this account in wei.
    fn balance(self) -> U256;
}

impl Balance for Address {
    fn balance(self) -> U256 {
        Env::current().balance(self)
    }
}

/// Everything a test module needs.
pub mod prelude {
    pub use alloy_primitives::Address;

    pub use crate::{
        Account, Balance, Contract, Env, Funding, Log, ResultExt, Routable,
        TestVM,
    };
}

/// Support code for the `#[testing::test]` macro. Not a public API.
#[doc(hidden)]
pub mod __private {
    use core::marker::PhantomData;

    use stylus_sdk::storage::StorageType;

    use crate::{Contract, Env, Routable};

    /// Deploys the type `T` for a test parameter.
    ///
    /// `(&Deployer::<T>::new()).deploy(env, name)` resolves to
    /// [`DeployRoutable::deploy`] when `T` has a `#[public]` impl block and
    /// to [`DeployAny::deploy`] otherwise (auto-ref specialization).
    pub struct Deployer<T>(PhantomData<T>);

    impl<T> Deployer<T> {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self(PhantomData)
        }
    }

    pub trait DeployRoutable<T> {
        fn deploy(&self, env: &Env, name: &str) -> Contract<T>;
    }

    impl<T: Routable> DeployRoutable<T> for Deployer<T> {
        fn deploy(&self, env: &Env, name: &str) -> Contract<T> {
            env.deploy::<T>(name)
        }
    }

    pub trait DeployAny<T> {
        fn deploy(&self, env: &Env, name: &str) -> Contract<T>;
    }

    impl<T: StorageType> DeployAny<T> for &Deployer<T> {
        fn deploy(&self, env: &Env, name: &str) -> Contract<T> {
            env.attach::<T>(name)
        }
    }
}
