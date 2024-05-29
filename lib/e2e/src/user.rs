use alloy::{
    network::EthereumSigner,
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    signers::wallet::LocalWallet,
};
use eyre::{bail, Result};
use once_cell::sync::Lazy;
use tokio::sync::{Mutex, MutexGuard};

use crate::system::{get_node_path, Signer};

const RPC_URL: &str = "RPC_URL";

/// Type that corresponds to a test user.
#[derive(Clone, Debug)]
pub struct User {
    pub wallet: LocalWallet,
    pub signer: Signer,
}

impl User {
    /// Create a new user.
    pub async fn new() -> Result<Self> {
        UserFactory::get().await.create()
    }

    /// Get a hex-encoded String representing this user's private key.
    pub fn pk(&self) -> String {
        alloy::hex::encode(self.wallet.to_bytes())
    }

    /// Retrieve this account's address.
    pub fn address(&self) -> Address {
        self.wallet.address()
    }

    pub fn url(&self) -> &str {
        self.signer.client().transport().url()
    }
}

/// A unit struct used as a synchronization mechanism in [`SYNC_USER_FACTORY`].
struct UserFactory;

impl UserFactory {
    async fn get() -> MutexGuard<'static, Self> {
        /// Singleton User Factory.
        ///
        /// Since after wallet generation users get funded in the nitro test
        /// node from a single "god" wallet, we must synchronize user
        /// creation (otherwise the nonce will be too low).
        static SYNC_USER_FACTORY: Lazy<Mutex<UserFactory>> =
            Lazy::new(|| Mutex::new(UserFactory));

        SYNC_USER_FACTORY.lock().await
    }

    /// Create new account and fund it via nitro test node access.
    fn create(&self) -> eyre::Result<User> {
        let wallet = LocalWallet::random();
        let addr = wallet.address();

        // ./test-node.bash script send-l2 --to
        // address_0x01fA6bf4Ee48B6C95900BCcf9BEA172EF5DBd478 --ethamount 10
        let node_script = get_node_path()?.join("test-node.bash");
        let output = std::process::Command::new(node_script)
            .arg("script")
            .arg("send-l2")
            .arg("--to")
            .arg(format!("address_{}", addr))
            .arg("--ethamount")
            .arg("10")
            .output()?;

        let rpc_url = std::env::var(RPC_URL)
            .expect("failed to load RPC_URL var from env")
            .parse()
            .expect("failed to parse RPC_URL string into a URL");
        let signer = ProviderBuilder::new()
            .with_recommended_fillers()
            .signer(EthereumSigner::from(wallet.clone()))
            .on_http(rpc_url);

        match output.status.success() {
            true => Ok(User { wallet, signer }),
            false => {
                let err = String::from_utf8_lossy(&output.stderr);
                bail!("user's wallet wasn't filled - address is {addr}:\n{err}")
            }
        }
    }
}
