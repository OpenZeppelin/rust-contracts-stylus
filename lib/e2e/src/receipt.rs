use alloy::{primitives::Address, rpc::types::TransactionReceipt};

/// Transaction receipt wrapper that contains both the receipt of the
/// deployment transaction and the address of the deployed contract.
///
/// This is necessary because for deployments that run a constructor the
/// transaction is sent to the `StylusDeployer` contract, so
/// [`TransactionReceipt::contract_address`] is empty.
#[derive(Debug)]
pub struct Receipt {
    /// Receipt of the deployment transaction.
    pub inner: TransactionReceipt,
    /// Address of the deployed contract.
    pub contract_address: Address,
}
