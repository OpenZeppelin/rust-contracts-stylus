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
    signers: Vec<Signer>,
    rpc_url: Url,
}

impl Context {
    pub fn new(rpc_url: Url, s: &[Signer]) -> Self {
        let mut signers = Vec::with_capacity(s.len());
        signers.extend_from_slice(s);

        Self { rpc_url, signers }
    }

    pub fn rpc_endpoint(&self) -> &Url {
        &self.rpc_url
    }

    pub fn signers(&self) -> &[Signer] {
        &self.signers
    }
}

pub fn build_context() -> Context {
    let rpc_url = std::env::var(RPC_URL)
        .expect("failed to load RPC_URL var from env")
        .parse()
        .expect("failed to parse RPC_URL string into a URL");

    let signers: Vec<Signer> = vec![ALICE_PRIV_KEY, BOB_PRIV_KEY]
        .into_iter()
        .map(|pk| {
            get_signer_from_env(pk, &rpc_url)
                .expect("failed to get signer from env")
        })
        .collect();

    Context::new(rpc_url, &signers)
}

fn get_signer_from_env(var: &str, rpc_url: &Url) -> eyre::Result<Signer> {
    let signer = std::env::var(var)
        .wrap_err(format!("failed to load {var}"))?
        .parse::<LocalWallet>()?;
    let signer = ProviderBuilder::new()
        .with_recommended_fillers()
        .signer(EthereumSigner::from(signer))
        .on_http(rpc_url.clone());
    Ok(signer)
}
