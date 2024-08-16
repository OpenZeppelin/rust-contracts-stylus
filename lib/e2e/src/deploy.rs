use std::path::PathBuf;

use alloy::{rpc::types::TransactionReceipt, sol_types::SolConstructor};
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
        let sol_path = if self.ctr_args.is_some() {
            Some(pkg.manifest_dir.join("src/constructor.sol"))
        } else {
            None
        };
        deploy_inner(
            self.rpc_url,
            self.private_key,
            wasm_path,
            self.ctr_args,
            sol_path,
        )
        .await
    }
}
