#![doc = include_str!("../README.md")]
mod account;
mod constructor;
mod deploy;
mod devnet;
mod error;
mod event;
mod receipt;

pub use account::{Account, Wallet, DEFAULT_FUNDING_WEI};
pub use constructor::Constructor;
pub use deploy::{ContractDeploymentError, ContractInitializationError, Deployer};
pub use devnet::{
    daemon_pid, devnet_dir, pid_file, rpc_file, rpc_url, DEVNET_PRIVATE_KEY,
    RPC_URL_ENV_VAR, STYLUS_DEPLOYER,
};
pub use e2e_proc::test;
pub use error::{Panic, PanicCode, Revert, RustPanic};
pub use event::Ext as EventExt;
pub use receipt::Receipt;

/// Items used by the macros of this crate. Not a public API.
#[doc(hidden)]
pub mod __private {
    pub use alloy::sol_types::SolValue;
}

/// This macro provides a shorthand for broadcasting the transaction to the
/// network.
///
/// See: <https://docs.rs/alloy-contract/latest/alloy_contract/struct.CallBuilder.html>
///
/// # Examples
///
/// ```rust,ignore
/// #[e2e::test]
/// async fn foo(alice: Account) -> eyre::Result<()> {
///     let contract_addr = alice.as_deployer().deploy().await?.contract_address;
///     let contract = Erc721::new(contract_addr, &alice.wallet);
///
///     let alice_addr = alice.address();
///     let token_id = random_token_id();
///     let pending_tx = send!(contract.mint(alice_addr, token_id))?;
///     // ...
/// }
#[macro_export]
macro_rules! send {
    ($e:expr) => {
        $e.send().await
    };
}

/// This macro provides a shorthand for broadcasting the transaction
/// to the network, and then waiting for the given number of confirmations.
///
/// See: <https://docs.rs/alloy-provider/latest/alloy_provider/struct.PendingTransactionBuilder.html>
///
/// # Examples
///
/// ```rust,ignore
/// #[e2e::test]
/// async fn foo(alice: Account) -> eyre::Result<()> {
///     let contract_addr = alice.as_deployer().deploy().await?.contract_address;
///     let contract = Erc721::new(contract_addr, &alice.wallet);
///
///     let alice_addr = alice.address();
///     let token_id = random_token_id();
///     let result = watch!(contract.mint(alice_addr, token_id))?;
///     // ...
/// }
#[macro_export]
macro_rules! watch {
    ($e:expr) => {
        $crate::send!($e)?.watch().await
    };
}

/// This macro provides a shorthand for broadcasting the transaction
/// to the network, waiting for the given number of confirmations, and then
/// fetching the transaction receipt.
///
/// See: <https://docs.rs/alloy-provider/latest/alloy_provider/struct.PendingTransactionBuilder.html>
///
/// # Examples
///
/// ```rust,ignore
/// #[e2e::test]
/// async fn foo(alice: Account) -> eyre::Result<()> {
///     let contract_addr = alice.as_deployer().deploy().await?.contract_address;
///     let contract = Erc721::new(contract_addr, &alice.wallet);
///
///     let alice_addr = alice.address();
///     let token_id = random_token_id();
///     let receipt = receipt!(contract.mint(alice_addr, token_id))?;
///     // ...
/// }
#[macro_export]
macro_rules! receipt {
    ($e:expr) => {
        $crate::send!($e)?.get_receipt().await
    };
}
