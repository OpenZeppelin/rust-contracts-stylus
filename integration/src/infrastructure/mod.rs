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

const FIRST_PRIV_KEY_PATH: &str = "FIRST_PRIV_KEY_PATH";
const SECOND_PRIV_KEY_PATH: &str = "SECOND_PRIV_KEY_PATH";
const RPC_URL: &str = "RPC_URL";
const STYLUS_PROGRAM_ADDRESS: &str = "STYLUS_PROGRAM_ADDRESS";

pub struct Infrastructure<T: Token> {
    pub first: Client<T>,
    pub second: Client<T>,
}

impl<T: Token> Infrastructure<T> {
    pub async fn new() -> eyre::Result<Self> {
        dotenv().ok();

        let first_priv_key_path = std::env::var(FIRST_PRIV_KEY_PATH)
            .with_context(|| format!("Load {} env var", FIRST_PRIV_KEY_PATH))?;
        let second_priv_key_path = std::env::var(SECOND_PRIV_KEY_PATH)
            .with_context(|| {
                format!("Load {} env var", SECOND_PRIV_KEY_PATH)
            })?;
        let rpc_url = std::env::var(RPC_URL)
            .with_context(|| format!("Load {} env var", RPC_URL))?;
        let stylus_program_address = std::env::var(STYLUS_PROGRAM_ADDRESS)
            .with_context(|| {
                format!("Load {} env var", STYLUS_PROGRAM_ADDRESS)
            })?;

        let program_address: Address = stylus_program_address.parse()?;
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let first_priv_key =
            std::fs::read_to_string(first_priv_key_path)?.trim().to_string();
        let second_priv_key =
            std::fs::read_to_string(second_priv_key_path)?.trim().to_string();

        Ok(Infrastructure {
            first: Client::<T>::new(
                provider.clone(),
                program_address,
                first_priv_key,
            )
            .await?,
            second: Client::<T>::new(
                provider,
                program_address,
                second_priv_key,
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
            bail!("Different error expected: expected error is {expected_err}: received error is {received_err}")
        }
    }
}
