use std::{fs, io::ErrorKind, path::PathBuf, time::Duration};

use alloy::{
    network::{Ethereum, EthereumWallet, TransactionBuilder},
    primitives::{utils::parse_ether, Address, Signature, B256, U256},
    providers::{
        fillers::{
            BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill,
            NonceFiller, WalletFiller,
        },
        Identity, Provider, ProviderBuilder, RootProvider,
    },
    rpc::types::TransactionRequest,
    signers::{local::PrivateKeySigner, Signer},
};
use eyre::Context;
use tokio::sync::{Mutex, OnceCell};

use crate::{
    deploy::Deployer,
    devnet::{devnet_dir, rpc_url, DEVNET_PRIVATE_KEY},
};

/// Balance given to every new [`Account`]: 0.1 ETH.
///
/// Deploying and activating an example contract on the devnet costs about
/// 0.005 ETH, and the devnet owner holds about 1000 ETH in total.
pub const DEFAULT_FUNDING_WEI: U256 = U256::from_limbs([100_000_000_000_000_000, 0, 0, 0]);

/// Amount moved from the devnet owner to this process's faucet at a time.
const FAUCET_TOP_UP_ETH: &str = "2";

/// A lock held for longer than this is considered abandoned.
const LOCK_TIMEOUT: Duration = Duration::from_secs(120);

/// Convenience type alias that represents an Ethereum wallet: an `alloy`
/// provider with the recommended fillers and a [`WalletFiller`].
pub type Wallet = FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<
                GasFiller,
                JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>,
            >,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<Ethereum>,
    Ethereum,
>;

/// Type that corresponds to a test account.
#[derive(Clone, Debug)]
pub struct Account {
    /// The account's local private key wrapper.
    pub signer: PrivateKeySigner,
    /// The account's wallet -- an `alloy` provider with a `WalletFiller`.
    pub wallet: Wallet,
    url: String,
}

impl Account {
    /// Create a new account with a default funding of
    /// [`DEFAULT_FUNDING_WEI`].
    ///
    /// # Errors
    ///
    /// May fail if the devnet is not reachable or funding the newly created
    /// account fails.
    pub async fn new() -> eyre::Result<Self> {
        let account = Self::unfunded()?;
        account.fund(DEFAULT_FUNDING_WEI).await?;
        Ok(account)
    }

    /// Create a new account without any balance.
    ///
    /// # Errors
    ///
    /// May fail if the RPC endpoint of the devnet is unknown.
    pub fn unfunded() -> eyre::Result<Self> {
        Self::with_signer(PrivateKeySigner::random())
    }

    fn with_signer(signer: PrivateKeySigner) -> eyre::Result<Self> {
        Self::from_signer(signer, &rpc_url()?)
    }

    /// Create an account for `signer` connected to the RPC endpoint `url`,
    /// without funding it.
    ///
    /// Use it to drive the deployer with an existing key on another chain.
    ///
    /// # Errors
    ///
    /// May fail if `url` is not a valid URL.
    pub fn from_signer(
        signer: PrivateKeySigner,
        url: &str,
    ) -> eyre::Result<Self> {
        let wallet = connect(url, signer.clone())?;
        Ok(Self { signer, wallet, url: url.to_owned() })
    }

    /// Send `amount` wei from the devnet faucet to this account.
    ///
    /// # Errors
    ///
    /// May fail if the devnet owner runs out of funds or the transfer is
    /// rejected.
    pub async fn fund(&self, amount: U256) -> eyre::Result<()> {
        Faucet::send(Faucet::get().await?, self.address(), amount).await
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

    /// The rpc endpoint this account's provider is connected to.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
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
        Deployer::new(self.clone())
    }
}

/// Connects a wallet provider for `signer` to `url`.
pub(crate) fn connect(
    url: &str,
    signer: PrivateKeySigner,
) -> eyre::Result<Wallet> {
    let url = url.parse().wrap_err("failed to parse the RPC URL")?;
    Ok(ProviderBuilder::new().wallet(EthereumWallet::from(signer)).connect_http(url))
}

/// The faucet of this process.
///
/// Every test binary funds its accounts from its own faucet account, so only
/// the faucet top-ups compete for the devnet owner's nonce across processes.
/// Those are serialized with a lock file in the devnet directory.
///
/// Only the key is kept: every `#[tokio::test]` runs on its own runtime, so a
/// provider (and its cached nonce) is created for each transfer instead.
struct Faucet {
    signer: PrivateKeySigner,
    address: Address,
}

static FAUCET: OnceCell<Mutex<Faucet>> = OnceCell::const_new();

impl Faucet {
    async fn get() -> eyre::Result<&'static Mutex<Faucet>> {
        FAUCET
            .get_or_try_init(|| async {
                let signer = PrivateKeySigner::random();
                let address = signer.address();
                Ok::<_, eyre::Report>(Mutex::new(Faucet { signer, address }))
            })
            .await
    }

    /// Sends `amount` wei to `to`, topping the faucet up from the devnet
    /// owner first when needed.
    async fn send(
        faucet: &Mutex<Faucet>,
        to: Address,
        amount: U256,
    ) -> eyre::Result<()> {
        let faucet = faucet.lock().await;
        let wallet = connect(&rpc_url()?, faucet.signer.clone())?;
        let reserve = parse_ether("0.01")?;
        if wallet.get_balance(faucet.address).await? < amount + reserve {
            let top_up = parse_ether(FAUCET_TOP_UP_ETH)?.max(amount + reserve);
            faucet.top_up(top_up).await?;
        }
        transfer(&wallet, to, amount)
            .await
            .wrap_err("failed to fund the account from the faucet")
    }

    /// Moves `amount` wei from the devnet owner to the faucet.
    async fn top_up(&self, amount: U256) -> eyre::Result<()> {
        let _lock = OwnerLock::acquire()?;
        let owner: PrivateKeySigner = DEVNET_PRIVATE_KEY
            .parse()
            .expect("the devnet private key should be valid");
        // A fresh provider reads the owner's current nonce from the node.
        let owner = connect(&rpc_url()?, owner)?;
        transfer(&owner, self.address, amount)
            .await
            .wrap_err("failed to fund the faucet from the devnet owner")
    }
}

async fn transfer(from: &Wallet, to: Address, amount: U256) -> eyre::Result<()> {
    let tx = TransactionRequest::default().with_to(to).with_value(amount);
    from.send_transaction(tx).await?.watch().await?;
    Ok(())
}

/// Cross-process lock around transactions sent by the devnet owner.
struct OwnerLock {
    path: PathBuf,
}

impl OwnerLock {
    fn acquire() -> eyre::Result<Self> {
        let dir = devnet_dir();
        fs::create_dir_all(&dir)?;
        let path = dir.join("owner.lock");
        loop {
            match fs::OpenOptions::new().write(true).create_new(true).open(&path)
            {
                Ok(_) => return Ok(Self { path }),
                Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                    let abandoned = fs::metadata(&path)
                        .and_then(|m| m.modified())
                        .map(|t| t.elapsed().unwrap_or_default() > LOCK_TIMEOUT)
                        .unwrap_or(false);
                    if abandoned {
                        let _ = fs::remove_file(&path);
                    } else {
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
                Err(err) => return Err(err.into()),
            }
        }
    }
}

impl Drop for OwnerLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
