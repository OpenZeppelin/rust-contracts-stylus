use std::{path::PathBuf, process::Command, str::FromStr};

use alloy::{
    consensus::Transaction,
    hex::{self, ToHexExt},
    network::EthereumWallet,
    primitives::{Address, TxHash},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    transports::{http::reqwest::Url, RpcError, TransportErrorKind},
};
use eyre::{Context, ContextCompat};
use regex::Regex;
use stylus_sdk::{abi::Bytes, alloy_primitives, function_selector};

use crate::{
    project::{get_wasm, Crate},
    system::DEPLOYER_ADDRESS,
    Constructor, Receipt,
};

const CONTRACT_INITIALIZATION_ERROR_SELECTOR: [u8; 4] =
    function_selector!("ContractInitializationError", Address, Bytes);

const PROGRAM_UP_TO_DATE_ERROR_SELECTOR: [u8; 4] =
    function_selector!("ProgramUpToDate");

const CONTRACT_DEPLOYMENT_ERROR_SELECTOR: [u8; 4] =
    function_selector!("ContractDeploymentError", Bytes);

/// Represents the `ContractInitializationError(address)` error in
/// StylusDeployer.
///
/// This error is returned when a revert happens inside the contract
/// constructor. The StylusDeployer contract then returns this error, which
/// contains the would-be address of the contract.
///
/// See: <https://github.com/OffchainLabs/nitro-contracts/blob/c32af127fe6a9124316abebbf756609649ede1f5/src/stylus/StylusDeployer.sol#L78-L81>
#[derive(Debug)]
pub struct ContractInitializationError;

impl ContractInitializationError {
    /// Convert [`eyre::Report`] into [`ContractInitializationError`].
    pub fn from_report(report: &eyre::Report) -> Option<&Self> {
        report.downcast_ref::<ContractInitializationError>()
    }
}

impl std::fmt::Display for ContractInitializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ContractInitializationError")
    }
}

impl std::error::Error for ContractInitializationError {}

/// Represents the `ContractDeploymentError(bytes)` error in StylusDeployer.
///
/// See: <https://github.com/OffchainLabs/nitro-contracts/blob/c32af127fe6a9124316abebbf756609649ede1f5/src/stylus/StylusDeployer.sol#L15>
#[derive(Debug)]
pub struct ContractDeploymentError {
    /// Contract bytecode.
    pub bytecode: alloy_primitives::Bytes,
}

impl ContractDeploymentError {
    /// Convert [`eyre::Report`] into [`ContractDeploymentError`].
    pub fn from_report(report: &eyre::Report) -> Option<&Self> {
        report.downcast_ref::<ContractDeploymentError>()
    }
}

impl std::fmt::Display for ContractDeploymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ContractDeploymentError {")
    }
}

impl std::error::Error for ContractDeploymentError {}

/// A basic smart contract deployer.
pub struct Deployer {
    rpc_url: String,
    private_key: String,
    ctor: Option<Constructor>,
}

impl Deployer {
    pub fn new(rpc_url: String, private_key: String) -> Self {
        Self { rpc_url, private_key, ctor: None }
    }

    /// Sets the constructor to be used during contract deployment.
    #[allow(clippy::needless_pass_by_value)]
    pub fn with_constructor(mut self, ctor: Constructor) -> Deployer {
        self.ctor = Some(ctor);
        self
    }

    /// See [`Deployer::deploy_wasm()`] for more details.
    pub async fn deploy(self) -> eyre::Result<Receipt> {
        let pkg = Crate::new()?;
        let wasm_path = pkg.wasm;

        self.deploy_wasm(&wasm_path).await
    }

