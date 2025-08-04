#![cfg(feature = "e2e")]

use alloy::sol;

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract Erc721HolderExample {
        function onERC721Received(
            address operator,
            address from,
            uint256 tokenId,
            bytes calldata data
        ) external returns (bytes4);
    }
}
