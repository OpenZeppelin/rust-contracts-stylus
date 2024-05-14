use std::sync::Arc;

use ethers::{addressbook::Address, contract::abigen};

use crate::infrastructure::{HttpMiddleware, Token};

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
    const STYLUS_PROGRAM_ADDRESS: &'static str = "ERC721_EXAMPLE_DEPLOYMENT_ADDRESS";

    fn new(address: Address, client: Arc<HttpMiddleware>) -> Self {
        Self::new(address, client)
    }
}
