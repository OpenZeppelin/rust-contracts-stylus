use std::{env, fs, path::Path};

use alloy::{
    primitives::{Address, Bytes, B256, U256},
    providers::WalletProvider,
    sol,
    sol_types::{SolCall, SolError},
    transports::RpcError,
};
use eyre::Context;
use stylus_tools::core::{
    activation::{activate_contract, ActivationConfig},
    check::check_wasm_file,
    code::Code,
    deployment::{
        deployer::{
            get_address_from_receipt, stylus_constructorCall, StylusDeployer,
        },
        fragments::deploy_fragments,
        prelude::DeploymentCalldata,
        request::DeploymentRequest,
        DeploymentConfig, DeploymentError,
    },
    project::contract::ContractStatus,
};

use crate::{
    account::{Account, Wallet},
    devnet::{target_dir, STYLUS_DEPLOYER},
    Constructor, Receipt,
};

sol! {
    /// Errors of the `StylusDeployer` contract.
    ///
    /// See: <https://github.com/OffchainLabs/nitro-contracts/blob/main/src/stylus/StylusDeployer.sol>
    #[derive(Debug)]
    #[allow(missing_docs)]
    interface StylusDeployerErrors {
        error ContractDeploymentError(bytes bytecode);
        error ContractInitializationError(address newContract);
    }
}

/// Represents the `ContractInitializationError(address)` error in
/// `StylusDeployer`.
///
/// This error is returned when a revert happens inside the contract
/// constructor. The `StylusDeployer` contract then returns this error, which
/// contains the would-be address of the contract.
#[derive(Debug)]
pub struct ContractInitializationError {
    /// The address the contract would have been deployed to.
    pub contract_address: Address,
}

impl ContractInitializationError {
    /// Convert [`eyre::Report`] into [`ContractInitializationError`].
    #[must_use]
    pub fn from_report(report: &eyre::Report) -> Option<&Self> {
        report.downcast_ref::<ContractInitializationError>()
    }
}

impl std::fmt::Display for ContractInitializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ContractInitializationError: the constructor of {} reverted",
            self.contract_address
        )
    }
}

impl std::error::Error for ContractInitializationError {}

/// Represents the `ContractDeploymentError(bytes)` error in `StylusDeployer`.
#[derive(Debug)]
pub struct ContractDeploymentError {
    /// Contract bytecode.
    pub bytecode: Bytes,
}

impl ContractDeploymentError {
    /// Convert [`eyre::Report`] into [`ContractDeploymentError`].
    #[must_use]
    pub fn from_report(report: &eyre::Report) -> Option<&Self> {
        report.downcast_ref::<ContractDeploymentError>()
    }
}

impl std::fmt::Display for ContractDeploymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ContractDeploymentError: the contract could not be created")
    }
}

impl std::error::Error for ContractDeploymentError {}

/// Deploys compiled Stylus contracts to the devnet on behalf of an
/// [`Account`].
///
/// Contracts without a constructor are created directly and then activated.
/// Contracts with a constructor are created, activated and initialized in
/// one transaction through the `StylusDeployer` contract, exactly like
/// `cargo stylus deploy` does. Contracts above the code size limit are
/// deployed as fragments (`ArbOS` 60 and later).
pub struct Deployer {
    account: Account,
    ctor: Option<Constructor>,
    example_name: Option<String>,
}

impl Deployer {
    pub(crate) fn new(account: Account) -> Self {
        Self { account, ctor: None, example_name: None }
    }

    /// Sets the constructor to be used during contract deployment.
    #[must_use]
    pub fn with_constructor(mut self, ctor: Constructor) -> Self {
        self.ctor = Some(ctor);
        self
    }

    /// Sets the example to deploy, by the name of its directory under
    /// `examples/` (for example `"erc20"`), instead of the crate the test
    /// belongs to.
    #[must_use]
    pub fn with_example_name(mut self, example_name: &str) -> Self {
        self.example_name = Some(format!("{example_name}_example"));
        self
    }

    /// Deploys the compiled WASM of the configured example, or of the crate
    /// of the current working directory, from
    /// `target/wasm32-unknown-unknown/release`.
    ///
    /// See [`Deployer::deploy_wasm`].
    ///
    /// # Errors
    ///
    /// May error if the crate name cannot be read from `Cargo.toml` or the
    /// deployment fails.
    pub async fn deploy(&self) -> eyre::Result<Receipt> {
        let name = match &self.example_name {
            Some(name) => name.clone(),
            None => read_pkg_name(env::current_dir()?)?,
        };
        let wasm = target_dir()
            .join("wasm32-unknown-unknown")
            .join("release")
            .join(format!("{}.wasm", name.replace('-', "_")));
        self.deploy_wasm(&wasm).await
    }

