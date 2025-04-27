use std::{process::Command, str::FromStr};

use alloy::{
    consensus::Transaction,
    hex::{self, ToHexExt},
    network::EthereumWallet,
    primitives::{Address, TxHash},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionReceipt,
    signers::local::PrivateKeySigner,
    transports::{http::reqwest::Url, RpcError, TransportErrorKind},
};
use eyre::{Context, ContextCompat};
use regex::Regex;
use stylus_sdk::function_selector;

use crate::system::DEPLOYER_ADDRESS;

const CONTRACT_INITIALIZATION_ERROR_SELECTOR: [u8; 4] =
    function_selector!("ContractInitializationError", Address);

const PROGRAM_UP_TO_DATE_ERROR_SELECTOR: [u8; 4] =
    function_selector!("ProgramUpToDate");

/// StylusDeployer error.
///
/// Currently only supports `error ContractInitializationError(address)`.
#[derive(Debug)]
pub struct StylusDeployerError {
    /// Deployed contract address.
    pub contract_address: Address,
    /// Hex encoded revert data.
    pub revert_data: String,
}

impl StylusDeployerError {
    /// Convert [`eyre::Report`] into [`StylusDeployerError`].
    pub fn from_report(report: &eyre::Report) -> Option<&Self> {
        report.downcast_ref::<StylusDeployerError>()
    }
}

impl std::fmt::Display for StylusDeployerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.revert_data)
    }
}

impl std::error::Error for StylusDeployerError {}

/// A basic smart contract deployer.
pub struct Deployer {
    rpc_url: String,
    private_key: String,
    ctr_args: Option<Vec<String>>,
}

impl Deployer {
    pub fn new(rpc_url: String, private_key: String) -> Self {
        Self { rpc_url, private_key, ctr_args: None }
    }

