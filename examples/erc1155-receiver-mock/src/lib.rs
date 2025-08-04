#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::{
    token::erc1155::receiver::IErc1155Receiver,
    utils::introspection::erc165::IErc165,
};
pub use sol::*;
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{
        aliases::{B32, U8},
        Address, U256,
    },
    evm,
    prelude::*,
    storage::{StorageB32, StorageU8},
};

mod sol {
    use alloy_sol_macro::sol;
    sol! {
        #[derive(Debug)]
        #[allow(missing_docs)]
        event Received(address operator, address from, uint256 id, uint256 value, bytes data);

        #[derive(Debug)]
        #[allow(missing_docs)]
        event BatchReceived(address operator, address from, uint256[] ids, uint256[] values, bytes data);
    }

    sol! {
        #[derive(Debug)]
        #[allow(missing_docs)]
        error CustomError(bytes4);
    }
}

/// Enum representing different revert types for testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevertType {
    None,
    RevertWithoutMessage,
    RevertWithMessage,
    RevertWithCustomError,
    Panic,
}

impl From<u8> for RevertType {
    fn from(value: u8) -> Self {
        match value {
            0 => RevertType::None,
            1 => RevertType::RevertWithoutMessage,
            2 => RevertType::RevertWithMessage,
            3 => RevertType::RevertWithCustomError,
            4 => RevertType::Panic,
            _ => RevertType::None,
        }
    }
}

impl From<U8> for RevertType {
    fn from(value: U8) -> Self {
        let revert_type: u8 = value.to_be_bytes::<1>()[0];
        RevertType::from(revert_type)
    }
}

impl From<RevertType> for u8 {
    fn from(value: RevertType) -> Self {
        match value {
            RevertType::None => 0,
            RevertType::RevertWithoutMessage => 1,
            RevertType::RevertWithMessage => 2,
            RevertType::RevertWithCustomError => 3,
            RevertType::Panic => 4,
        }
    }
}

#[entrypoint]
#[storage]
struct Erc1155ReceiverMock {
    rec_retval: StorageB32,
    bat_retval: StorageB32,
    error_type: StorageU8,
}

#[public]
#[implements(IErc1155Receiver, IErc165)]
impl Erc1155ReceiverMock {
    #[constructor]
    fn constructor(
        &mut self,
        rec_retval: B32,
        bat_retval: B32,
        error_type: U8,
    ) -> Result<(), Vec<u8>> {
        self.rec_retval.set(rec_retval);
        self.bat_retval.set(bat_retval);
        self.error_type.set(error_type);
        Ok(())
    }
}

#[public]
impl IErc1155Receiver for Erc1155ReceiverMock {
    #[allow(deprecated)]
    fn on_erc1155_received(
        &mut self,
        operator: Address,
        from: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<B32, Vec<u8>> {
        let error_type: RevertType = self.error_type.get().into();

        match error_type {
            RevertType::RevertWithoutMessage => {
                return Err(vec![]);
            }
            RevertType::RevertWithMessage => {
                return Err(
                    b"ERC1155ReceiverMock: reverting on receive".to_vec()
                );
            }
            RevertType::RevertWithCustomError => {
                // For custom error, we'll just revert with the return value as
                // data
                let retval = self.rec_retval.get();
                return Err(retval.to_vec());
            }
            RevertType::Panic => {
                // Simulate a panic by dividing by zero
                let _ = U256::from(0) / U256::from(0);
                return Err(vec![]);
            }
            RevertType::None => {}
        }

        // Emit the Received event
        evm::log(Received {
            operator,
            from,
            id,
            value,
            data: data.to_vec().into(),
        });

        // Return the configured return value
        Ok(self.rec_retval.get())
    }

    #[allow(deprecated)]
    fn on_erc1155_batch_received(
        &mut self,
        operator: Address,
        from: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<B32, Vec<u8>> {
        let error_type: RevertType = self.error_type.get().into();

        match error_type {
            RevertType::RevertWithoutMessage => {
                return Err(vec![]);
            }
            RevertType::RevertWithMessage => {
                return Err(
                    b"ERC1155ReceiverMock: reverting on batch receive".to_vec()
                );
            }
            RevertType::RevertWithCustomError => {
                // For custom error, we'll just revert with the return value as
                // data
                let retval = self.bat_retval.get();
                return Err(retval.to_vec());
            }
            RevertType::Panic => {
                // Simulate a panic by dividing by zero
                let _ = U256::from(0) / U256::from(0);
                return Err(vec![]);
            }
            RevertType::None => {}
        }

        // Emit the BatchReceived event
        evm::log(BatchReceived {
            operator,
            from,
            ids,
            values,
            data: data.to_vec().into(),
        });

        // Return the configured return value
        Ok(self.bat_retval.get())
    }
}

#[public]
impl IErc165 for Erc1155ReceiverMock {
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IErc1155Receiver>::interface_id() == interface_id
            || <Self as IErc165>::interface_id() == interface_id
    }
}
