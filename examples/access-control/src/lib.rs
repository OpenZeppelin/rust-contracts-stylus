#![cfg_attr(not(test), no_std, no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use contracts::{access::control::AccessControl, erc20::ERC20};
use stylus_sdk::{
    msg,
    prelude::{entrypoint, external, sol_storage},
};

sol_storage! {
    #[entrypoint]
    struct Token {
        #[borrow]
        ERC20 erc20;
        #[borrow]
        AccessControl access;
    }
}

#[external]
#[inherit(ERC20, AccessControl)]
impl Token {
    // `keccak256("TRANSFER_ROLE")`
    const TRANSFER_ROLE: [u8; 32] = [
        133, 2, 35, 48, 150, 217, 9, 190, 251, 218, 9, 153, 187, 142, 162, 243,
        166, 190, 60, 19, 139, 159, 191, 0, 55, 82, 164, 200, 188, 232, 111,
        108,
    ];

    pub fn constructor(&mut self) -> Result<(), Vec<u8>> {
        // Make caller the default admin.
        self.access.grant_role(
            AccessControl::DEFAULT_ADMIN_ROLE.into(),
            msg::sender(),
        )?;
        Ok(())
    }

    pub fn make_admin(&mut self, account: Address) -> Result<(), Vec<u8>> {
        self.access.only_role(AccessControl::DEFAULT_ADMIN_ROLE.into())?;
        self.access.grant_role(Token::TRANSFER_ROLE.into(), account)?;
        Ok(())
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        self.access.only_role(Token::TRANSFER_ROLE.into())?;
        let transfer_result = self.erc20.transfer_from(from, to, value)?;
        Ok(transfer_result)
    }
}
