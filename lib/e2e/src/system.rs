use std::ops::Deref;

use alloy::{
    network::{Ethereum, EthereumSigner},
    providers::{
        fillers::{
            ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
            SignerFiller,
        },
        Identity, ProviderBuilder, RootProvider,
    },
    signers::wallet::LocalWallet,
    transports::http::{reqwest::Url, Client, Http},
};
use eyre::Context as Ctx;

use crate::user::User;

const ALICE_PRIV_KEY: &str = "ALICE_PRIV_KEY";
const BOB_PRIV_KEY: &str = "BOB_PRIV_KEY";
const RPC_URL: &str = "RPC_URL";

pub type Signer = FillProvider<
    JoinFill<
        JoinFill<
            JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>,
            ChainIdFiller,
        >,
        SignerFiller<EthereumSigner>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

pub type Provider = FillProvider<
    JoinFill<
        JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>,
        ChainIdFiller,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

pub fn env(name: &str) -> eyre::Result<String> {
    std::env::var(name).wrap_err(format!("failed to load {name}"))
}

pub fn provider() -> Provider {
    let rpc_url = std::env::var(RPC_URL)
        .expect("failed to load RPC_URL var from env")
        .parse()
        .expect("failed to parse RPC_URL string into a URL");
    let p = ProviderBuilder::new().with_recommended_fillers().on_http(rpc_url);
    p
}
