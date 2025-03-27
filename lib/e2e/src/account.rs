use std::str::FromStr;

use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::{uint, Address, B256, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::{local::PrivateKeySigner, Signature, Signer},
    transports::http::reqwest::Url,
};
use once_cell::sync::Lazy;
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    deploy::Deployer,
    system::{Wallet, RPC_URL_ENV_VAR_NAME},
};

const MASTER_PRIVATE_KEY: &str =
    "0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659";
const DEFAULT_FUNDING_ETH: U256 = uint!(100_000_000_000_000_000_U256);

/// Type that corresponds to a test account.
#[derive(Clone, Debug)]
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

    /// The rpc endpoint this account's provider is connect to.
    #[must_use]
    pub fn url(&self) -> &str {
        self.wallet.client().transport().url()
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
        Deployer::new(self.url().to_string(), self.pk())
    }

    /// Get gas token balance.
    #[must_use]
    pub async fn balance(&self) -> U256 {
        ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(self.signer.clone()))
            .on_http(self.url().parse().expect("URL already valid"))
            .get_balance(self.address())
            .await
            .expect("should get balance")
    }

    /// Send gas token to an address.
    pub async fn send_value(
        &self,
        to: Address,
        value: U256,
    ) -> eyre::Result<()> {
        let wallet = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(self.signer.clone()))
            .on_http(self.url().parse().expect("URL already valid"));

        let tx = TransactionRequest::default()
            .with_from(self.address())
            .with_to(to)
            .with_value(value);

        wallet
            .send_transaction(tx)
            .await?
            .watch()
            .await
            .expect("funds were not sent");

        Ok(())
    }

    /// Return all balance to master.
    pub async fn return_balance_to_master(&self) -> eyre::Result<()> {
        let balance = self.balance().await;
        let to = get_master_signer().address();
        let wallet = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(self.signer.clone()))
            .on_http(self.url().parse().expect("URL already valid"));

        let tx = TransactionRequest::default()
            .with_from(self.address())
            .with_to(to)
            .with_value(balance);

        let estimated_gas = wallet.estimate_gas(&tx).await?;

        let gas_price = wallet.get_gas_price().await?;

        // Compute gas cost
        let gas_cost = U256::from(estimated_gas)
            .saturating_mul(U256::from(gas_price))
            .saturating_mul(U256::from(3));

        // Ensure balance is sufficient to cover gas cost
        if balance <= gas_cost {
            return Ok(()); // If not enough, then there's nothing to do
        };

        self.send_value(to, balance - gas_cost).await
    }
}

/// A unit struct used as a synchronization mechanism in
/// [`SYNC_ACCOUNT_FACTORY`].
struct AccountFactory;

impl AccountFactory {
    /// Get access to the factory in a synchronized manner.
    async fn lock() -> MutexGuard<'static, Self> {
        /// Since after wallet generation accounts get funded in the nitro dev
        /// node from a single "god" wallet, we must synchronize account
        /// creation (otherwise the nonce will be too low).
        static SYNC_ACCOUNT_FACTORY: Lazy<Mutex<AccountFactory>> =
            Lazy::new(|| Mutex::new(AccountFactory));

        SYNC_ACCOUNT_FACTORY.lock().await
    }

    /// Create new account and fund it via nitro node access.
    ///
    /// # Errors
    ///
    /// May fail if unable to find the path to the node or if funding the newly
    /// created account fails.
    async fn create() -> eyre::Result<Account> {
        let _lock = AccountFactory::lock().await;

        let signer = PrivateKeySigner::random();
        let account_address = signer.address();

        let rpc_url: Url = std::env::var(RPC_URL_ENV_VAR_NAME)
            .expect("failed to load RPC_URL var from env")
            .parse()
            .expect("failed to parse RPC_URL string into a URL");

        let master = get_master_signer();

        let master_wallet = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(master.clone()))
            .on_http(rpc_url.clone());

        let tx = TransactionRequest::default()
            .with_from(master.address())
            .with_to(account_address)
            .with_value(DEFAULT_FUNDING_ETH);

        master_wallet
            .send_transaction(tx)
            .await?
            .watch()
            .await
            .expect("account's wallet wasn't funded");

        let wallet = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(signer.clone()))
            .on_http(rpc_url);

        Ok(Account { signer, wallet })
    }
}

/// Get Master signer for the chain.
fn get_master_signer() -> PrivateKeySigner {
    PrivateKeySigner::from_str(MASTER_PRIVATE_KEY)
        .expect("failed to create master signer")
}