    /// Add solidity constructor to the deployer.
    #[allow(clippy::needless_pass_by_value)]
    pub fn with_constructor(mut self, ctr_args: Vec<String>) -> Deployer {
        self.ctr_args = Some(ctr_args);
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
    pub async fn deploy(self) -> eyre::Result<(TransactionReceipt, Address)> {
        let mut command = Command::new("cargo");
        command
            .args(["stylus", "deploy"])
            .args(["-e", &self.rpc_url])
            .args(["--private-key", &self.private_key])
            .args(["--no-verify"]);

        if let Some(ctor_args) = self.ctr_args.clone() {
            let deployer_address = std::env::var(DEPLOYER_ADDRESS)
                .expect("deployer address should be set");

            command
                .args(["--experimental-deployer-address", &deployer_address])
                .args(
                    [
                        vec!["--experimental-constructor-args".to_string()],
                        ctor_args,
                    ]
                    .concat(),
                );
        }

        let output = command
            .output()
            .context("failed to execute `cargo stylus deploy` command")?;

        // Check if the command failed
        if !output.status.success() {
            self.parse_deployment_error(output).await
        } else {
            self.get_receipt(output).await
        }
    }

    /// These are all band-aid solutions for peculiar nitro-testnode behavior
    /// (as of commit de8cf4edec0d12e5ef1b7623e54e35ddb579ff0b on branch
    /// "v3-support").
    async fn parse_deployment_error(
        &self,
        output: std::process::Output,
    ) -> eyre::Result<(TransactionReceipt, Address)> {
        let stderr = &String::from_utf8_lossy(&output.stderr);

        // Look for the error pattern with hex data
        let contract_init_error_regex =
            Regex::new(r#"data: Some\(String\("(0x[a-fA-F0-9]+)"\)\)"#)
                .context(
                    "failed to create ContractInitializationError regex",
                )?;

        if let Some(captures) = contract_init_error_regex.captures(stderr) {
            if let Some(hex_data) = captures.get(1) {
                let hex_str = hex_data.as_str();
                let hex_str = &hex_str[2..]; // Skip "0x" prefix
                let data = hex::decode(hex_str)
                    .context(format!("failed to decode hex: {hex_str}"))?;
                let error_selector = &data[0..4];

                if error_selector == CONTRACT_INITIALIZATION_ERROR_SELECTOR {
                    // Extract the address (last 20 bytes)
                    let address_bytes = &data[16..36];
                    let contract_address = Address::from_slice(address_bytes);

                    let deployment_error = StylusDeployerError {
                        contract_address,
                        revert_data: hex_str.to_string(),
                    };

                    return Err(eyre::Report::new(deployment_error));
                } else if error_selector == PROGRAM_UP_TO_DATE_ERROR_SELECTOR {
                    // For some reason, the error here is:
                    // ```
                    // did not estimate correctly: (code: 3, message: execution reverted: error ProgramUpToDate(), data: Some(String(\"0xcc944bf2\")))
                    // ```
                    // which is the same as the one handles later (the
                    // activation error), but it still contains the stdout that
                    // is the same as if the deployment was successful.
                    //
                    // This is probably some weird nitro-testnode issue, but for
                    // now this quick-fix should work.
                    return self.get_receipt(output).await;
                } else {
                    return Err(eyre::eyre!(hex_str.to_string()));
                }
            }
        }

        let activation_error_regex =
            Regex::new(r#"activate tx reverted (?:\x1B\[[0-9;]*[a-zA-Z])*(0x[a-fA-F0-9]+)"#)
                .context("failed to create activation error regex")?;

        if let Some(captures) = activation_error_regex.captures(stderr) {
            if let Some(tx_hash_match) = captures.get(1) {
                let tx_hash = tx_hash_match.as_str();

                let tx_hash = TxHash::from_str(tx_hash)
                    .context("Failed to parse transaction hash")?;

                let provider = ProviderBuilder::new()
                    .with_recommended_fillers()
                    .wallet(EthereumWallet::from(
                        self.private_key.parse::<PrivateKeySigner>()?,
                    ))
                    .on_http(
                        Url::from_str(&self.rpc_url).expect("invalid Url"),
                    );

                let tx = provider
                    .get_transaction_by_hash(tx_hash)
                    .await
                    .map_err(|e: RpcError<TransportErrorKind>| {
                        eyre::eyre!("RPC error: {}", e)
                    })?
                    .ok_or_else(|| {
                        eyre::eyre!("Transaction receipt not found")
                    })?;

                let input = tx.input().encode_hex();

                // The pattern matches the contract address contained in the
                // input
                let contract_addr_regex =
                    Regex::new(r"[a-fA-F0-9]{8}0+([a-fA-F0-9]{40})$")
                        .context("Failed to create contract addr regex")?;

                let contract_addr = contract_addr_regex
                    .captures(&input)
                    .and_then(|cap| cap.get(1))
                    .context(format!(
                        "No contract address found in input {input}"
                    ))?
                    .as_str();

                let contract_address = Address::from_str(contract_addr)
                    .context(format!(
                        "Failed to parse contract address from string: {contract_addr}"
                    ))?;

                let receipt = provider
                    .get_transaction_receipt(tx_hash)
                    .await
                    .map_err(|e: RpcError<TransportErrorKind>| {
                        eyre::eyre!("RPC error: {}", e)
                    })?
                    .ok_or_else(|| {
                        eyre::eyre!("Transaction receipt not found")
                    })?;

                return Ok((receipt, contract_address));
            }
        }

        Err(eyre::eyre!("Deployment failed: {}", stderr))
    }

    async fn get_receipt(
        &self,
        output: std::process::Output,
    ) -> eyre::Result<(TransactionReceipt, Address)> {
        // Convert output to string
        let output_str = String::from_utf8_lossy(&output.stdout);

        // Extract transaction hash using regex
        // The pattern matches a 0x followed by 64 hex characters
        let tx_hash_regex = Regex::new(r"0x[a-fA-F0-9]{64}")
            .context("Failed to create tx hash regex")?;

        // The pattern matches the contract address that is preceeded by
        // ANSI escape codes (`cargo stylus deploy` outputs
        // colored text).
        let contract_addr_regex =
            Regex::new(r"deployed code at address:\s*(?:\x1B\[[0-9;]*[a-zA-Z])*(0x[a-fA-F0-9]{40})")
                .context("Failed to create contract addr regex")?;

        let tx_hash = tx_hash_regex
            .find(&*output_str)
            .context(format!(
                "No transaction hash found in output {output_str}"
            ))?
            .as_str();

        let contract_addr = contract_addr_regex
            .captures(&output_str)
            .and_then(|cap| cap.get(1))
            .context(format!(
                "No contract address found in output {output_str}"
            ))?
            .as_str();
        let contract_address =
            Address::from_str(contract_addr).context(format!(
                "Failed to parse contract address from string: {contract_addr}"
            ))?;

        // Convert string to TxHash
        let tx_hash = TxHash::from_str(tx_hash)
            .context("Failed to parse transaction hash")?;

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(
                self.private_key.parse::<PrivateKeySigner>()?,
            ))
            .on_http(Url::from_str(&self.rpc_url).expect("invalid Url"));

        let receipt = provider
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|e: RpcError<TransportErrorKind>| {
                eyre::eyre!("RPC error: {}", e)
            })?
            .ok_or_else(|| eyre::eyre!("Transaction receipt not found"))?;

        Ok((receipt, contract_address))
    }
}
