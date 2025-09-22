#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract UUPSProxyErc20Example {
        error InvalidInitialization();
        error InvalidVersion(uint32 current_version);
        error UUPSUnauthorizedCallContext();
        error UUPSUnsupportedProxiableUUID(bytes32 slot);
        error ERC1967InvalidImplementation(address implementation);

        function upgradeToAndCall(address newImplementation, bytes calldata data) external payable;
        function UPGRADE_INTERFACE_VERSION() external view returns (string version);
        function initialize(address owner) external;
        function getVersion() external pure returns (uint32 version);
        #[derive(Debug)]
        function setVersion() external;

        // ERC1822 proxiable function
        function proxiableUUID() external view returns (bytes32 uuid);
    }

    #[sol(rpc)]
    contract Erc1967Example {
        function implementation() public view returns (address implementation);

        // ERC20 functions that we want to delegate to the implementation.
        function totalSupply() external view returns (uint256 totalSupply);
        function balanceOf(address account) external view returns (uint256 balance);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256 allowance);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        function mint(address account, uint256 value) external;

        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
        error ERC20InvalidSender(address sender);
        error ERC20InvalidReceiver(address receiver);
        error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
        error ERC20InvalidSpender(address spender);

        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 value);
        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed spender, uint256 value);

        // UUPS upgrade function
        function upgradeToAndCall(address newImplementation, bytes calldata data) external payable;
        function UPGRADE_INTERFACE_VERSION() external view returns (string version);
        function initialize(address owner) external;
        function getVersion() external pure returns (uint32 version);

        // ERC1822 proxiable function
        #[derive(Debug)]
        function proxiableUUID() external view returns (bytes32 uuid);

        // Ownable function
        function owner() public view returns (address owner);
        function transferOwnership(address newOwner) public;
        function renounceOwnership() public;

        #[derive(Debug, PartialEq)]
        event Upgraded(address indexed implementation);

        error ERC1967NonPayable();

        error FailedCallWithReason(bytes reason);

    }
);
