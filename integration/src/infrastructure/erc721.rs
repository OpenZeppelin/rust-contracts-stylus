use std::sync::Arc;

use ethers::{
    addressbook::Address,
    contract::abigen,
    prelude::{TransactionReceipt, U256},
};
use eyre::{Context, ContextCompat};

use crate::{
    function,
    infrastructure::{Client, HttpMiddleware, Token},
};

abigen!(
    Erc721Token,
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

pub type Erc721 = Erc721Token<HttpMiddleware>;

impl Token for Erc721 {
    fn new<T: Into<Address>>(address: T, client: Arc<HttpMiddleware>) -> Self {
        Erc721Token::new(address, client)
    }
}

impl Client<Erc721> {
    pub async fn name(&self) -> eyre::Result<String> {
        self.caller
            .name()
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn symbol(&self) -> eyre::Result<String> {
        self.caller
            .symbol()
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn token_uri(&self, token_id: U256) -> eyre::Result<String> {
        self.caller
            .token_uri(token_id)
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn balance_of(&self, owner: Address) -> eyre::Result<U256> {
        self.caller
            .balance_of(owner)
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn mint(
        &self,
        to: Address,
        token_id: U256,
    ) -> eyre::Result<TransactionReceipt> {
        self.caller
            .mint(to, token_id)
            .send()
            .await?
            .await?
            .context(format!("Error sending {}", function!()))
    }

    pub async fn burn(
        &self,
        token_id: U256,
    ) -> eyre::Result<TransactionReceipt> {
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
    ) -> eyre::Result<TransactionReceipt> {
        self.caller
            .transfer_from(from, to, token_id)
            .send()
            .await?
            .await?
            .context(format!("Error sending {}", function!()))
    }

    pub async fn owner_of(&self, token_id: U256) -> eyre::Result<Address> {
        self.caller
            .owner_of(token_id)
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn approve(
        &self,
        to: Address,
        token_id: U256,
    ) -> eyre::Result<TransactionReceipt> {
        self.caller
            .approve(to, token_id)
            .send()
            .await?
            .await?
            .context(format!("Error sending {}", function!()))
    }

    pub async fn get_approved(&self, token_id: U256) -> eyre::Result<Address> {
        self.caller
            .get_approved(token_id)
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn paused(&self) -> eyre::Result<bool> {
        self.caller
            .paused()
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }

    pub async fn pause(&self) -> eyre::Result<TransactionReceipt> {
        self.caller
            .pause()
            .send()
            .await?
            .await?
            .context(format!("Error sending {}", function!()))
    }

    pub async fn unpause(&self) -> eyre::Result<TransactionReceipt> {
        self.caller
            .unpause()
            .send()
            .await?
            .await?
            .context(format!("Error sending {}", function!()))
    }

    pub async fn support_interface(
        &self,
        interface_id: u32,
    ) -> eyre::Result<bool> {
        let interface_id = interface_id.to_be_bytes();
        self.caller
            .supports_interface(interface_id)
            .call()
            .await
            .context(format!("Error calling {}", function!()))
    }
}
