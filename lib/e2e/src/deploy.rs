use std::{
    path::Path,
    process::Command,
    str::{from_utf8, FromStr},
};

use alloy::{
    primitives::TxHash,
    providers::ProviderBuilder,
    rpc::{
        client::RpcClient,
        types::{request, TransactionReceipt},
    },
    signers::k256::{self, ecdsa::SigningKey, Secp256k1},
    sol_types::SolConstructor,
    transports::{http::reqwest, RpcError, TransportErrorKind},
};
use eyre::{Context, ContextCompat};
use koba::config::Deploy;
use regex::Regex;

use crate::project::Crate;

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
    pub fn with_constructor<C: SolConstructor + Send>(
        mut self,
        constructor: C,
    ) -> Deployer {
        self.ctr_args = Some(alloy::hex::encode(constructor.abi_encode()));
        self
    }

    /// Add the default constructor to the deployer.
    pub fn with_default_constructor<C: SolConstructor + Send + Default>(
        self,
    ) -> Deployer {
        self.with_constructor(C::default())
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
        let pkg = Crate::new()?;
        let wasm_path = pkg.wasm;
        let sol_path = pkg.manifest_dir.join("src/constructor.sol");
        let sol =
            if Path::new(&sol_path).exists() { Some(sol_path) } else { None };

        let config = Deploy {
            generate_config: koba::config::Generate {
                wasm: wasm_path.clone(),
                sol,
                args: self.ctr_args,
                legacy: false,
            },
            auth: koba::config::PrivateKey {
                private_key_path: None,
                private_key: Some(self.private_key),
                keystore_path: None,
                keystore_password_path: None,
            },
            endpoint: self.rpc_url,
            deploy_only: false,
            quiet: false,
        };

        // koba::deploy(&config).await
        let signer = config.auth.wallet()?;
        let output = Command::new("cargo")
            .args(["stylus", "deploy"])
            .args(["-e", &self.rpc_url])
            .args(["--private-key", &self.private_key])
            .args(["--args", &self.ctr_args.unwrap()])
            .output()
            .context("failed to execute `cargo stylus cache bid` command")?;

        // output.
        get_receipt(output, self.rpc_url, signer).await
    }
}

async fn get_receipt(
    output: std::process::Output,
    rpc_url: String,
    signer: alloy::signers::local::LocalSigner<SigningKey<Secp256k1>>,
) -> eyre::Result<TransactionReceipt> {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(rpc_url);

    // // Convert output to string
    // let output_str = from_utf8(&output.stdout)
    //     .context("Failed to convert output to UTF-8")?;

    // // Extract transaction hash using regex
    // // The pattern matches a 0x followed by 64 hex characters
    // let tx_hash_regex = Regex::new(r"0x[a-fA-F0-9]{64}")
    //     .context("Failed to create regex")?;

    // let tx_hash = tx_hash_regex
    //     .find(output_str)
    //     .context("No transaction hash found in output")?
    //     .as_str();

    // // Convert string to TxHash
    // let tx_hash = TxHash::from_str(tx_hash)
    //     .context("Failed to parse transaction hash")?;

    // // Create RPC client
    // let client =
    //     RpcClient::new_http(reqwest::Url::parse(&self.rpc_url).unwrap());

    // // Get transaction receipt
    // let receipt = client
    //     .request("eth_getTransactionReceipt", [tx_hash])
    //     .await
    //     .map_err(|e: RpcError<TransportErrorKind>| {
    //         eyre::eyre!("RPC error: {}", e)
    //     })?
    //     .ok_or_else(|| eyre::eyre!("Transaction receipt not found"))?;

    // Ok(receipt)
}
