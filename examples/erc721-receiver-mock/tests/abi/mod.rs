#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc721ReceiverMock {
        #[derive(Debug)]
        function onERC721Received(
            address operator,
            address from,
            uint256 token_id,
            bytes calldata data
        ) external returns (bytes4);

        #[derive(Debug, PartialEq)]
        error CustomError(bytes4 data);
    }
);
