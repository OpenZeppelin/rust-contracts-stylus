
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;


// /// @title ERC20 Interface
// /// @notice Defines the standard functions and events for ERC20 tokens
// interface IERC20 {
//     /// @notice Returns the total token supply
//     /// @return The total supply of tokens
//     function totalSupply() external view returns (uint256);

//     /// @notice Returns the token balance of a given account
//     /// @param account The address of the account to query
//     /// @return The token balance of the specified account
//     function balanceOf(address account) external view returns (uint256);

//     /// @notice Transfers tokens to a specified address
//     /// @param to The address of the recipient
//     /// @param amount The number of tokens to transfer
//     /// @return A boolean value indicating whether the transfer was successful
//     function transfer(address to, uint256 amount) external returns (bool);

//     /// @notice Returns the remaining number of tokens that `spender` is allowed to spend on behalf of `owner`
//     /// @param owner The address which owns the tokens
//     /// @param spender The address which is allowed to spend the tokens
//     /// @return The remaining number of tokens that `spender` can spend
//     function allowance(address owner, address spender) external view returns (uint256);

//     /// @notice Approves `spender` to spend a specified number of tokens on behalf of the caller
//     /// @param spender The address which is allowed to spend the tokens
//     /// @param amount The number of tokens to approve
//     /// @return A boolean value indicating whether the approval was successful
//     function approve(address spender, uint256 amount) external returns (bool);

//     /// @notice Transfers tokens from one address to another using the allowance mechanism
//     /// @param from The address to transfer tokens from
//     /// @param to The address to transfer tokens to
//     /// @param amount The number of tokens to transfer
//     /// @return A boolean value indicating whether the transfer was successful
//     function transferFrom(
//         address from,
//         address to,
//         uint256 amount
//     ) external returns (bool);

//     /// @notice Emitted when tokens are transferred from one address to another
//     /// @param from The address tokens are transferred from
//     /// @param to The address tokens are transferred to
//     /// @param value The number of tokens transferred
//     event Transfer(address indexed from, address indexed to, uint256 value);

//     /// @notice Emitted when the allowance of a `spender` for an `owner` is set by a call to `approve`
//     /// @param owner The address which owns the tokens
//     /// @param spender The address which is allowed to spend the tokens
//     /// @param value The number of tokens approved
//     event Approval(address indexed owner, address indexed spender, uint256 value);
// }


// /// @title ERC20 Metadata Interface
// /// @notice Extends the ERC20 standard to include token metadata functions
// interface IERC20Metadata is IERC20 {
//     /// @notice Returns the name of the token
//     /// @return The token name
//     function name() external view returns (string memory);

//     /// @notice Returns the symbol of the token
//     /// @return The token symbol
//     function symbol() external view returns (string memory);

//     /// @notice Returns the decimals places of the token
//     /// @return The number of decimals for the token
//     function decimals() external view returns (uint8);
// }

contract Erc4626Example {
    address private  _asset;
    uint8  private  _underlyingDecimals;
    string private _name;
    string private _symbol;


    constructor(address asset_, string memory name_, string memory symbol_) {
        _underlyingDecimals = 18;
        _asset = asset_;
        _name = name_;
        _symbol = symbol_;
    }
}
