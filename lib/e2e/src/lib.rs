mod assert;
mod deploy;
mod environment;
mod project;
mod system;
mod user;

pub use assert::{Assert, Emits};
pub use deploy::deploy;
pub use e2e_proc::test;
pub use system::{provider, Provider, Signer};
pub use user::User;

/// This macro provides an easy way for broadcasting the transaction
/// to the network.
///
/// See: https://alloy-rs.github.io/alloy/alloy_contract/struct.CallBuilder.html
///
/// # Examples
///
/// ```rust,ignore
/// #[e2e::test]
/// async fn foo(alice: User) -> eyre::Result<()> {
///     let contract_addr = deploy(alice.url(), &alice.pk()).await?;
///     let contract = Erc721::new(contract_addr, &alice.signer);
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

/// This macro provides an easy way for broadcasting the transaction
/// to the network and then waiting for the given number of confirmations.
///
/// See: https://alloy-rs.github.io/alloy/alloy_provider/heart/struct.PendingTransactionBuilder.html
///
/// # Examples
///
/// ```rust,ignore
/// #[e2e::test]
/// async fn foo(alice: User) -> eyre::Result<()> {
///     let contract_addr = deploy(alice.url(), &alice.pk()).await?;
///     let contract = Erc721::new(contract_addr, &alice.signer);
///
///     let alice_addr = alice.address();
///     let token_id = random_token_id();
///     let result = watch!(contract.mint(alice_addr, token_id))?;
///     // ...
/// }
#[macro_export]
macro_rules! watch {
    ($e:expr) => {
        send!($e)?.watch().await
    };
}

/// This macro provides an easy way for broadcasting the transaction
/// to the network, waiting for the given number of confirmations, and then
/// fetching the transaction receipt.
///
/// See: https://alloy-rs.github.io/alloy/alloy_provider/heart/struct.PendingTransactionBuilder.html
///
/// # Examples
///
/// ```rust,ignore
/// #[e2e::test]
/// async fn foo(alice: User) -> eyre::Result<()> {
///     let contract_addr = deploy(alice.url(), &alice.pk()).await?;
///     let contract = Erc721::new(contract_addr, &alice.signer);
///
///     let alice_addr = alice.address();
///     let token_id = random_token_id();
///     let receipt = receipt!(contract.mint(alice_addr, token_id))?;
///     // ...
/// }
#[macro_export]
macro_rules! receipt {
    ($e:expr) => {
        send!($e)?.get_receipt().await
    };
}
