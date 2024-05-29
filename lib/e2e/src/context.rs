use std::marker::PhantomData;

use ethers::addressbook::Address;

use crate::contract::Contract;

/// End-to-end testing context that allows to act on behalf of any user that
/// [`crate::user::User::uses`] it.
pub struct E2EContext<T: Contract> {
    address: Address,
    phantom_data: PhantomData<T>,
}

impl<T: Contract> E2EContext<T> {
    pub(crate) fn new(address: Address) -> E2EContext<T> {
        Self { address, phantom_data: PhantomData }
    }

    /// Retrieve address of the contract deployed
    pub fn address(&self) -> Address {
        self.address
    }
}
