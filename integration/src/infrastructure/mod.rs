pub mod erc20;
pub mod erc721;

use std::{ops::Deref, str::FromStr, sync::Arc};

use async_trait::async_trait;
use ethers::{
    abi::{AbiEncode, Detokenize},
    contract::ContractCall,
    middleware::{Middleware, SignerMiddleware},
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, TransactionReceipt, U256},
};
use eyre::{bail, Context, ContextCompat, Report, Result};

const ALICE_PRIV_KEY: &str = "ALICE_PRIV_KEY";
const BOB_PRIV_KEY: &str = "BOB_PRIV_KEY";
const RPC_URL: &str = "RPC_URL";


/// Integration testing infrastructure that allows to act on behalf of `alice`
/// and `bob` accounts.
pub struct Infrastructure<T: Token> {
    pub alice: Client<T>,
    pub bob: Client<T>,
}

impl<T: Token> Infrastructure<T> {
    /// Constructs new instance of an integration testing infrastructure.
    ///
    /// Requires env variables `ALICE_PRIV_KEY`, `BOB_PRIV_KEY`, `RPC_URL`
    /// and <CRATE_NAME>_DEPLOYMENT_ADDRESS
    /// where <CRATE_NAME> is the "SCREAMING_SNAKE_CASE" conversion of the crate
    /// name from the `./examples` directory.
    pub async fn new() -> eyre::Result<Self> {
        let alice_priv_key = std::env::var(ALICE_PRIV_KEY)
            .with_context(|| format!("Load {} env var", ALICE_PRIV_KEY))?;
        let bob_priv_key = std::env::var(BOB_PRIV_KEY)
            .with_context(|| format!("Load {} env var", BOB_PRIV_KEY))?;
        let rpc_url = std::env::var(RPC_URL)
            .with_context(|| format!("Load {} env var", RPC_URL))?;
        let stylus_program_address = std::env::var(T::STYLUS_PROGRAM_ADDRESS)
            .with_context(|| {
            format!("Load {} env var", T::STYLUS_PROGRAM_ADDRESS)
        })?;

        let program_address: Address = stylus_program_address.parse()?;
        let provider = Provider::<Http>::try_from(rpc_url)?;

        Ok(Infrastructure {
            alice: Client::new(
                provider.clone(),
                program_address,
                alice_priv_key,
            )
            .await?,
            bob: Client::new(provider, program_address, bob_priv_key).await?,
        })
    }
}

/// Client of participant that allows to check wallet address and call contract functions.
pub struct Client<T: Token> {
    pub wallet: LocalWallet,
    pub caller: T,
}

// Lets not to mention `caller` property every time we call a function.
impl<T: Token> Deref for Client<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.caller
    }
}


/// Abstraction for the deployed contract.
pub trait Token {
    /// Deployed program address environment variable name for the contract.
    ///
    /// e.g can be `ERC721_EXAMPLE_DEPLOYMENT_ADDRESS`.
    /// Formed by template <CRATE_NAME>_DEPLOYMENT_ADDRESS
    /// where <CRATE_NAME> is the "SCREAMING_SNAKE_CASE" conversion of the crate
    /// name from the `./examples` directory.
    /// That environment variable should store an address of the corresponding
    /// deployed program.
    const STYLUS_PROGRAM_ADDRESS: &'static str;

    /// Abstracts token creation function.
    ///
    /// e.g. `Self::new(address, client)`.
    fn new(address: Address, client: Arc<HttpMiddleware>) -> Self;
}

pub type HttpMiddleware = SignerMiddleware<Provider<Http>, LocalWallet>;

impl<T: Token> Client<T> {
    pub async fn new(
        provider: Provider<Http>,
        program_address: Address,
        priv_key: String,
    ) -> Result<Self> {
        let wallet = LocalWallet::from_str(&priv_key)?;
        let chain_id = provider.get_chainid().await?.as_u64();
        let signer = Arc::new(SignerMiddleware::new(
            provider,
            wallet.clone().with_chain_id(chain_id),
        ));
        let caller = T::new(program_address, signer);
        Ok(Self { wallet, caller })
    }
}

pub fn random_token_id() -> U256 {
    let num: u32 = ethers::core::rand::random();
    num.into()
}

pub trait Assert<E: AbiEncode> {
    /// Asserts that current error result corresponds to the typed abi encoded
    /// error `expected_err`.
    fn assert(&self, expected_err: E) -> Result<()>;
}

impl<E: AbiEncode> Assert<E> for Report {
    fn assert(&self, expected_err: E) -> Result<()> {
        let received_err = format!("{:#}", self);
        let expected_err = expected_err.encode_hex();
        if received_err.contains(&expected_err) {
            Ok(())
        } else {
            bail!("Different error expected: Expected error is {expected_err}: Received error is {received_err}")
        }
    }
}

#[async_trait]
pub trait ContextCall<R> {
    /// Queries the blockchain via an `eth_call` for the provided transaction.
    ///
    /// Wraps error with function info context.
    ///
    /// If executed on a non-state mutating smart contract function (i.e.
    /// `view`, `pure`) then it will return the raw data from the chain.
    ///
    /// If executed on a mutating smart contract function, it will do a "dry
    /// run" of the call and return the return type of the transaction
    /// without mutating the state
    async fn ctx_call(self) -> Result<R>;
}

#[async_trait]
impl<R: Detokenize + Send + Sync> ContextCall<R>
    for ContractCall<HttpMiddleware, R>
{
    async fn ctx_call(self) -> Result<R> {
        let function_name = self.function.name.clone();
        self.call().await.context(format!("calling {function_name}"))
    }
}

#[async_trait]
pub trait ContextSend {
    /// Signs and broadcasts the provided transaction.
    ///
    /// Wraps error with function info context.
    async fn ctx_send(self) -> Result<TransactionReceipt>;
}

#[async_trait]
impl ContextSend for ContractCall<HttpMiddleware, ()> {
    async fn ctx_send(self) -> Result<TransactionReceipt> {
        let function_name = self.function.name.clone();
        self.send()
            .await
            .context(format!("sending {function_name}"))?
            .await
            .context(format!("sending {function_name}"))?
            .context(format!("sending {function_name}"))
    }
}
