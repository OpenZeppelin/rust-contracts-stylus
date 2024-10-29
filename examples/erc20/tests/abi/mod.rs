#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc20 {
        function name() external view returns (string name);
        function symbol() external view returns (string symbol);
        function decimals() external view returns (uint8 decimals);
        function totalSupply() external view returns (uint256 totalSupply);
        function balanceOf(address account) external view returns (uint256 balance);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256 allowance);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        function cap() public view virtual returns (uint256 cap);

        function mint(address account, uint256 amount) external;
        function burn(uint256 amount) external;
        function burnFrom(address account, uint256 amount) external;

        function paused() external view returns (bool paused);
        function pause() external;
        function unpause() external;

        function supportsInterface(bytes4 interface_id) external view returns (bool supportsInterface);

        error EnforcedPause();
        error ExpectedPause();

        error ERC20ExceededCap(uint256 increased_supply, uint256 cap);
        error ERC20InvalidCap(uint256 cap);

        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
        error ERC20InvalidSender(address sender);
        error ERC20InvalidReceiver(address receiver);
        error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
        error ERC20InvalidSpender(address spender);

        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 value);
        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed spender, uint256 value);

        #[derive(Debug, PartialEq)]
        event Paused(address account);
        #[derive(Debug, PartialEq)]
        event Unpaused(address account);
    }
);
