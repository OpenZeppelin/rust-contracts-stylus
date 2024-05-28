use std::sync::Arc;

use async_trait::async_trait;
use ethers::{
    abi::{AbiEncode, Detokenize},
    addressbook::Address,
    contract::ContractCall,
    middleware::SignerMiddleware,
    prelude::{Http, LocalWallet, Provider, TransactionReceipt, U256},
};
use eyre::{bail, Context, ContextCompat, Report};

use crate::prelude::U64;
pub mod context;
pub mod prelude;
pub mod user;
pub use e2e_proc::test;

/// Abstraction for the deployed contract.
pub trait Contract {
    /// Crate name of the contract.
    ///
    /// e.g can be `erc721-example`.
    const CRATE_NAME: &'static str;

    /// Abstracts token creation function.
    ///
    /// e.g. `Self::new(address, client)`.
    fn new(address: Address, client: Arc<HttpMiddleware>) -> Self;
}

#[macro_export]
/// Link `abigen!` contract to the crate name.
///
/// # Example
/// ```
/// use e2e::prelude::*;
///
/// abigen!(
///     Erc20Token,
///     r#"[
///         function transferFrom(address sender, address recipient, uint256 amount) external returns (bool)
///         function mint(address account, uint256 amount) external
///         
///         error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed)
///     ]"#
/// );
///
/// pub type Erc20 = Erc20Token<HttpMiddleware>;
/// link_to_crate!(Erc20, "erc20-example");
/// ```
macro_rules! link_to_crate {
    ($token_type:ty, $program_address:literal) => {
        impl $crate::Contract for $token_type {
            const CRATE_NAME: &'static str = $program_address;

            fn new(
                address: ethers::types::Address,
                client: std::sync::Arc<HttpMiddleware>,
            ) -> Self {
                Self::new(address, client)
            }
        }
    };
}

pub type HttpMiddleware = SignerMiddleware<Provider<Http>, LocalWallet>;

pub fn random_token_id() -> U256 {
    let num: u32 = ethers::core::rand::random();
    num.into()
}

pub trait Assert<E: AbiEncode> {
    /// Asserts that current error result corresponds to the typed abi encoded
    /// error `expected_err`.
    fn assert(&self, expected_err: E) -> eyre::Result<()>;
}

impl<E: AbiEncode> Assert<E> for Report {
    fn assert(&self, expected_err: E) -> eyre::Result<()> {
        let received_err = format!("{:#}", self);
        let expected_err = expected_err.encode_hex();
        if received_err.contains(&expected_err) {
            Ok(())
        } else {
            bail!("Different error expected: Expected error is {expected_err}: Received error is {received_err}")
        }
    }
}

#[async_trait]
pub trait ContextCall<R> {
    /// Queries the blockchain via an `eth_call` for the provided transaction.
    ///
    /// Wraps error with function info context.
    ///
    /// If executed on a non-state mutating smart contract function (i.e.
    /// `view`, `pure`) then it will return the raw data from the chain.
    ///
    /// If executed on a mutating smart contract function, it will do a "dry
    /// run" of the call and return the return type of the transaction
    /// without mutating the state
    async fn ctx_call(self) -> eyre::Result<R>;
}

#[async_trait]
impl<R: Detokenize + Send + Sync> ContextCall<R>
    for ContractCall<HttpMiddleware, R>
{
    async fn ctx_call(self) -> eyre::Result<R> {
        let function_name = &self.function.name;
        self.call().await.context(format!("call {function_name}"))
    }
}

#[async_trait]
pub trait ContextSend {
    /// Signs and broadcasts the provided transaction.
    ///
    /// Wraps error with function info context.
    async fn ctx_send(self) -> eyre::Result<TransactionReceipt>;
}

#[async_trait]
impl ContextSend for ContractCall<HttpMiddleware, ()> {
    async fn ctx_send(self) -> eyre::Result<TransactionReceipt> {
        let function_name = &self.function.name;
        let tx = self
            .send()
            .await
            .context(format!("send {function_name}"))?
            .await
            .context(format!("send {function_name}"))?
            .context(format!("send {function_name}"))?;
        match tx.status {
            Some(status) if status == U64::zero() => {
                bail!("send {function_name}: transaction status is not success")
            }
            _ => Ok(tx),
        }
    }
}
