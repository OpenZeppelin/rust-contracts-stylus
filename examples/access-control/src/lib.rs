#![cfg_attr(not(test), no_std, no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, B256, U256};
use openzeppelin_stylus::{
    access::control::{AccessControl, IAccessControl},
    token::erc20::{Erc20, IErc20},
};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct AccessControlExample {
        #[borrow]
        Erc20 erc20;
        #[borrow]
        AccessControl access;
    }
}

// `keccak256("TRANSFER_ROLE")`
pub const TRANSFER_ROLE: [u8; 32] = [
    133, 2, 35, 48, 150, 217, 9, 190, 251, 218, 9, 153, 187, 142, 162, 243,
    166, 190, 60, 19, 139, 159, 191, 0, 55, 82, 164, 200, 188, 232, 111, 108,
];

#[public]
#[inherit(Erc20, AccessControl)]
impl AccessControlExample {
    pub const TRANSFER_ROLE: [u8; 32] = TRANSFER_ROLE;

    pub fn make_admin(&mut self, account: Address) -> Result<(), Vec<u8>> {
        self.access.only_role(AccessControl::DEFAULT_ADMIN_ROLE.into())?;
        self.access
            .grant_role(AccessControlExample::TRANSFER_ROLE.into(), account)?;
        Ok(())
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        self.access.only_role(AccessControlExample::TRANSFER_ROLE.into())?;
        let transfer_result = self.erc20.transfer_from(from, to, value)?;
        Ok(transfer_result)
    }

    // WARNING: This should not be part of the public API, it's here for testing
    // purposes only.
    pub fn set_role_admin(&mut self, role: B256, new_admin_role: B256) {
        self.access._set_role_admin(role, new_admin_role)
    }
}
