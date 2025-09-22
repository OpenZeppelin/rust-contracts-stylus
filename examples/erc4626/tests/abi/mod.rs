#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc4626 {
        function decimals() external view  returns (uint8 decimals);

        function totalSupply() external view returns (uint256 totalSupply);
        function balanceOf(address account) external view returns (uint256 balance);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256 allowance);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        function asset() external view  returns (address asset);
        #[derive(Debug)]
        function totalAssets() external view returns (uint256 totalAssets);
        #[derive(Debug)]
        function convertToShares(uint256 assets) external view  returns (uint256 shares);
        #[derive(Debug)]
        function convertToAssets(uint256 shares) external view  returns (uint256 assets);
        #[derive(Debug)]
        function maxMint(address) external view  returns (uint256 maxMint);
        #[derive(Debug)]
        function maxDeposit(address) external view  returns (uint256 maxDeposit);
        #[derive(Debug) ]
        function maxWithdraw(address owner) external view  returns (uint256 maxWithdraw);
        #[derive(Debug)]
        function maxRedeem(address owner) external view  returns (uint256 maxRedeem);
        #[derive(Debug)]
        function previewDeposit(uint256 assets) external view  returns (uint256 shares);
        #[derive(Debug)]
        function previewMint(uint256 shares) external view  returns (uint256 assets);
        #[derive(Debug)]
        function previewRedeem(uint256 shares) external view  returns (uint256 assets);
        #[derive(Debug)]
        function previewWithdraw(uint256 assets) external view  returns (uint256 shares);
        function deposit(uint256 assets, address receiver) external  returns (uint256);
        function mint(uint256 shares, address receiver) external  returns (uint256);
        function redeem(uint256 shares, address receiver,address owner) external returns (uint256);
        function withdraw(uint256 assets, address receiver,address owner) external returns (uint256);

        error SafeErc20FailedOperation(address token);

        error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
        error ERC20InvalidApprover(address approver);

        error ERC4626ExceededMaxMint(address receiver, uint256 shares, uint256 max);
        error ERC4626ExceededMaxDeposit(address receiver, uint256 assets, uint256 max);
        error ERC4626ExceededMaxWithdraw(address owner, uint256 assets, uint256 max);
        error ERC4626ExceededMaxRedeem(address owner, uint256 shares, uint256 max);
        error InvalidAsset(address asset);

        #[derive(Debug, PartialEq)]
        event Deposit(address indexed sender, address indexed owner, uint256 assets, uint256 shares);
        #[derive(Debug, PartialEq)]
        event Withdraw(address indexed sender,address indexed receiver,address indexed owner,uint256 assets, uint256 shares);
    }
);
