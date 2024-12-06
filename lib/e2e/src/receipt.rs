use alloy::{
    network::ReceiptResponse, primitives::Address,
    rpc::types::TransactionReceipt,
};
use eyre::ContextCompat;

/// Extension trait to recover address of the contract that was deployed.
pub trait Ext {
    /// Returns the address of the contract from the [`TransactionReceipt`].
    ///
    /// # Errors
    ///
    /// May fail if there's no contract address.
    fn address(&self) -> eyre::Result<Address>;
}

impl Ext for TransactionReceipt {
    fn address(&self) -> eyre::Result<Address> {
        self.contract_address().context("should contain contract address")
    }
}
