#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{extensions::Erc20Metadata, Erc20};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Erc20Example {
        #[borrow]
        Erc20 erc20;
        #[borrow]
        Erc20Metadata metadata;
    }
}

#[public]
#[inherit(Erc20, Erc20Metadata)]
impl Erc20Example {
    pub fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        self.erc20._mint(account, value)?;
        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::U256;

    use super::Erc20Example;

    #[motsu::test]
    fn dummy_test(_contract: Erc20Example) {
        assert_eq!(U256::ZERO, U256::ZERO)
    }
}
