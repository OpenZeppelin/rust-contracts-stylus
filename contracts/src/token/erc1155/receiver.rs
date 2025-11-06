//! Module with an interface required for smart contract in order to receive
//! ERC-1155 token transfers.
#![allow(missing_docs)]
#![cfg_attr(coverage_nightly, coverage(off))]
use alloc::vec::Vec;

use alloy_primitives::{aliases::B32, Address, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{abi::Bytes, function_selector};

use crate::utils::introspection::erc165::IErc165;

/// The expected value returned from [`IErc1155Receiver::on_erc1155_received`].
pub const SINGLE_TRANSFER_FN_SELECTOR: B32 = B32::new(function_selector!(
    "onERC1155Received",
    Address,
    Address,
    U256,
    U256,
    Bytes
));

/// The expected value returned from
/// [`IErc1155Receiver::on_erc1155_batch_received`].
pub const BATCH_TRANSFER_FN_SELECTOR: B32 = B32::new(function_selector!(
    "onERC1155BatchReceived",
    Address,
    Address,
    Vec<U256>,
    Vec<U256>,
    Bytes
));

/// Interface that must be implemented by smart contracts in order to receive
/// ERC-1155 token transfers.
#[interface_id]
pub trait IErc1155Receiver: IErc165 {
    /// Handles the receipt of a single ERC-1155 token type. This function
    /// is called at the end of [`super::IErc1155::safe_transfer_from`] after
    /// the balance has been updated.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - The address which initiated the transfer.
    /// * `from` - The address which previously owned the token.
    /// * `id` - The ID of the token being transferred.
    /// * `value` - The amount of tokens being transferred.
    /// * `data` - Additional data with no specified format.
    ///
    /// # Errors
    ///
    /// * May return a custom error.
    #[selector(name = "onERC1155Received")]
    fn on_erc1155_received(
        &mut self,
        operator: Address,
        from: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<B32, Vec<u8>>;

    /// Handles the receipt of multiple ERC-1155 token types. This function
    /// is called at the end of
    /// [`super::IErc1155::safe_batch_transfer_from`] after the balances
    /// have been updated.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - The address which initiated the batch transfer.
    /// * `from` - The address which previously owned the token.
    /// * `ids` - An array containing ids of each token being transferred (order
    ///   and length must match `values` array).
    /// * `values` - An array containing amounts of each token being transferred
    ///   (order and length must match `ids` array).
    /// * `data` - Additional data with no specified format.
    ///
    /// # Errors
    ///
    /// * May return a custom error.
    #[selector(name = "onERC1155BatchReceived")]
    fn on_erc1155_batch_received(
        &mut self,
        operator: Address,
        from: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<B32, Vec<u8>>;
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
pub(crate) mod tests {
    #![allow(clippy::unused_self)]

    use stylus_sdk::prelude::*;

    use super::*;
    use crate::utils::introspection::erc165::IErc165;

    /// ERC-1155 receiver that returns the wrong selector.
    #[storage]
    pub(crate) struct BadSelectorReceiver1155;

    unsafe impl TopLevelStorage for BadSelectorReceiver1155 {}

    #[public]
    #[implements(IErc1155Receiver, IErc165)]
    impl BadSelectorReceiver1155 {}

    #[public]
    impl IErc165 for BadSelectorReceiver1155 {
        fn supports_interface(&self, interface_id: B32) -> bool {
            // Declare support for [`IErc1155Receiver`] so calls are attempted.
            <Self as IErc1155Receiver>::interface_id() == interface_id
                || <Self as IErc165>::interface_id() == interface_id
        }
    }

    #[public]
    impl IErc1155Receiver for BadSelectorReceiver1155 {
        #[selector(name = "onERC1155Received")]
        fn on_erc1155_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _id: U256,
            _value: U256,
            _data: Bytes,
        ) -> Result<B32, Vec<u8>> {
            Ok(B32::ZERO) // wrong selector -> should be rejected.
        }

        #[selector(name = "onERC1155BatchReceived")]
        fn on_erc1155_batch_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _ids: Vec<U256>,
            _values: Vec<U256>,
            _data: Bytes,
        ) -> Result<B32, Vec<u8>> {
            Ok(B32::ZERO) // wrong selector -> should be rejected.
        }
    }

    /// ERC-1155 receiver that reverts.
    #[storage]
    pub(crate) struct RevertingReceiver1155;

    unsafe impl TopLevelStorage for RevertingReceiver1155 {}

    #[public]
    #[implements(IErc1155Receiver, IErc165)]
    impl RevertingReceiver1155 {}

    #[public]
    impl IErc165 for RevertingReceiver1155 {
        fn supports_interface(&self, interface_id: B32) -> bool {
            <Self as IErc1155Receiver>::interface_id() == interface_id
                || <Self as IErc165>::interface_id() == interface_id
        }
    }

    #[public]
    impl IErc1155Receiver for RevertingReceiver1155 {
        #[selector(name = "onERC1155Received")]
        fn on_erc1155_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _id: U256,
            _value: U256,
            _data: Bytes,
        ) -> Result<B32, Vec<u8>> {
            Err("Receiver rejected single".into())
        }

        #[selector(name = "onERC1155BatchReceived")]
        fn on_erc1155_batch_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _ids: Vec<U256>,
            _values: Vec<U256>,
            _data: Bytes,
        ) -> Result<B32, Vec<u8>> {
            Err("Receiver rejected batch".into())
        }
    }

    /// ERC-1155 receiver that returns the correct acceptance selectors.
    #[storage]
    pub(crate) struct SuccessReceiver1155;

    unsafe impl TopLevelStorage for SuccessReceiver1155 {}

    #[public]
    #[implements(IErc1155Receiver, IErc165)]
    impl SuccessReceiver1155 {}

    #[public]
    impl IErc165 for SuccessReceiver1155 {
        fn supports_interface(&self, interface_id: B32) -> bool {
            <Self as IErc1155Receiver>::interface_id() == interface_id
                || <Self as IErc165>::interface_id() == interface_id
        }
    }

    #[public]
    impl IErc1155Receiver for SuccessReceiver1155 {
        #[selector(name = "onERC1155Received")]
        fn on_erc1155_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _id: U256,
            _value: U256,
            _data: Bytes,
        ) -> Result<B32, Vec<u8>> {
            Ok(SINGLE_TRANSFER_FN_SELECTOR)
        }

        #[selector(name = "onERC1155BatchReceived")]
        fn on_erc1155_batch_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _ids: Vec<U256>,
            _values: Vec<U256>,
            _data: Bytes,
        ) -> Result<B32, Vec<u8>> {
            Ok(BATCH_TRANSFER_FN_SELECTOR)
        }
    }

    /// ERC-1155 receiver that reverts with an empty reason (covers Err(Revert)
    /// with empty data).
    #[storage]
    pub(crate) struct EmptyReasonReceiver1155;

    unsafe impl TopLevelStorage for EmptyReasonReceiver1155 {}

    #[public]
    #[implements(IErc1155Receiver, IErc165)]
    impl EmptyReasonReceiver1155 {}

    #[public]
    impl IErc1155Receiver for EmptyReasonReceiver1155 {
        #[selector(name = "onERC1155Received")]
        fn on_erc1155_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _id: U256,
            _value: U256,
            _data: Bytes,
        ) -> Result<B32, Vec<u8>> {
            Err(Vec::new())
        }

        #[selector(name = "onERC1155BatchReceived")]
        fn on_erc1155_batch_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _ids: Vec<U256>,
            _values: Vec<U256>,
            _data: Bytes,
        ) -> Result<B32, Vec<u8>> {
            Err(Vec::new())
        }
    }

    #[public]
    impl IErc165 for EmptyReasonReceiver1155 {
        fn supports_interface(&self, interface_id: B32) -> bool {
            <Self as IErc1155Receiver>::interface_id() == interface_id
                || <Self as IErc165>::interface_id() == interface_id
        }
    }

    /// ERC-1155 receiver that exposes the expected selectors but with no return
    /// value, producing a successful call with empty return data (ABI decode
    /// error -> non-Revert call error)
    #[storage]
    pub(crate) struct MisdeclaredReceiver1155;

    unsafe impl TopLevelStorage for MisdeclaredReceiver1155 {}

    impl MisdeclaredReceiver1155 {
        // mock interface_id function
        fn interface_id(&self) -> B32 {
            let single_transfer_fn_selector = B32::new(function_selector!(
                "onERC1155Received",
                Address,
                Address,
                U256,
                U256,
                Bytes
            ));

            let batch_transfer_fn_selector = B32::new(function_selector!(
                "onERC1155BatchReceived",
                Address,
                Address,
                Vec<U256>,
                Vec<U256>,
                Bytes
            ));

            single_transfer_fn_selector ^ batch_transfer_fn_selector
        }
    }

    #[public]
    impl IErc165 for MisdeclaredReceiver1155 {
        fn supports_interface(&self, interface_id: B32) -> bool {
            // Pretend to support the receiver so the call is attempted
            self.interface_id() == interface_id
                || <Self as IErc165>::interface_id() == interface_id
        }
    }

    /// ERC-1155 receiver that exposes the expected selectors but with no return
    /// value, producing a successful call with empty return data (ABI decode
    /// error -> non-Revert call error).
    #[public]
    impl MisdeclaredReceiver1155 {
        #[selector(name = "onERC1155Received")]
        fn on_erc1155_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _id: U256,
            _value: U256,
            _data: Bytes,
        ) {
            // return no data
        }

        #[selector(name = "onERC1155BatchReceived")]
        fn on_erc1155_batch_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _ids: Vec<U256>,
            _values: Vec<U256>,
            _data: Bytes,
        ) {
            // return no data
        }
    }
}
