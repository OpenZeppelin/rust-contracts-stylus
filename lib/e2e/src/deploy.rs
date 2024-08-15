use std::path::PathBuf;

use alloy::{rpc::types::TransactionReceipt, sol_types::SolConstructor};
use async_trait::async_trait;
use koba::config::Deploy;

use crate::project::Crate;

/// Deploy and activate the contract implemented as `#[entrypoint]` in the
/// current crate.
///
/// # Arguments
/// * `rpc_url` - The RPC URL of the network to deploy to.
/// * `private_key` - The private key of the account to deploy from.
/// * `wasm_path` - The path to the contract's wasm file.
/// * `ctr_args` - Optional ABI-encoded constructor arguments.
/// * `sol_path` - Optional path to the contract's solidity constructor.
async fn deploy_inner(
    rpc_url: String,
    private_key: String,
    wasm_path: PathBuf,
    ctr_args: Option<String>,
    sol_path: Option<PathBuf>,
) -> eyre::Result<TransactionReceipt> {
    let config = Deploy {
        generate_config: koba::config::Generate {
            wasm: wasm_path.clone(),
            sol: sol_path,
            args: ctr_args,
            legacy: false,
        },
        auth: koba::config::PrivateKey {
            private_key_path: None,
            private_key: Some(private_key),
            keystore_path: None,
            keystore_password_path: None,
        },
        endpoint: rpc_url,
        deploy_only: false,
        quiet: false,
    };

    let receipt = koba::deploy(&config).await?;
    Ok(receipt)
}

/// Abstraction for configured deployer.
#[async_trait]
pub trait ContractDeployer {
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
    async fn deploy(self) -> eyre::Result<TransactionReceipt>;
}

/// A basic smart contract deployer.
pub struct Deployer {
    rpc_url: String,
    private_key: String,
}

impl Deployer {
    pub fn new(rpc_url: String, private_key: String) -> Self {
        Self { rpc_url, private_key }
    }

    /// Add solidity constructor to the deployer.
    pub fn with_constructor<C: SolConstructor + Send>(
        self,
        constructor: C,
    ) -> DeployerWithConstructor<C> {
        DeployerWithConstructor { deployer: self, constructor }
    }
}

#[async_trait]
impl ContractDeployer for Deployer {
    async fn deploy(self) -> eyre::Result<TransactionReceipt> {
        let pkg = Crate::new()?;
        let wasm_path = pkg.wasm;
        deploy_inner(self.rpc_url, self.private_key, wasm_path, None, None)
            .await
    }
}

/// A smart contract deployer with solidity constructor compiled with `koba`.
pub struct DeployerWithConstructor<C: SolConstructor + Send> {
    deployer: Deployer,
    constructor: C,
}

#[async_trait]
impl<C: SolConstructor + Send> ContractDeployer for DeployerWithConstructor<C> {
    async fn deploy(self) -> eyre::Result<TransactionReceipt> {
        let pkg = Crate::new()?;
        let wasm_path = pkg.wasm;
        let args = alloy::hex::encode(self.constructor.abi_encode());
        let sol_path = pkg.manifest_dir.join("src/constructor.sol");
        deploy_inner(
            self.deployer.rpc_url,
            self.deployer.private_key,
            wasm_path,
            Some(args),
            Some(sol_path),
        )
        .await
    }
}
