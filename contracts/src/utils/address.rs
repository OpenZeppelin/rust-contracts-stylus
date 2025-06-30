use alloc::{vec, vec::Vec};

use alloy_primitives::Address;
use stylus_sdk::{abi::Bytes, prelude::*};

/// An [`AddressHelper`] error.
#[derive(SolidityError, Debug)]
pub enum Error {}

/// State of an [`AddressHelper`] contract.
#[storage]
pub struct AddressHelper {}

impl AddressHelper {
    /// TODO: docs
    pub fn function_delegate_call(
        &self,
        _target: Address,
        _data: Bytes,
    ) -> Result<Bytes, Error> {
        unimplemented!()
    }
}