    /// See [`Deployer::deploy_wasm()`] for more details.
    pub async fn deploy_from_example(
        self,
        example_name: &str,
    ) -> eyre::Result<Receipt> {
        let wasm_path = get_wasm(format!("{example_name}_example").as_str())?;

        self.deploy_wasm(&wasm_path).await
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
    /// - `cargo stylus deploy` errors.
    pub async fn deploy_wasm(
        self,
        wasm_path: &PathBuf,
    ) -> eyre::Result<Receipt> {
        let wasm_path = wasm_path.to_str().expect("wasm file should exist");
        let mut command = self.create_command(wasm_path);

        let output = command
            .output()
            .context("failed to execute `cargo stylus deploy` command")?;

        // Resources for context on the implementation:
        // - https://github.com/OffchainLabs/nitro-contracts/blob/c32af127fe6a9124316abebbf756609649ede1f5/src/stylus/StylusDeployer.sol#L10
        // - https://github.com/OffchainLabs/nitro/blob/98aefbacd814b002bd93a625edaaa0abd9e0d2f0/arbos/programs/programs.go#L113
        if !output.status.success() {
            self.parse_deployment_error(output).await
        } else {
            self.get_receipt(output).await
        }
    }

    fn create_command(&self, wasm_path: &str) -> Command {
        let mut command = Command::new("cargo");
        command
            .args(["stylus", "deploy"])
            .args(["-e", &self.rpc_url])
            .args(["--private-key", &self.private_key])
            .args(["--wasm-file", wasm_path])
            .args(["--no-verify"]);

        // There are 3 possible cases when it comes to invoking constructors:
        //      1. No constructor exists on a contract - `self.ctor_args` should
        //         be None
        //      2. Constructor exists, but accepts no arguments -
        //         `self.ctor_args` should be Some(vec![])
        //      3. Constructor exists and accepts arguments - `self.ctor_args`
        //         should be Some(vec!["arg1", "arg2", ...])
        //
        // The deployer address and constructor-args must both be set if the
        // constructor is to be invoked on a contract. Otherwise,
        // neither should be set.
        if let Some(ctor) = self.ctor.as_ref() {
            let deployer_address = std::env::var(DEPLOYER_ADDRESS)
                .expect("deployer address should be set");

            command
                .args(["--deployer-address", &deployer_address])
                .args(["--constructor-signature", &ctor.signature])
                .args(
                    [&["--constructor-args".to_string()], ctor.args.as_slice()]
                        .concat(),
                );
        }

        command
    }

    /// These are all band-aid solutions for peculiar nitro-testnode behavior
    /// (as of commit de8cf4edec0d12e5ef1b7623e54e35ddb579ff0b on branch
    /// "v3-support").
    async fn parse_deployment_error(
        &self,
        output: std::process::Output,
    ) -> eyre::Result<Receipt> {
        let stderr = &String::from_utf8_lossy(&output.stderr);

        // Look for the error pattern with hex data
        let error_data_regex = Regex::new(r#"data:.+"(0x[a-fA-F0-9]+)""#)
            .context("failed to create error data regex")?;

        if let Some(captures) = error_data_regex.captures(stderr) {
            if let Some(hex_data) = captures.get(1) {
                let hex_str = hex_data.as_str();
                let hex_str = &hex_str[2..]; // Skip "0x" prefix
                let data = hex::decode(hex_str)
                    .context(format!("failed to decode hex: {hex_str}"))?;
                let error_selector = &data[0..4];

                if error_selector == CONTRACT_INITIALIZATION_ERROR_SELECTOR {
                    return Err(eyre::Report::new(
                        ContractInitializationError {},
                    ));
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
                } else if error_selector == CONTRACT_DEPLOYMENT_ERROR_SELECTOR {
                    return Err(eyre::Report::new(ContractDeploymentError {
                        bytecode: data[4..].to_vec().into(),
                    }));
                } else {
                    return Err(eyre::eyre!(hex_str.to_string()));
                }
            }
        }

        // The pattern matches the contract address that is preceeded by
        // ANSI escape codes (`cargo stylus deploy` outputs colored text).
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

                // We extract the address of the contract that was supposed to
                // be activated by StylusDeployer by getting the transaction
                // that StylusDeployer sent to Arbitrum `activateProgram`
                // precompile and extracting the address used in the transaction
                // input.

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

                // now that we have the contract address, we only need the
                // receipt
                let receipt = provider
                    .get_transaction_receipt(tx_hash)
                    .await
                    .map_err(|e: RpcError<TransportErrorKind>| {
                        eyre::eyre!("RPC error: {}", e)
                    })?
                    .ok_or_else(|| {
                        eyre::eyre!("Transaction receipt not found")
                    })?;

                return Ok(Receipt { inner: receipt, contract_address });
            }
        }

        Err(eyre::eyre!("Deployment failed: {}", stderr))
    }

    /// Constructs the receipt struct extracting the necessary receipt and
    /// contract address data out of the `cargo stylus deploy` output with the
    /// help of regex.
    async fn get_receipt(
        &self,
        output: std::process::Output,
    ) -> eyre::Result<Receipt> {
        // Convert output to string
        let output_str = String::from_utf8_lossy(&output.stdout);

        // first we get the contract address

        // The pattern matches the contract address that is preceeded by
        // ANSI escape codes (`cargo stylus deploy` outputs colored text).
        let contract_addr_regex =
            Regex::new(r"deployed code at address:\s*(?:\x1B\[[0-9;]*[a-zA-Z])*(0x[a-fA-F0-9]{40})")
                .context("Failed to create contract addr regex")?;

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

        // Now we extract the transaction hash to fetch to receipt

        let tx_hash_regex = Regex::new(r"0x[a-fA-F0-9]{64}")
            .context("Failed to create tx hash regex")?;

        let tx_hash = tx_hash_regex
            .find(&*output_str)
            .context(format!(
                "No transaction hash found in output {output_str}"
            ))?
            .as_str();
        let tx_hash = TxHash::from_str(tx_hash)
            .context("Failed to parse transaction hash")?;

        // Finally we can fetch the receipt

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

        Ok(Receipt { inner: receipt, contract_address })
    }
}
