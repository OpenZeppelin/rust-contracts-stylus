use alloy::{
    network::{Ethereum, EthereumWallet},
    providers::{
        fillers::{
            ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
            WalletFiller,
        },
        Identity, ProviderBuilder, RootProvider,
    },
    transports::http::{Client, Http},
};
use eyre::Context;

pub(crate) const RPC_URL_ENV_VAR_NAME: &str = "RPC_URL";

/// Convenience type alias that represents an Ethereum wallet.
pub type Wallet = FillProvider<
    JoinFill<
        JoinFill<
            JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>,
            ChainIdFiller,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

/// Convenience type alias that represents an alloy provider.
pub type Provider = FillProvider<
    JoinFill<
        JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>,
        ChainIdFiller,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

/// Load the `name` environment variable.
fn env(name: &str) -> eyre::Result<String> {
    std::env::var(name).wrap_err(format!("failed to load {name}"))
}

/// Returns an alloy provider connected to the `RPC_URL` rpc endpoint.
///
/// # Panics
///
/// May panic if unable to load the `RPC_URL` environment variable.
#[must_use]
pub fn provider() -> Provider {
    let rpc_url = env(RPC_URL_ENV_VAR_NAME)
        .expect("failed to load RPC_URL var from env")
        .parse()
        .expect("failed to parse RPC_URL string into a URL");
    ProviderBuilder::new().with_recommended_fillers().on_http(rpc_url)
}
