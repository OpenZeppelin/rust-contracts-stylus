#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc721Wrapper {
        #[derive(Debug)]
        function balanceOf(address owner) external view returns (uint256 balance);
        #[derive(Debug)]
        function underlying() external view returns (address underlying);
        #[derive(Debug)]
        function ownerOf(uint256 tokenId) external view returns (address owner);
        #[derive(Debug)]
        function depositFor(address account, uint256[] memory tokenIds) external returns (bool);
        #[derive(Debug)]
        function withdrawTo(address account, uint256[] memory tokenIds) external returns (bool);


        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);

        error ERC721NonexistentToken(uint256 tokenId);
    }

    contract Erc721 {
        function safeTransferFrom(address from, address to, uint256 tokenId, bytes calldata data) external;

        #[derive(Debug)]
        function ownerOf(uint256 tokenId) external view returns (address owner);

        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
    }
);
