use alloy::{primitives::Address, rpc::types::TransactionReceipt};

/// Transaction receipt wrapper that contains both the receipt of the
/// transaction sent to the StylusDeployer, and the contract address of the
/// created/activated or even would-be created contract.
///
/// This is necessary because calling [`TransactionReceipt::contract_address`]
/// would return the address of StylusDeployer, instead of the newly deployed
/// contract.
pub struct Receipt {
    /// Transaction receipt of the tx sent to StylusDeployer.
    pub inner: TransactionReceipt,
    /// Address of the contract.
    pub contract_address: Address,
}
