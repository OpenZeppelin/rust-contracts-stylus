use alloy_primitives::{Address, U256};
use stylus_sdk::prelude::AccountAccess;

/// Provides additional access to account details of the host environment.
pub trait AccountAccessExt: AccountAccess {
    /// Gets the ETH balance in wei of the current program's account.
    fn contract_balance(&self) -> U256 {
        self.balance(self.contract_address())
    }

    /// Determines if an account has code.
    /// Note that this is insufficient to determine if an address is an [`EOA`].
    /// During contract deployment, an account only gets its code at the very
    /// end, meaning that this method will return `false` while the
    /// constructor executing.
    ///
    /// [`EOA`]: https://ethereum.org/en/developers/docs/accounts/#types-of-account
    fn has_code(&self, account: Address) -> bool {
        self.code_size(account) > 0
    }
}

impl<T: AccountAccess> AccountAccessExt for T {}
