#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc1155 {
        function balanceOf(address account, uint256 id) external view returns (uint256 balance);
        #[derive(Debug)]
        function balanceOfBatch(address[] accounts, uint256[] ids) external view returns (uint256[] memory balances);
        function isApprovedForAll(address account, address operator) external view returns (bool approved);
        function setApprovalForAll(address operator, bool approved) external;
        function safeTransferFrom(address from, address to, uint256 id, uint256 value, bytes memory data) external;
        function safeBatchTransferFrom(address from, address to, uint256[] memory ids, uint256[] memory values, bytes memory data) external;
        function mint(address to, uint256 id, uint256 amount, bytes memory data) external;
        function mintBatch(address to, uint256[] memory ids, uint256[] memory amounts, bytes memory data) external;
        function burn(address account, uint256 id, uint256 value) external;
        function burnBatch(address account, uint256[] memory ids, uint256[] memory values) external;

        error InvalidReceiverWithReason(string message);
        error ERC1155InvalidArrayLength(uint256 idsLength, uint256 valuesLength);
        error ERC1155InvalidOperator(address operator);
        error ERC1155InvalidSender(address sender);
        error ERC1155InvalidReceiver(address receiver);
        error ERC1155MissingApprovalForAll(address operator, address owner);
        error ERC1155InsufficientBalance(address sender, uint256 balance, uint256 needed, uint256 tokenId);

        error Error(string message);
        error Panic(uint256 code);

        #[derive(Debug, PartialEq)]
        event TransferSingle(address indexed operator, address indexed from, address indexed to, uint256 id, uint256 value);
        #[derive(Debug, PartialEq)]
        event TransferBatch(address indexed operator, address indexed from, address indexed to, uint256[] ids, uint256[] values);
        #[derive(Debug, PartialEq)]
        event ApprovalForAll(address indexed account, address indexed operator, bool approved);
    }
);
