use alloy::{
    network::EthereumWallet,
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
};
use eyre::{bail, Result};
use once_cell::sync::Lazy;
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    environment::get_node_path,
    system::{Wallet, RPC_URL_ENV_VAR_NAME},
};

/// Type that corresponds to a test user.
#[derive(Clone, Debug)]
pub struct User {
    /// The account's local private key wrapper.
    pub signer: PrivateKeySigner,
    /// The account's wallet -- an `alloy` provider with a `WalletFiller`.
    pub wallet: Wallet,
}

impl User {
    /// Create a new user.
    ///
    /// # Errors
    ///
    /// May fail if funding the newly created account fails.
    pub async fn new() -> Result<Self> {
        UserFactory::create().await
    }

    /// Get a hex-encoded String representing this user's private key.
    #[must_use]
    pub fn pk(&self) -> String {
        alloy::hex::encode(self.signer.to_bytes())
    }

    /// Retrieve this account's address.
    #[must_use]
    pub fn address(&self) -> Address {
        self.signer.address()
    }

    /// The rpc endpoint this user's provider is connect to.
    #[must_use]
    pub fn url(&self) -> &str {
        self.wallet.client().transport().url()
    }
}

/// A unit struct used as a synchronization mechanism in [`SYNC_USER_FACTORY`].
struct UserFactory;

impl UserFactory {
    /// Get access to the factory in a synchronized manner.
    async fn lock() -> MutexGuard<'static, Self> {
        /// Since after wallet generation users get funded in the nitro test
        /// node from a single "god" wallet, we must synchronize user
        /// creation (otherwise the nonce will be too low).
        static SYNC_USER_FACTORY: Lazy<Mutex<UserFactory>> =
            Lazy::new(|| Mutex::new(UserFactory));

        SYNC_USER_FACTORY.lock().await
    }

    /// Create new account and fund it via nitro test node access.
    ///
    /// # Errors
    ///
    /// May fail if unable to find the path to the node or if funding the newly
    /// created account fails.
    async fn create() -> eyre::Result<User> {
        let _lock = UserFactory::lock().await;

        let signer = PrivateKeySigner::random();
        let addr = signer.address();

        // ./test-node.bash script send-l2 --to
        // address_0x01fA6bf4Ee48B6C95900BCcf9BEA172EF5DBd478 --ethamount 10
        let node_script = get_node_path()?.join("test-node.bash");
        let output = std::process::Command::new(node_script)
            .arg("script")
            .arg("send-l2")
            .arg("--to")
            .arg(format!("address_{addr}"))
            .arg("--ethamount")
            .arg("10")
            .output()?;

        let rpc_url = std::env::var(RPC_URL_ENV_VAR_NAME)
            .expect("failed to load RPC_URL var from env")
            .parse()
            .expect("failed to parse RPC_URL string into a URL");
        let wallet = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(signer.clone()))
            .on_http(rpc_url);

        if output.status.success() {
            Ok(User { signer, wallet })
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            bail!("user's wallet wasn't funded - address is {addr}:\n{err}")
        }
    }
}
