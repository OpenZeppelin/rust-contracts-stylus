use std::{process::Command, str::FromStr};

use alloy::{
    network::EthereumWallet,
    primitives::TxHash,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionReceipt,
    signers::k256::ecdsa::SigningKey,
    transports::{http::reqwest::Url, RpcError, TransportErrorKind},
};
use eyre::{Context, ContextCompat};
use regex::Regex;

/// A basic smart contract deployer.
pub struct Deployer {
    rpc_url: String,
    private_key: String,
    ctr_args: Option<String>,
}

impl Deployer {
    pub fn new(rpc_url: String, private_key: String) -> Self {
        Self { rpc_url, private_key, ctr_args: None }
    }

    /// Add solidity constructor to the deployer.
    #[allow(clippy::needless_pass_by_value)]
    pub fn with_constructor(mut self, constructor: String) -> Deployer {
        self.ctr_args = Some(constructor);
        self
    }

    /// Deploy and activate the contract implemented as `#[entrypoint]` in the
    /// current crate.
    /// Consumes currently configured deployer.
    ///
    /// # Errors
    ///
    /// May error if:
    ///
    /// - Unable to collect information about the crate required for deployment.
    /// - [`koba::deploy`] errors.
    pub async fn deploy(self) -> eyre::Result<TransactionReceipt> {
        let auth = koba::config::PrivateKey {
            private_key_path: None,
            private_key: Some(self.private_key.clone()),
            keystore_path: None,
            keystore_password_path: None,
        };

        // koba::deploy(&config).await
        let signer = auth.wallet()?;
        let output = Command::new("cargo")
            .args(["stylus", "deploy"])
            .args(["-e", &self.rpc_url])
            .args(["--private-key", &self.private_key])
            .args(["--experimental-constructor-args", &self.ctr_args.unwrap()])
            .args(["--no-verify"])
            .output()
            .context("failed to execute `cargo stylus deploy` command")?;

        // output.
        get_receipt(output, self.rpc_url, signer).await
    }
}

async fn get_receipt(
    output: std::process::Output,
    rpc_url: String,
    signer: alloy::signers::local::LocalSigner<SigningKey>,
) -> eyre::Result<TransactionReceipt> {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(Url::from_str(&rpc_url).expect("invalid Url"));

    // Convert output to string
    let output_str = String::from_utf8_lossy(&output.stderr);

    // Extract transaction hash using regex
    // The pattern matches a 0x followed by 64 hex characters
    let tx_hash_regex =
        Regex::new(r"0x[a-fA-F0-9]{64}").context("Failed to create regex")?;

    let tx_hash = tx_hash_regex
        .find(&*output_str)
        .context(format!("No transaction hash found in output {output_str}"))?
        .as_str();

    // Convert string to TxHash
    let tx_hash = TxHash::from_str(tx_hash)
        .context("Failed to parse transaction hash")?;

    let receipt = provider
        .get_transaction_receipt(tx_hash)
        .await
        .map_err(|e: RpcError<TransportErrorKind>| {
            eyre::eyre!("RPC error: {}", e)
        })?
        .ok_or_else(|| eyre::eyre!("Transaction receipt not found"))?;

    Ok(receipt)
}
