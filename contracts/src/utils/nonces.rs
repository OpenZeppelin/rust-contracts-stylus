//! Nonces Contract
//! 
//! Contract module which provides functionalities for tracking nonces for addresses.
//! 
//! Note: Nonce will only increment.

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use stylus_proc::{external, sol_storage, SolidityError};
use stylus_sdk::{evm, msg};


sol! {
    /// The nonce used for an `account` is not the expected current nonce.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error InvalidAccountNonce(address account, uint256 currentNonce);
}

/// A Nonces error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// The nonce used for an `account` is not the expected current nonce.
    InvalidAccountNonce(InvalidAccountNonce),
}

sol_storage! {
    /// State of a Nonces Contract.
    #[cfg_attr(all(test, feature = "std"), derive(motsu::DefaultStorageLayout))]
    pub struct Nonces {
        /// Mapping from address to its nonce.
        mapping(address => uint256) _nonces;
    }
}

#[external]
impl Nonces {
    /// Returns the unuse nonce for the given `account`.
    /// 
    /// # Arguments
    /// 
    /// - `account` - The address for which to return the nonce.
    /// * `&self` - Read access to the contract's state.
    fn nonce(&self, owner: Address) -> U256 {
        self._nonces.get(owner)
    }
    
    /// Consumes a nonce for the given `account`.
    /// 
    /// # Arguments
    /// 
    /// - `account` - The address for which to consume the nonce.
    /// * `&mut self` - Write access to the contract's state.
    /// 
    /// * Returns the current nonce for the given `account` and increments the nonce.
    fn use_nonce(&mut self, owner: Address) -> Result<U256, Error> {
        let nonce = self._nonces.get(owner);
        self._nonces.setter(owner).set(nonce + U256::from(1u32));
        
        Ok(nonce)
    }
    
    /// Same as `use_nonce` but checking that the `nonce` is the next valid for the owner.
    /// 
    /// # Arguments
    /// 
    /// - `account` - The address for which to consume the nonce.
    /// - `nonce` - The nonce to consume.
    /// * `&mut self` - Write access to the contract's state.
    fn use_checked_nonce(&mut self, owner: Address, nonce: U256) -> Result<(), Error> {
        let current_nonce = self._nonces.get(owner);
        
        if nonce != current_nonce {
            return Err(Error::InvalidAccountNonce(InvalidAccountNonce {
                account: owner,
                currentNonce: current_nonce,
            }));
        }
        
        self._nonces.setter(owner).set(nonce + U256::from(1u32));
        
        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{Address, U256, address};
    use crate::utils::nonces::Nonces;
    use alloy_sol_types::sol;
    
    
    #[motsu::test]
    fn test_initiate_nonce(contract: Nonces) {
        let owner = address!("d8da6bf26964af9d7eed9e03e53415d37aa96045");
        
        assert_eq!(contract.nonce(owner), U256::from(0u32));
    }
}