//! Module with an interface required for smart contract in order to receive
//! ERC-721 token transfers.
#![allow(missing_docs)]
#![cfg_attr(coverage_nightly, coverage(off))]
use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{abi::Bytes, function_selector, prelude::*};

/// The expected value returned from [`IErc721Receiver::on_erc721_received`].
pub const RECEIVER_FN_SELECTOR: [u8; 4] =
    function_selector!("onERC721Received", Address, Address, U256, Bytes,);

sol_interface! {
    /// [`super::Erc721`] token receiver Solidity interface.
    ///
    /// Check [`super::IErc721Receiver`] trait for more details.
    interface IErc721ReceiverInterface {
        /// See [`super::IErc721Receiver::on_erc721_received`].
        #[allow(missing_docs)]
        function onERC721Received(
            address operator,
            address from,
            uint256 token_id,
            bytes calldata data
        ) external returns (bytes4);
    }
}

/// [`super::IErc721`] token receiver trait.
///
/// Interface for any contract that wants to support
/// [`super::IErc721::safe_transfer_from`]
/// and [`super::IErc721::safe_transfer_from_with_data`] from ERC-721 asset
/// contracts.
#[interface_id]
pub trait IErc721Receiver {
    /// The error type associated to the trait implementation.
    type Error: Into<Vec<u8>>;

    /// This function is called whenever an [`super::Erc721`] `token_id`
    /// token is transferred to this contract via
    /// [`super::IErc721::safe_transfer_from`] or
    /// [`super::IErc721::safe_transfer_from_with_data`].
    ///
    /// It must return its its Solidity selector to confirm the token transfer.
    /// If any other value is returned or the interface is not implemented
    /// by the recipient, the transfer will be reverted.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - Account of the operator.
    /// * `from` - Account of the sender.
    /// * `token_id` - Token id as a number.
    /// * `data` - Additional data with no specified format.
    ///
    /// # Errors
    ///
    /// * May return custom error.
    fn on_erc721_received(
        &mut self,
        operator: Address,
        from: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<FixedBytes<4>, Self::Error>;
}

/// Mock ERC-721 token receiver for testing purposes.
#[cfg_attr(coverage_nightly, coverage(off))]
// #[cfg(test)]
pub mod mock {
    use alloy_primitives::{FixedBytes, U8};
    use alloy_sol_macro::sol;
    use stylus_sdk::{
        call::MethodError,
        evm,
        storage::{StorageFixedBytes, StorageU8},
    };

    pub use super::*;

    sol! {
        #[derive(Debug)]
        #[allow(missing_docs)]
        event Received(address operator, address from, uint256 tokenId, bytes data);

        #[derive(Debug)]
        #[allow(missing_docs)]
        error CustomError(bytes4);
    }

    /// Mock ERC-721 token receiver revert type.
    #[derive(Debug, PartialEq)]
    #[repr(u8)]
    pub enum RevertType {
        /// No revert.
        None = 0,
        /// Simulate a revert without message.
        RevertWithoutMessage = 1,
        /// Simulate a revert with message.
        RevertWithMessage = 2,
        /// Simulate a revert with custom error.
        CustomError = 3,
        /// Simulate a panic (division by zero).
        Panic = 4,
    }

    impl From<U8> for RevertType {
        fn from(value: U8) -> Self {
            value.into()
        }
    }

    /// An [`Erc721ReceiverMock`] error.
    #[derive(SolidityError, Debug)]
    pub enum Error {
        /// Custom error.
        ReceiverCustomError(CustomError),
    }

    impl MethodError for Error {
        fn encode(self) -> alloc::vec::Vec<u8> {
            self.into()
        }
    }

    /// Mock ERC-721 token receiver for testing purposes.
    #[storage]
    pub struct Erc721ReceiverMock {
        retval: StorageFixedBytes<4>,
        revert_type: StorageU8,
    }

    #[public]
    #[implements(IErc721Receiver<Error = Error>)]
    impl Erc721ReceiverMock {
        #[constructor]
        fn constructor(&mut self, retval: FixedBytes<4>, revert_type: U8) {
            self.retval.set(retval);
            self.revert_type.set(revert_type.into());
        }
    }

    #[public]
    impl IErc721Receiver for Erc721ReceiverMock {
        type Error = Error;

        #[selector(name = "onERC721Received")]
        fn on_erc721_received(
            &mut self,
            operator: Address,
            from: Address,
            token_id: U256,
            data: Bytes,
        ) -> Result<FixedBytes<4>, Self::Error> {
            match self.revert_type.get().into() {
                RevertType::RevertWithoutMessage => {
                    panic!("");
                }
                RevertType::RevertWithMessage => {
                    panic!("ERC721ReceiverMock: reverting");
                }
                RevertType::CustomError => {
                    return Err(Error::ReceiverCustomError(CustomError {
                        _0: self.retval.get(),
                    }))
                }
                RevertType::Panic => {
                    let _panic = U256::ZERO / U256::ZERO;
                    unreachable!()
                }
                RevertType::None => {
                    evm::log(Received {
                        operator,
                        from,
                        tokenId: token_id,
                        data: data.as_slice().to_vec().into(),
                    });
                    Ok(self.retval.get())
                }
            }
        }
    }
}
