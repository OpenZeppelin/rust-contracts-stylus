#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, B256, U256};
use openzeppelin_stylus::{
    access::control::AccessControl,
    token::erc20::{Erc20, IErc20},
};
use stylus_sdk::prelude::{entrypoint, public, storage};

#[entrypoint]
#[storage]
struct AccessControlExample {
    #[borrow]
    pub erc20: Erc20,
    #[borrow]
    pub access: AccessControl,
}

pub const TRANSFER_ROLE: [u8; 32] =
    keccak_const::Keccak256::new().update(b"TRANSFER_ROLE").finalize();

#[public]
#[inherit(Erc20, AccessControl)]
impl AccessControlExample {
    pub fn make_admin(&mut self, account: Address) -> Result<(), Vec<u8>> {
        self.access.only_role(AccessControl::DEFAULT_ADMIN_ROLE.into())?;
        self.access.grant_role(TRANSFER_ROLE.into(), account)?;
        Ok(())
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        self.access.only_role(TRANSFER_ROLE.into())?;
        let transfer_result = self.erc20.transfer_from(from, to, value)?;
        Ok(transfer_result)
    }

    // WARNING: This should not be part of the public API, it's here for testing
    // purposes only.
    pub fn set_role_admin(&mut self, role: B256, new_admin_role: B256) {
        self.access._set_role_admin(role, new_admin_role)
    }
}

impl AccessControlExample {
    pub const TRANSFER_ROLE: [u8; 32] = TRANSFER_ROLE;
}
