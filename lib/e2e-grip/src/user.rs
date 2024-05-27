use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use ethers::{
    core::{k256::ecdsa::SigningKey, rand::thread_rng},
    middleware::SignerMiddleware,
    prelude::*,
    utils::hex::hex,
};
use eyre::{bail, eyre, Context, ContextCompat, Report, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::sync::Mutex;

use crate::{context::E2EContext, Contract};

const RPC_URL: &str = "RPC_URL";
const TEST_NITRO_NODE_PATH: &str = "TEST_NITRO_NODE_PATH";

fn load_env_var(var_name: &str) -> eyre::Result<String> {
    std::env::var(var_name)
        .with_context(|| format!("failed to load {} env var", var_name))
}

/// Type that corresponds to a test user.
pub struct User {
    wallet: LocalWallet,
    provider: Provider<Http>,
    private_key: String,
}

/// Singleton user factory.
/// Since after wallet generation user get funded inside nitro test node from a
/// single "god" wallet. We should have synchronized user creation (otherwise
/// nonce will be too low)
static SYNC_USER_FACTORY: Lazy<Mutex<UserFactory>> =
    Lazy::new(|| Mutex::new(UserFactory));

impl User {
    /// Create new instance of user.
    pub async fn new() -> Result<Self> {
        SYNC_USER_FACTORY.lock().await.create().await
    }

    /// Accept deployed [`Self::deploys`] contract as an argument.
    /// Simulates user making call to a specific contract function.
    ///
    /// # Arguments
    /// * `contract_ctx` - Context of the contract deployed
    pub fn uses<T: Contract>(&self, contract_ctx: &E2EContext<T>) -> T {
        let signer = Arc::new(SignerMiddleware::new(
            self.provider.clone(),
            self.wallet.clone(),
        ));
        T::new(contract_ctx.address(), signer)
    }

    /// Allows to user deploy specific contract.
    /// [`T`] is an abi association with crate.
    ///
    /// # Examples
    /// ```rust, ignore
    /// let erc20 = &alice.deploys::<Erc20>().await?;
    /// ```
    pub async fn deploys<T: Contract>(&self) -> Result<E2EContext<T>> {
        let rpc_url = load_env_var(RPC_URL)?;
        let wasm_bin_name = T::CRATE_NAME.replace('-', "_") + ".wasm";

        let abs_wasm_bin_path = get_target_dir()?
            .join("wasm32-unknown-unknown/release")
            .join(wasm_bin_name);
        let output = tokio::process::Command::new("cargo")
            .arg("stylus")
            .arg("deploy")
            .arg("--wasm-file-path")
            .arg(abs_wasm_bin_path)
            .arg("-e")
            .arg(rpc_url)
            .arg("--private-key")
            .arg(&self.private_key)
            .arg("--nightly")
            .output()
            .await?;

        // NOTE: impossible to use `output.status` since it will be error
        // returned for a duplicated deployment of the contract
        let address =
            extract_deployment_address(&output.stdout).map_err(|err| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                eyre!(
                "deploy of the contract wasn't successful - crate name is {}:\n\
                parsing error is {err}:\n\
                stdout is {stdout}:\n\
                stderr is {stderr}",
                T::CRATE_NAME,
            )
            })?;
        Ok(E2EContext::new(address))
    }

    /// Retrieve wallet address of the user.
    pub fn address(&self) -> Address {
        self.wallet.address()
    }
}

fn extract_deployment_address(output: &[u8]) -> Result<Address> {
    let deployment_address_line = std::str::from_utf8(output)?
        .lines()
        .find(|l| l.contains("Deploying program to address"))
        .context("find deployment address line in the cargo-stylus output")?;

    let re = Regex::new(r"(0x)?[0-9a-fA-F]{40}").unwrap();
    let address = re
        .find(deployment_address_line)
        .context("extract deployment address from the cargo-stylus output")?;
    Ok(address.as_str().parse()?)
}

struct UserFactory;

impl UserFactory {
    /// Create new user and fund his wallet via test nitro node access.
    async fn create(&self) -> Result<User> {
        let rpc_url = load_env_var(RPC_URL)?;
        let test_nitro_node_path = load_env_var(TEST_NITRO_NODE_PATH)?;
        let test_nitro_node_script =
            Path::new(&test_nitro_node_path).join("test-node.bash");

        let provider = Provider::<Http>::try_from(rpc_url)?;

        let private_key =
            hex::encode(SigningKey::random(&mut thread_rng()).to_bytes());
        let chain_id = get_chain_id(&provider).await?;
        let wallet =
            LocalWallet::from_str(&private_key)?.with_chain_id(chain_id);
        let hex_address = hex::encode(wallet.address().as_bytes());

        // ./test-node.bash script send-l2 --to
        // address_0x01fA6bf4Ee48B6C95900BCcf9BEA172EF5DBd478 --ethamount 10000
        let output = tokio::process::Command::new(test_nitro_node_script)
            .arg("script")
            .arg("send-l2")
            .arg("--to")
            .arg(format!("address_0x{}", hex_address))
            .arg("--ethamount")
            .arg("10")
            .output()
            .await?;
        if output.status.success() {
            let user = User { wallet, provider, private_key };
            Ok(user)
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            bail!(
                "user wallet wasn't filled - address is {hex_address}:\n{err}"
            )
        }
    }
}

fn get_target_dir() -> Result<PathBuf> {
    let target_dir = load_env_var("TARGET_DIR")?;
    Ok(Path::new(&target_dir).to_path_buf())
}

async fn get_chain_id(provider: &Provider<Http>) -> Result<u64> {
    static CHAIN_ID: tokio::sync::OnceCell<u64> =
        tokio::sync::OnceCell::const_new();

    CHAIN_ID
        .get_or_try_init(|| async {
            Ok(provider
                .get_chainid()
                .await
                .context("Trying to get configured chain id. Try to setup nitro test node first")?
                .as_u64())
        })
        .await
        .cloned()
}
