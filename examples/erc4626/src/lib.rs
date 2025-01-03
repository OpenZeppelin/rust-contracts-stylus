#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{
        extensions::{Erc20Metadata, Erc4626},
        Erc20,
    };
use stylus_sdk::prelude::{entrypoint, public, storage};


#[entrypoint]
#[storage]
struct Erc4626Example {
    #[borrow]
    pub erc20: Erc20,
    #[borrow]
    pub metadata: Erc20Metadata,
    #[borrow]
    pub erc4626: Erc4626,
}

#[public]
#[inherit(Erc20, Erc20Metadata, Erc4626)]
impl Erc4626Example {
    fn max_deposit(&self, _receiver: Address) -> U256 {
        //self.metadata.decimals()
        U256::from(100)
    }
}
