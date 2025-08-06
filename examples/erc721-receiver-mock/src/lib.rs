#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::token::erc721::{
    receiver::{IErc721Receiver, RECEIVER_FN_SELECTOR},
    utils::Erc721Holder,
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
        event Received(address operator, address from, uint256 token_id, bytes data);
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
struct Erc721ReceiverMock {
    error_type: StorageU8,
    holder: Erc721Holder,
}

#[public]
#[implements(IErc721Receiver)]
impl Erc721ReceiverMock {
    #[constructor]
    fn constructor(&mut self, error_type: U8) {
        self.error_type.set(error_type)
    }
}

#[public]
impl IErc721Receiver for Erc721ReceiverMock {
    #[selector(name = "onERC721Received")]
    fn on_erc721_received(
        &mut self,
        operator: Address,
        from: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<B32, Vec<u8>> {
        let error_type: RevertType = self.error_type.get().into();

        match error_type {
            RevertType::CustomError => {
                Err(Error::MockCustomError(CustomError {
                    data: RECEIVER_FN_SELECTOR,
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
                    token_id,
                    data: data.to_vec().into(),
                });
                self.holder.on_erc721_received(operator, from, token_id, data)
            }
        }
    }
}
