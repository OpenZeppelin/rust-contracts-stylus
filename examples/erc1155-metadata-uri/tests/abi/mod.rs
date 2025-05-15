#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc1155 {
        #[derive(Debug)]
        function uri(uint256 id) external view returns (string memory uri);
        function setTokenURI(uint256 tokenId, string memory tokenURI) external;
        function setBaseURI(string memory tokenURI) external;

        #[derive(Debug, PartialEq)]
        event URI(string value, uint256 indexed id);
    }
);
