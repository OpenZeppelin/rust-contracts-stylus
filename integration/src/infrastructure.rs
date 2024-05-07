use crate::function;
use dotenv::dotenv;
use ethers::abi::AbiEncode;
use ethers::contract::ContractError;
use ethers::middleware::Middleware;
use ethers::prelude::{FunctionCall, PendingTransaction, ProviderError};
use ethers::{
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, TransactionReceipt, U256},
};
use eyre::{bail, Context, ContextCompat, Report, Result};
use std::str::FromStr;
use std::sync::Arc;

abigen!(
    Nft,
    r#"[
        function name() external view returns (string memory)
        function symbol() external view returns (string memory)
        function tokenURI(uint256 token_id) external view returns (string memory)

        function supportsInterface(bytes4 interface_id) external pure returns (bool)

        function balanceOf(address owner) external view returns (uint256)
        function ownerOf(uint256 token_id) external view returns (address)
        function safeTransferFrom(address from, address to, uint256 token_id) external
        function safeTransferFrom(address from, address to, uint256 token_id, bytes calldata data) external
        function transferFrom(address from, address to, uint256 token_id) external
        function approve(address to, uint256 token_id) external
        function setApprovalForAll(address operator, bool approved) external
        function getApproved(uint256 token_id) external view returns (address)
        function isApprovedForAll(address owner, address operator) external view returns (bool)

        function burn(uint256 token_id) external
        function mint(address to, uint256 token_id) external

        function paused() external view returns (bool)
        function pause() external
        function unpause() external

        error ERC721InvalidOwner(address owner)
        error ERC721NonexistentToken(uint256 tokenId)
        error ERC721IncorrectOwner(address sender, uint256 tokenId, address owner)
        error ERC721InvalidSender(address sender)
        error ERC721InvalidReceiver(address receiver)
        error ERC721InsufficientApproval(address operator, uint256 tokenId)
        error ERC721InvalidApprover(address approver)
        error ERC721InvalidOperator(address operator)

        error EnforcedPause()
        error ExpectedPause()
    ]"#
);

const FIRST_PRIV_KEY_PATH: &str = "FIRST_PRIV_KEY_PATH";
const SECOND_PRIV_KEY_PATH: &str = "SECOND_PRIV_KEY_PATH";
const RPC_URL: &str = "RPC_URL";
const STYLUS_PROGRAM_ADDRESS: &str = "STYLUS_PROGRAM_ADDRESS";

pub struct Infrastructure {
    pub first: Client,
    pub second: Client,
}

impl Infrastructure {
    pub async fn create() -> eyre::Result<Self> {
        dotenv().ok();

        let first_priv_key_path = std::env::var(FIRST_PRIV_KEY_PATH)
            .with_context(|| format!("Load {} env var", FIRST_PRIV_KEY_PATH))?;
        let second_priv_key_path = std::env::var(SECOND_PRIV_KEY_PATH)
            .with_context(|| format!("Load {} env var", SECOND_PRIV_KEY_PATH))?;
        let rpc_url =
            std::env::var(RPC_URL).with_context(|| format!("Load {} env var", RPC_URL))?;
        let stylus_program_address = std::env::var(STYLUS_PROGRAM_ADDRESS)
            .with_context(|| format!("Load {} env var", STYLUS_PROGRAM_ADDRESS))?;

        let program_address: Address = stylus_program_address.parse()?;
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let first_priv_key = std::fs::read_to_string(first_priv_key_path)?
            .trim()
            .to_string();
        let second_priv_key = std::fs::read_to_string(second_priv_key_path)?
            .trim()
            .to_string();

        Ok(Infrastructure {
            first: Client::create(provider.clone(), program_address, first_priv_key).await?,
            second: Client::create(provider, program_address, second_priv_key).await?,
        })
    }
}

pub struct Client {
    pub wallet: LocalWallet,
    pub caller: Caller,
}

impl Client {
    pub async fn create(
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
        let caller = Nft::new(program_address, signer);
        Ok(Self { wallet, caller })
    }

    pub async fn name(&self) -> Result<String> {
        self.caller
            .name()
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn symbol(&self) -> Result<String> {
        self.caller
            .symbol()
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn token_uri(&self, token_id: U256) -> Result<String> {
        self.caller
            .token_uri(token_id)
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn balance_of(&self, owner: Address) -> Result<U256> {
        self.caller
            .balance_of(owner)
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn mint(&self, to: Address, token_id: U256) -> Result<TransactionReceipt> {
        self.caller
            .mint(to, token_id)
            .send()
            .await?
            .await?
            .context(format!("Error sending {}", function!()))
    }

    pub async fn burn(&self, token_id: U256) -> Result<TransactionReceipt> {
        self.caller
            .burn(token_id)
            .send()
            .await?
            .await?
            .context(format!("Error sending {}", function!()))
    }

    pub async fn transfer_from(
        &self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<TransactionReceipt> {
        self.caller
            .transfer_from(from, to, token_id)
            .send()
            .await?
            .await?
            .context(format!("Error sending {}", function!()))
    }

    pub async fn owner_of(&self, token_id: U256) -> Result<Address> {
        self.caller
            .owner_of(token_id)
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn approve(&self, to: Address, token_id: U256) -> Result<TransactionReceipt> {
        self.caller
            .approve(to, token_id)
            .send()
            .await?
            .await?
            .context(format!("Error sending {}", function!()))
    }

    pub async fn get_approved(&self, token_id: U256) -> Result<Address> {
        self.caller
            .get_approved(token_id)
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn paused(&self) -> Result<bool> {
        self.caller
            .paused()
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn pause(&self) -> Result<TransactionReceipt> {
        self.caller
            .pause()
            .send()
            .await?
            .await?
            .context(format!("Error sending {}", function!()))
    }

    pub async fn unpause(&self) -> Result<TransactionReceipt> {
        self.caller
            .unpause()
            .send()
            .await?
            .await?
            .context(format!("Error sending {}", function!()))
    }

    pub async fn support_interface(&self, interface_id: u32) -> Result<bool> {
        let interface_id = interface_id.to_be_bytes();
        self.caller
            .supports_interface(interface_id)
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }
}

pub type Caller = Nft<SignerMiddleware<Provider<Http>, LocalWallet>>;

pub fn random_token_id() -> U256 {
    let num: u32 = ethers::core::rand::random();
    num.into()
}

pub trait Assert<E: AbiEncode> {
    fn assert_has(&self, expected_err: E) -> Result<()>;
}

impl<E: AbiEncode> Assert<E> for Report {
    fn assert_has(&self, expected_err: E) -> Result<()> {
        let received_err = format!("{:#}", self);
        let expected_err = expected_err.encode_hex();
        if received_err.contains(&expected_err) {
            Ok(())
        } else {
            bail!("Different error expected: expected error is {expected_err}: received error is {received_err}")
        }
    }
}
