#![allow(dead_code)]
use alloy::sol;

sol! {
    #[sol(rpc)]
    contract Erc1155Supply {
        function totalSupply(uint256 id) external view returns (uint256 total_supply);
        function totalSupplyAll() external view returns (uint256 total_supply_all);
        function exists(uint256 id) external view returns (bool existed);
        function mint(address to, uint256[] memory ids, uint256[] memory values) external;
    }
}
