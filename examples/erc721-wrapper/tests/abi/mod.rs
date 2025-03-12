#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc721Wrapper {
        function name() external view returns (string name);
        function symbol() external view returns (string symbol);
        function totalSupply() external view returns (uint256 totalSupply);
        function balanceOf(address account) external view returns (uint256 balance);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256 allowance);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        #[derive(Debug)]
        function decimals() external view returns (uint8 decimals);
        #[derive(Debug)]
        function underlying() external view returns (address underlying);
        #[derive(Debug)]
        function depositFor(address account, uint256 value) external  returns (bool);
        #[derive(Debug)]
        function withdrawTo(address account, uint256 value) external  returns (bool);

        error ERC721InvalidOwner(address owner);
        error ERC721NonexistentToken(uint256 tokenId);
        error ERC721IncorrectOwner(address sender, uint256 tokenId, address owner);
        error ERC721InvalidSender(address sender);
        error ERC721InvalidReceiver(address receiver);
        error ERC721InsufficientApproval(address operator, uint256 tokenId);
        error ERC721InvalidApprover(address approver);
        error ERC721InvalidOperator(address operator);

        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 value);
        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }
);
