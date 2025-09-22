#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc20FlashMint {
        function totalSupply() external view returns (uint256 totalSupply);
        function balanceOf(address account) external view returns (uint256 balance);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256 allowance);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        function mint(address account, uint256 amount) external;

        function maxFlashLoan(address token) external view returns (uint256 maxLoan);
        #[derive(Debug)]
        function flashFee(address token, uint256 amount) external view returns (uint256 fee);
        function flashLoan(address receiver, address token, uint256 amount, bytes calldata data) external returns (bool);

        function setFlashFeeReceiver(address newReceiver) external;
        function setFlashFeeValue(uint256 newValue) external;

        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
        error ERC20InvalidSender(address sender);
        error ERC20InvalidReceiver(address receiver);
        error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
        error ERC20InvalidSpender(address spender);

        error ERC3156UnsupportedToken(address token);
        error ERC3156ExceededMaxLoan(uint256 maxLoan);
        error ERC3156InvalidReceiver(address receiver);

        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 value);
        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }
);
