use std::{str::FromStr, sync::Arc};

use dotenv::dotenv;
use ethers::{
    abi::AbiEncode,
    middleware::{Middleware, SignerMiddleware},
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
};
use eyre::{bail, Context, ContextCompat, Report, Result};

pub mod erc721;
mod utils;

const ALICE_PRIV_KEY: &str = "ALICE_PRIV_KEY";
const BOB_PRIV_KEY: &str = "BOB_PRIV_KEY";
const RPC_URL: &str = "RPC_URL";
const STYLUS_PROGRAM_ADDRESS: &str = "STYLUS_PROGRAM_ADDRESS";

pub struct Infrastructure<T: Token> {
    pub alice: Client<T>,
    pub bob: Client<T>,
}

impl<T: Token> Infrastructure<T> {
    pub async fn new() -> eyre::Result<Self> {
        dotenv().ok();

        let alice_priv_key = std::env::var(ALICE_PRIV_KEY)
            .with_context(|| format!("Load {} env var", ALICE_PRIV_KEY))?;
        let bob_priv_key = std::env::var(BOB_PRIV_KEY)
            .with_context(|| {
                format!("Load {} env var", BOB_PRIV_KEY)
            })?;
        let rpc_url = std::env::var(RPC_URL)
            .with_context(|| format!("Load {} env var", RPC_URL))?;
        let stylus_program_address = std::env::var(T::STYLUS_PROGRAM_ADDRESS)
            .with_context(|| {
                format!("Load {} env var", T::STYLUS_PROGRAM_ADDRESS)
            })?;
        dbg!(&stylus_program_address);

        let program_address: Address = stylus_program_address.parse()?;
        let provider = Provider::<Http>::try_from(rpc_url)?;

        Ok(Infrastructure {
            alice: Client::new(
                provider.clone(),
                program_address,
                alice_priv_key,
            )
            .await?,
            bob: Client::new(
                provider,
                program_address,
                bob_priv_key,
            )
            .await?,
        })
    }
}

pub struct Client<T: Token> {
    pub wallet: LocalWallet,
    pub caller: T,
}

pub trait Token {
    const STYLUS_PROGRAM_ADDRESS: &'static str;
    
    fn new<T: Into<Address>>(address: T, client: Arc<HttpMiddleware>) -> Self;
}

pub type HttpMiddleware = SignerMiddleware<Provider<Http>, LocalWallet>;

impl<T: Token> Client<T> {
    pub async fn new(
        provider: Provider<Http>,
        program_address: Address,
        priv_key: String,
    ) -> eyre::Result<Self> {
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
