#![allow(dead_code)]
use alloy::sol;

sol! {
    #[sol(rpc)]
    contract AccessControlEnumerableExample {
        constructor(string memory name_, string memory symbol_);

        // ERC20 functions
        function name() external view returns (string memory);
        function symbol() external view returns (string memory);
        function decimals() external view returns (uint8);
        function totalSupply() external view returns (uint256);
        function balanceOf(address account) external view returns (uint256);

        // AccessControlEnumerable functions
        function addMinter(address account) public;
        function addBurner(address account) public;
        function removeMinter(address account) public;
        function removeBurner(address account) public;
        function getMinters() public view returns (address[] memory);
        function getBurners() public view returns (address[] memory);
        function mint(address to, uint256 amount) public;
        function burn(address from, uint256 amount) public;

        // Errors
        error AccessControlUnauthorizedAccount(address account, bytes32 neededRole);
        error AccessControlBadConfirmation();
        error AccessControlEnumerableOutOfBounds(bytes32 role, uint256 index);

        // Events
        #[derive(Debug, PartialEq)]
        event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender);
        #[derive(Debug, PartialEq)]
        event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender);
        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 value);
    }
}
