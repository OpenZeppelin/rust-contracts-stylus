#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;
use core::borrow;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::{
    token::erc20::{
        extensions::{Erc20Metadata, Erc4626, IErc20Metadata, IERC4626},
        Erc20, IErc20,
    },
    utils::{introspection::erc165::IErc165, Pausable},
};
use stylus_sdk::prelude::{entrypoint, public, storage};

#[entrypoint]
#[storage]
struct Erc4262xample {
    #[borrow]
    pub erc20: Erc20,
    #[borrow]
    pub metadata: Erc20Metadata,
    #[borrow]
    pub erc4626: Erc4626,
}

#[public]
#[inherit(Erc20, Erc20Metadata, Erc4626)]
impl Erc4262xample {
    fn max_deposit(&self, _receiver: Address) -> U256 {
        //self.metadata.decimals()
        U256::from(100)
    }
    // Add token minting feature.
}
