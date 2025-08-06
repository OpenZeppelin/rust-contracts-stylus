#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::{
    token::erc1155::{
        receiver::{
            IErc1155Receiver, BATCH_TRANSFER_FN_SELECTOR,
            SINGLE_TRANSFER_FN_SELECTOR,
        },
        utils::Erc1155Holder,
    },
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
    storage::StorageU8,
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
        error CustomError(bytes4 data);
    }
}

#[derive(SolidityError, Debug)]
pub enum Error {
    MockCustomError(CustomError),
}

/// Enum representing different revert types for testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevertType {
    None,
    CustomError,
    Panic,
}

impl From<u8> for RevertType {
    fn from(value: u8) -> Self {
        match value {
            1 => RevertType::CustomError,
            2 => RevertType::Panic,
            _ => RevertType::None,
        }
    }
}

impl From<U8> for RevertType {
    fn from(value: U8) -> Self {
        let revert_type: u8 = u8::try_from(value).expect("should be valid");
        revert_type.into()
    }
}

impl From<RevertType> for u8 {
    fn from(value: RevertType) -> Self {
        match value {
            RevertType::None => 0,
            RevertType::CustomError => 1,
            RevertType::Panic => 2,
        }
    }
}

#[entrypoint]
#[storage]
struct Erc1155ReceiverMock {
    error_type: StorageU8,
    holder: Erc1155Holder,
}

#[public]
#[implements(IErc1155Receiver, IErc165)]
impl Erc1155ReceiverMock {
    #[constructor]
    fn constructor(&mut self, error_type: U8) {
        self.error_type.set(error_type)
    }
}

#[public]
impl IErc1155Receiver for Erc1155ReceiverMock {
    #[selector(name = "onERC1155Received")]
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
            RevertType::CustomError => {
                Err(Error::MockCustomError(CustomError {
                    data: SINGLE_TRANSFER_FN_SELECTOR,
                })
                .into())
            }
            RevertType::Panic => {
                // simulate a panic by dividing by [`U256::ZERO`].
                let _ = U256::from(0) / U256::from(0);
                unreachable!()
            }
            RevertType::None => {
                #[allow(deprecated)]
                evm::log(Received {
                    operator,
                    from,
                    id,
                    value,
                    data: data.to_vec().into(),
                });
                self.holder.on_erc1155_received(operator, from, id, value, data)
            }
        }
    }

    #[selector(name = "onERC1155BatchReceived")]
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
            RevertType::CustomError => {
                Err(Error::MockCustomError(CustomError {
                    data: BATCH_TRANSFER_FN_SELECTOR,
                })
                .into())
            }
            RevertType::Panic => {
                // simulate a panic by dividing by [`U256::ZERO`].
                let _ = U256::from(0) / U256::from(0);
                unreachable!()
            }
            RevertType::None => {
                #[allow(deprecated)]
                evm::log(BatchReceived {
                    operator,
                    from,
                    ids: ids.clone(),
                    values: values.clone(),
                    data: data.to_vec().into(),
                });
                self.holder.on_erc1155_batch_received(
                    operator, from, ids, values, data,
                )
            }
        }
    }
}

#[public]
impl IErc165 for Erc1155ReceiverMock {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.holder.supports_interface(interface_id)
    }
}
