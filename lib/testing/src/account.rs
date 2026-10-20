//! Test accounts that can sign.
use alloy_primitives::{keccak256, Address, B256};
use alloy_signer_local::PrivateKeySigner;

use crate::{Env, FromEnv};

/// A test account with a private key.
///
/// Use it where a test needs signatures, for example for ERC-2612 permits.
/// It converts into an [`Address`], so it can be passed to
/// [`Contract::sender`](crate::Contract::sender) directly.
#[derive(Clone, Debug)]
pub struct Account {
    signer: PrivateKeySigner,
}

impl Account {
    /// Creates the account whose private key is derived from `seed`.
    #[must_use]
    pub fn from_seed(seed: &str) -> Self {
        let key: B256 =
            keccak256(format!("openzeppelin-stylus-testing:key:{seed}"));
        let signer = PrivateKeySigner::from_bytes(&key)
            .expect("a keccak hash should be a valid secp256k1 key");
        Self { signer }
    }

    /// Creates an account with a random private key.
    #[must_use]
    pub fn random() -> Self {
        Self { signer: PrivateKeySigner::random() }
    }

    /// The address of this account.
    #[must_use]
    pub fn address(&self) -> Address {
        self.signer.address()
    }

    /// The signer of this account.
    #[must_use]
    pub fn signer(&self) -> PrivateKeySigner {
        self.signer.clone()
    }
}

impl From<Account> for Address {
    fn from(account: Account) -> Self {
        account.address()
    }
}

impl From<&Account> for Address {
    fn from(account: &Account) -> Self {
        account.address()
    }
}

impl FromEnv for Account {
    fn from_env(env: &Env, name: &str) -> Self {
        let account = Self::from_seed(name);
        env.name_address(account.address(), name);
        account
    }
}