    /// Deploys and activates the compiled contract at `wasm_path`.
    ///
    /// # Errors
    ///
    /// * [`ContractInitializationError`] - If the constructor reverted.
    /// * [`ContractDeploymentError`] - If the `StylusDeployer` could not
    ///   create the contract.
    /// * Any other deployment, activation or RPC error.
    pub async fn deploy_wasm(&self, wasm_path: &Path) -> eyre::Result<Receipt> {
        let provider = &self.account.wallet;
        let config = DeploymentConfig {
            deployer_address: STYLUS_DEPLOYER,
            ..Default::default()
        };
        let status = check_wasm_file(
            wasm_path,
            Default::default(),
            None,
            None,
            &config.check,
            provider,
        )
        .await
        .wrap_err_with(|| {
            format!("failed to check {}", wasm_path.display())
        })?;
        let data_fee = status.suggest_fee();

        let code = match status.code() {
            Code::Contract(contract) => contract.0.clone(),
            Code::Fragments(fragments) => {
                let wasm_len = u32::try_from(status.wasm().len())
                    .wrap_err("the WASM is too large")?;
                deploy_fragments(fragments, wasm_len, &config, provider)
                .await
                .map_err(deployment_error)?
                .0
            }
        };

        match &self.ctor {
            None => {
                self.deploy_without_constructor(&code, &status, provider).await
            }
            Some(ctor) => {
                self.deploy_with_constructor(&code, ctor, data_fee, provider).await
            }
        }
    }

    async fn deploy_without_constructor(
        &self,
        code: &[u8],
        status: &ContractStatus,
        provider: &Wallet,
    ) -> eyre::Result<Receipt> {
        let from = provider.default_signer_address();
        let receipt = DeploymentRequest::new(from, code, None)
            .exec(provider)
            .await
            .map_err(deployment_error)?;
        let contract_address = receipt
            .contract_address
            .ok_or_else(|| eyre::eyre!("no contract address in the receipt"))?;
        if !matches!(status, ContractStatus::Active { .. }) {
            activate_contract(
                contract_address,
                &ActivationConfig::default(),
                provider,
            )
            .await
            .wrap_err("failed to activate the contract")?;
        }
        Ok(Receipt { inner: receipt, contract_address })
    }

    async fn deploy_with_constructor(
        &self,
        code: &[u8],
        ctor: &Constructor,
        data_fee: U256,
        provider: &Wallet,
    ) -> eyre::Result<Receipt> {
        let mut init_data = stylus_constructorCall::SELECTOR.to_vec();
        init_data.extend_from_slice(&ctor.args);
        let calldata = StylusDeployer::deployCall {
            bytecode: DeploymentCalldata::new(code).into(),
            initData: init_data.into(),
            initValue: U256::ZERO,
            salt: B256::ZERO,
        }
        .abi_encode();
        let from = provider.default_signer_address();
        let receipt = DeploymentRequest::new_with_args(
            from,
            STYLUS_DEPLOYER,
            data_fee,
            calldata,
            None,
        )
        .exec(provider)
        .await
        .map_err(deployment_error)?;
        let contract_address = get_address_from_receipt(&receipt)
            .map_err(|e| eyre::eyre!("{e}"))?;
        Ok(Receipt { inner: receipt, contract_address })
    }
}

/// Converts a deployment error into a report, decoding the `StylusDeployer`
/// errors into [`ContractInitializationError`] and
/// [`ContractDeploymentError`].
fn deployment_error(error: DeploymentError) -> eyre::Report {
    if let DeploymentError::Rpc(RpcError::ErrorResp(payload)) = &error {
        if let Some(data) = payload.data.as_ref() {
            let hex = data.get().trim_matches('"');
            if let Ok(data) = alloy::hex::decode(hex) {
                if let Ok(err) =
                    StylusDeployerErrors::ContractInitializationError::abi_decode(
                        &data,
                    )
                {
                    return eyre::Report::new(ContractInitializationError {
                        contract_address: err.newContract,
                    });
                }
                if let Ok(err) =
                    StylusDeployerErrors::ContractDeploymentError::abi_decode(
                        &data,
                    )
                {
                    return eyre::Report::new(ContractDeploymentError {
                        bytecode: err.bytecode,
                    });
                }
            }
        }
    }
    eyre::Report::new(error)
}

/// Reads and parses the package name from a manifest in `path`.
fn read_pkg_name<P: AsRef<Path>>(path: P) -> eyre::Result<String> {
    let cargo_toml = path.as_ref().join("Cargo.toml");
    let manifest = fs::read_to_string(&cargo_toml)
        .wrap_err_with(|| format!("failed to read {}", cargo_toml.display()))?;
    let table = manifest.parse::<toml::Table>()?;
    table
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(str::to_owned)
        .ok_or_else(|| eyre::eyre!("unable to find package name in toml"))
}
