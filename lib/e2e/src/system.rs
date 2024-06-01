use std::{path::PathBuf, process::Command};

use alloy::{
    network::{Ethereum, EthereumSigner},
    providers::{
        fillers::{
            ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
            SignerFiller,
        },
        Identity, ProviderBuilder, RootProvider,
    },
    transports::http::{Client, Http},
};
use eyre::Context as Ctx;

pub const RPC_URL_ENV_VAR_NAME: &str = "RPC_URL";

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

/// Load the `name` environment variable.
pub fn env(name: &str) -> eyre::Result<String> {
    std::env::var(name).wrap_err(format!("failed to load {name}"))
}

/// Returns an alloy provider connected to the `RPC_URL` rpc endpoint.
pub fn provider() -> Provider {
    let rpc_url = std::env::var(RPC_URL_ENV_VAR_NAME)
        .expect("failed to load RPC_URL var from env")
        .parse()
        .expect("failed to parse RPC_URL string into a URL");
    ProviderBuilder::new().with_recommended_fillers().on_http(rpc_url)
}

/// Runs the following command to get the worskpace root:
///
/// ```bash
/// dirname "$(cargo locate-project --workspace --message-format plain)"
/// ```
pub(crate) fn get_workspace_root() -> eyre::Result<PathBuf> {
    let output = Command::new("cargo")
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format")
        .arg("plain")
        .output()
        .wrap_err("should run `cargo locate-project`")?;

    let manifest_path = String::from_utf8_lossy(&output.stdout);
    let manifest_dir = Command::new("dirname")
        .arg(&*manifest_path)
        .output()
        .wrap_err("should run `dirname`")?;

    let path = String::from_utf8_lossy(&manifest_dir.stdout)
        .trim()
        .to_string()
        .parse::<PathBuf>()
        .wrap_err("failed to parse manifest dir path")?;
    Ok(path)
}

/// Get's expected path to the nitro test node.
pub(crate) fn get_node_path() -> eyre::Result<PathBuf> {
    let manifest_dir = get_workspace_root()?;
    Ok(manifest_dir.join("nitro-testnode"))
}
