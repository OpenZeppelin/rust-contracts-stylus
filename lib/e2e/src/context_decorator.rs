use async_trait::async_trait;
use eyre::{bail, ContextCompat, WrapErr};

use crate::{
    prelude::{abi::Detokenize, ContractCall, TransactionReceipt, U64},
    HttpMiddleware,
};

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
