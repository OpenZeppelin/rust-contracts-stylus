#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc1155 {
        function balanceOf(address account, uint256 id) external view returns (uint256 balance);
        // function balanceOfBatch(address[] accounts, uint256[] ids) external view returns (uint256[] memory);
        function paused() external view returns (bool paused);
        function pause() external;
        function unpause() external;
    }
);
