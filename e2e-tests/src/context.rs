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

pub struct Context {
    rpc_url: Url,
    private_keys: Vec<String>,
    signers: Vec<Signer>,
}

impl Context {
    pub fn rpc_url(&self) -> &Url {
        &self.rpc_url
    }

    pub fn signers(&self) -> &[Signer] {
        &self.signers
    }

    pub fn private_keys(&self) -> &[String] {
        &self.private_keys
    }
}

pub fn build_context() -> Context {
    let rpc_url = std::env::var(RPC_URL)
        .expect("failed to load RPC_URL var from env")
        .parse()
        .expect("failed to parse RPC_URL string into a URL");

    let private_keys = vec![ALICE_PRIV_KEY.to_owned(), BOB_PRIV_KEY.to_owned()];
    let private_keys = private_keys
        .iter()
        .map(|s| load_var(&s))
        .collect::<eyre::Result<Vec<String>>>()
        .expect("failed to load private keys");

    let signers: Vec<Signer> = private_keys
        .iter()
        .map(|pk| {
            get_signer_from_env(pk, &rpc_url)
                .expect("failed to get signer from env")
        })
        .collect();

    Context { rpc_url, private_keys, signers }
}

fn load_var(name: &str) -> eyre::Result<String> {
    std::env::var(name).wrap_err(format!("failed to load {name}"))
}

fn get_signer_from_env(var: &str, rpc_url: &Url) -> eyre::Result<Signer> {
    let signer = var.parse::<LocalWallet>()?;
    let signer = ProviderBuilder::new()
        .with_recommended_fillers()
        .signer(EthereumSigner::from(signer))
        .on_http(rpc_url.clone());
    Ok(signer)
}
