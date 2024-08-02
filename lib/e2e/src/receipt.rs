use alloy::network::ReceiptResponse;
use alloy::rpc::types::TransactionReceipt;
use alloy::primitives::Address;
use eyre::{ContextCompat, Result};

/// Extension trait to recover address of the contract that was deployed.
pub trait ReceiptExt {
    /// Returns the address of the contract from the [`TransactionReceipt`].
    fn address(&self) -> eyre::Result<Address>;
}

impl ReceiptExt for TransactionReceipt {
    fn address(&self) -> Result<Address> {
        self.contract_address().context("should contain contract address")
    }
}
