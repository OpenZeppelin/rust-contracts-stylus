use alloy::{primitives::Address, rpc::types::TransactionReceipt};

/// Extension trait to recover address of the contract that was deployed.
pub trait Ext {
    /// Returns the address of the contract from the [`TransactionReceipt`].
    ///
    /// # Errors
    ///
    /// May fail if there's no contract address.
    fn address(&self) -> Address;
}

impl Ext for (TransactionReceipt, Address) {
    fn address(&self) -> Address {
        self.1
    }
}
