use alloy::{
    network::{Ethereum, EthereumWallet},
    primitives::Address,
    providers::{
        fillers::{
            BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill,
            NonceFiller, SimpleNonceManager, WalletFiller,
        },
        Identity, RootProvider,
    },
    transports::http::reqwest::Url,
};
use eyre::bail;

use crate::environment::get_node_path;

const RPC_URL_ENV_VAR_NAME: &str = "RPC_URL";

/// Loads the rpc url from the environment variable.
pub fn get_rpc_url() -> Url {
    std::env::var(RPC_URL_ENV_VAR_NAME)
        .unwrap_or_else(|_| {
            panic!("failed to load {RPC_URL_ENV_VAR_NAME} var from env");
        })
        .parse()
        .unwrap_or_else(|_| {
            panic!("failed to parse {RPC_URL_ENV_VAR_NAME} string into a URL");
        })
}

/// Convenience type alias that represents an Ethereum wallet.
pub type Wallet = FillProvider<
    JoinFill<
        JoinFill<
            JoinFill<
                Identity,
                JoinFill<
                    GasFiller,
                    JoinFill<
                        BlobGasFiller,
                        JoinFill<NonceFiller, ChainIdFiller>,
                    >,
                >,
            >,
            NonceFiller<SimpleNonceManager>,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider,
>;

/// Send `amount` eth to `address` in the nitro-tesnode.
///
/// # Errors
///
/// May fail if unable to find the path to the node or if funding the newly
/// created account fails.
pub fn fund_account(address: Address, amount: u32) -> eyre::Result<()> {
    let node_script = get_node_path()?.join("test-node.bash");
    if !node_script.exists() {
        bail!("Test nitro node wasn't setup properly. Try to setup it first with `./scripts/nitro-testnode.sh -i -d`")
    }

    let output = std::process::Command::new(node_script)
        .arg("script")
        .arg("send-l2")
        .arg("--to")
        .arg(format!("address_{address}"))
        .arg("--ethamount")
        .arg(amount.to_string())
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("account's wallet wasn't funded - address is {address}:\n{err}")
    }

    Ok(())
}
