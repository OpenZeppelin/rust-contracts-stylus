#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc6909 {
        function balanceOf(address owner, uint256 id) external view returns (uint256 balance);
        function allowance(address owner, address spender, uint256 id) external view returns (uint256 allowance);
        function isOperator(address owner, address spender) external view returns (bool approved);
        function approve(address spender, uint256 id, uint256 amount) external returns (bool);
        function setOperator(address spender, bool approved) external returns (bool);
        function transfer(address receiver, uint256 id, uint256 amount) external returns (bool);
        function transferFrom(address sender, address receiver, uint256 id, uint256 amount) external returns (bool);

        function mint(address to, uint256 id, uint256 amount) external;
        function burn(address from, uint256 id, uint256 amount) external;

        function supportsInterface(bytes4 interface_id) external view returns (bool supportsInterface);

        error ERC6909InsufficientBalance(address sender, uint256 balance, uint256 needed, uint256 id);
        error ERC6909InsufficientAllowance(address spender, uint256 allowance, uint256 needed, uint256 id);
        error ERC6909InvalidApprover(address approver);
        error ERC6909InvalidReceiver(address receiver);
        error ERC6909InvalidSender(address sender);
        error ERC6909InvalidSpender(address spender);

        error Error(string message);
        error Panic(uint256 code);

        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed spender, uint256 indexed id, uint256 amount);
        #[derive(Debug, PartialEq)]
        event OperatorSet(address indexed owner, address indexed spender, bool approved);
        #[derive(Debug, PartialEq)]
        event Transfer(address caller, address indexed sender, address indexed receiver, uint256 indexed id, uint256 amount);
    }
);
