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

pub type Provider = FillProvider<
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

pub struct Context {
    rpc_url: Url,
}

impl Context {
    pub fn rpc_url(&self) -> &Url {
        &self.rpc_url
    }

    pub fn users(&self) -> &[User] {
        &self.users
    }
}

pub fn build_context() -> Context {
    let rpc_url = std::env::var(RPC_URL)
        .expect("failed to load RPC_URL var from env")
        .parse()
        .expect("failed to parse RPC_URL string into a URL");

    Context { rpc_url }
}

fn load_var(name: &str) -> eyre::Result<String> {
    std::env::var(name).wrap_err(format!("failed to load {name}"))
}
