#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{
    extensions::IERC3156FlashLender, Erc20, IErc20,
};
use stylus_sdk::{
    abi::Bytes,
    prelude::{entrypoint, public, sol_storage},
};

sol_storage! {
    #[entrypoint]
    struct Erc20FlashMintExample {
        #[borrow]
        Erc20 erc20;
    }
}

#[public]
#[inherit(Erc20)]
impl Erc20FlashMintExample {
    fn max_flash_loan(&self, token: Address) -> U256 {
        self.erc20.max_flash_loan(token)
    }

    fn flash_fee(&self, token: Address, amount: U256) -> Result<U256, Vec<u8>> {
        self.
        //self.erc20._flash_fee(token, value);
        self.erc20.flash_fee(token, amount).map_err(|e| e.into())
    }

    fn flash_loan(
        &mut self,
        receiver: Address,
        token: Address,
        value: U256,
        data: Bytes,
    ) -> Result<bool, Vec<u8>> {
        self.erc20
            .flash_loan(receiver, token, value, data)
            .map_err(|e| e.into())
    }
}

impl Erc20FlashMintExample {
    pub fn _flash_fee(&self, token: Address, value: U256) -> U256 {
        let _ = token;
        let _ = value;
        // if self._flash_fee_amount.is_zero() {
        return U256::MIN;
        //}
        //self._flash_fee_amount.get()
    }

    /// Returns the address of the receiver contract that will receive the flash
    /// loan. The default implementation returns `Address::ZERO`.
    pub fn _flash_fee_receiver(&self) -> Address {
        // if self._flash_fee_receiver_address.eq(&Address::ZERO) {
        //     return self._flash_fee_receiver_address.get();
        // }
        Address::ZERO
    }
}
