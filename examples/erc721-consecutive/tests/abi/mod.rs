#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract Erc721 {
        #[derive(Debug)]
        function balanceOf(address owner) external view returns (uint256 balance);
        #[derive(Debug)]
        function ownerOf(uint256 tokenId) external view returns (address ownerOf);
        function safeTransferFrom(address from, address to, uint256 tokenId) external;
        function safeTransferFrom(address from, address to, uint256 tokenId, bytes calldata data) external;
        function transferFrom(address from, address to, uint256 tokenId) external;
        function approve(address to, uint256 tokenId) external;
        function setApprovalForAll(address operator, bool approved) external;
        function getApproved(uint256 tokenId) external view returns (address);
        function isApprovedForAll(address owner, address operator) external view returns (bool);
        function mint(address to, uint256 tokenId) external;

        function burn(uint256 tokenId) external;

        error ERC721InvalidOwner(address owner);
        error ERC721NonexistentToken(uint256 tokenId);
        error ERC721IncorrectOwner(address sender, uint256 tokenId, address owner);
        error ERC721InvalidSender(address sender);
        error ERC721InvalidReceiver(address receiver);
        error ERC721InsufficientApproval(address operator, uint256 tokenId);
        error ERC721InvalidApprover(address approver);
        error ERC721InvalidOperator(address operator);

        error ERC721ForbiddenBatchMint();
        error ERC721ExceededMaxBatchMint(uint256 batchSize, uint256 maxBatch);
        error ERC721ForbiddenMint();
        error ERC721ForbiddenBatchBurn();

        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);

        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);

        #[derive(Debug, PartialEq)]
        event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

        #[derive(Debug, PartialEq)]
        event ConsecutiveTransfer(
               uint256 indexed fromTokenId,
               uint256 toTokenId,
               address indexed fromAddress,
               address indexed toAddress
          );
    }
);
