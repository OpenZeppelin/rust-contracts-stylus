use alloy::{
    network::EthereumWallet,
    primitives::{Address, B256},
    providers::ProviderBuilder,
    signers::{local::PrivateKeySigner, Signature, Signer},
};
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    deploy::Deployer,
    system::{fund_account, get_rpc_url, Wallet},
};

const DEFAULT_FUNDING_ETH: u32 = 100;

/// Type that corresponds to a test account.
#[derive(Clone)]
pub struct Account {
    /// The account's local private key wrapper.
    pub signer: PrivateKeySigner,
    /// The account's wallet -- an `alloy` provider with a `WalletFiller`.
    pub wallet: Wallet,
}

impl Account {
    /// Create a new account with a default funding of [`DEFAULT_FUNDING_ETH`].
    ///
    /// # Errors
    ///
    /// May fail if funding the newly created account fails.
    pub async fn new() -> eyre::Result<Self> {
        AccountFactory::create().await
    }

    /// Get a hex-encoded String representing this account's private key.
    #[must_use]
    pub fn pk(&self) -> String {
        alloy::hex::encode(self.signer.to_bytes())
    }

    /// Retrieve this account's address.
    #[must_use]
    pub fn address(&self) -> Address {
        self.signer.address()
    }

    /// Sign the given hash.
    ///
    /// # Panics
    ///
    /// May fail when the method is not implemented for `Signer`. Should not
    /// happen.
    pub async fn sign_hash(&self, hash: &B256) -> Signature {
        self.signer.sign_hash(hash).await.expect("should sign a hash")
    }

    /// Sign the given message.
    ///
    /// # Panics
    ///
    /// May fail when the method is not implemented for `Signer`. Should not
    /// happen.
    pub async fn sign_message(&self, message: &[u8]) -> Signature {
        self.signer.sign_message(message).await.expect("should sign a message")
    }

    /// Create a configurable smart contract deployer on behalf of this account.
    #[must_use]
    pub fn as_deployer(&self) -> Deployer {
        Deployer::new(get_rpc_url(), self.pk())
    }
}

/// A unit struct used as a synchronization mechanism in
/// [`SYNC_ACCOUNT_FACTORY`].
struct AccountFactory;

impl AccountFactory {
    /// Get access to the factory in a synchronized manner.
    async fn lock() -> MutexGuard<'static, Self> {
        /// Since after wallet generation accounts get funded in the nitro test
        /// node from a single "god" wallet, we must synchronize account
        /// creation (otherwise the nonce will be too low).
        static SYNC_ACCOUNT_FACTORY: std::sync::LazyLock<
            Mutex<AccountFactory>,
        > = std::sync::LazyLock::new(|| Mutex::new(AccountFactory));

        SYNC_ACCOUNT_FACTORY.lock().await
    }

    /// Create new account and fund it via nitro test node access.
    ///
    /// # Errors
    ///
    /// May fail if unable to find the path to the node or if funding the newly
    /// created account fails.
    async fn create() -> eyre::Result<Account> {
        let _lock = AccountFactory::lock().await;

        let signer = PrivateKeySigner::random();
        let addr = signer.address();
        fund_account(addr, DEFAULT_FUNDING_ETH)?;

        let rpc_url = get_rpc_url();
        let wallet = ProviderBuilder::new()
            .with_simple_nonce_management()
            .wallet(EthereumWallet::from(signer.clone()))
            .connect_http(rpc_url);

        Ok(Account { signer, wallet })
    }
}
