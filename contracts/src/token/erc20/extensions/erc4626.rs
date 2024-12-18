use alloy_primitives::{ keccak256, Address, B256, U256};
use alloy_sol_macro::sol;
use stylus_sdk::{
    block,
    prelude::StorageType,
    storage::TopLevelStorage,
    stylus_proc::{public, sol_storage, SolidityError},
};
use alloc::string::String;
use crate::token::erc20::{self, Erc20, IErc20};



/// `ierc4626_metadata` is a Rust trait that defines an interface for interacting with
/// an ERC-4626 compliant tokenized vault. This standard is designed for yield-bearing
/// vaults that represent ownership in underlying assets and facilitates deposit, mint, 
/// withdraw, and redeem operations.
///
/// The trait methods provide mechanisms to query metadata, perform calculations related
/// to shares and assets, and interact with the vault efficiently.
pub trait IERC4626 {
    /// Error type associated with the operations, convertible to a vector of bytes.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the address of the underlying asset that the vault manages.
    fn asset(self) -> Address;

    /// Returns the total amount of the underlying asset held in the vault.
    fn total_assets(self) -> U256;

    /// Converts a given amount of assets into the equivalent number of shares.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset.
    ///
    /// # Returns
    /// The corresponding amount of shares.
    fn convert_to_shares(assets: U256) -> U256;

    /// Converts a given number of shares into the equivalent amount of assets.
    ///
    /// # Parameters
    /// - `shares`: Number of shares.
    ///
    /// # Returns
    /// The corresponding amount of assets.
    fn convert_to_assets(shares: U256) -> U256;

    /// Calculates the maximum amount of assets that can be deposited for a given receiver.
    ///
    /// # Parameters
    /// - `receiver`: The address of the entity receiving the shares.
    ///
    /// # Returns
    /// The maximum depositable amount.
    fn max_deposit(receiver: Address) -> U256;

    /// Previews the outcome of depositing a specific amount of assets.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset to deposit.
    ///
    /// # Returns
    /// The number of shares that would be issued.
    fn preview_deposit(assets: U256) -> U256;

    /// Deposits a specific amount of assets into the vault, issuing shares to the receiver.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset to deposit.
    /// - `receiver`: The address receiving the shares.
    ///
    /// # Returns
    /// The number of shares issued.
    fn deposit(assets: U256, receiver: Address) -> U256;

    /// Calculates the maximum number of shares that can be minted for a given receiver.
    ///
    /// # Parameters
    /// - `receiver`: The address of the entity receiving the shares.
    ///
    /// # Returns
    /// The maximum mintable number of shares.
    fn max_mint(receiver: Address) -> U256;

    /// Previews the outcome of minting a specific number of shares.
    ///
    /// # Parameters
    /// - `shares`: Number of shares to mint.
    ///
    /// # Returns
    /// The equivalent amount of assets required.
    fn preview_mint(shares: U256) -> U256;

    /// Mints a specific number of shares for a given receiver.
    ///
    /// # Parameters
    /// - `shares`: Number of shares to mint.
    /// - `receiver`: The address receiving the shares.
    ///
    /// # Returns
    /// The amount of assets deposited.
    fn mint(shares: U256, receiver: Address) -> U256;

    /// Calculates the maximum amount of assets that can be withdrawn by a given owner.
    ///
    /// # Parameters
    /// - `owner`: The address of the entity owning the shares.
    ///
    /// # Returns
    /// The maximum withdrawable amount.
    fn max_withdraw(owner: Address) -> U256;

    /// Previews the outcome of withdrawing a specific amount of assets.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset to withdraw.
    ///
    /// # Returns
    /// The equivalent number of shares required.
    fn preview_withdraw(assets: U256) -> U256;

    /// Withdraws a specific amount of assets from the vault, deducting shares from the owner.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset to withdraw.
    /// - `receiver`: The address receiving the withdrawn assets.
    /// - `owner`: The address owning the shares to be deducted.
    ///
    /// # Returns
    /// The number of shares burned.
    fn withdraw(assets: U256, receiver: Address, owner: Address) -> U256;

    /// Calculates the maximum number of shares that can be redeemed by a given owner.
    ///
    /// # Parameters
    /// - `owner`: The address of the entity owning the shares.
    ///
    /// # Returns
    /// The maximum redeemable number of shares.
    fn max_redeem(owner: Address) -> U256;

    /// Previews the outcome of redeeming a specific number of shares.
    ///
    /// # Parameters
    /// - `shares`: Number of shares to redeem.
    ///
    /// # Returns
    /// The equivalent amount of assets returned.
    fn preview_redeem(shares: U256) -> U256;

    /// Redeems a specific number of shares for the underlying assets, transferring them to the receiver.
    ///
    /// # Parameters
    /// - `shares`: Number of shares to redeem.
    /// - `receiver`: The address receiving the underlying assets.
    /// - `owner`: The address owning the shares to be redeemed.
    ///
    /// # Returns
    /// The amount of assets transferred.
    fn redeem(shares: U256, receiver: Address, owner: Address) -> U256;
}


sol_storage! {
    pub struct ERC4626 {
        Erc20  _asset;
        uint8  _underlying_decimals;
    }
}

sol! {
    event Deposit(address indexed sender, address indexed owner, uint256 assets, uint256 shares);

    event Withdraw(
        address indexed sender,
        address indexed receiver,
        address indexed owner,
        uint256 assets,
        uint256 shares
    );
}

sol! {
    #[derive(Debug)]
    #[allow(missing_docs)]
     error ERC4626ExceededMaxDeposit(address receiver, uint256 assets, uint256 max);

    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC4626ExceededMaxMint(address receiver, uint256 shares, uint256 max);

    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC4626ExceededMaxWithdraw(address owner, uint256 assets, uint256 max);

    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC4626ExceededMaxRedeem(address owner, uint256 shares, uint256 max);
}

#[derive(SolidityError, Debug)]
pub enum Error {
    ExceededMaxDeposit(ERC4626ExceededMaxDeposit),
    ExceededMaxMint(ERC4626ExceededMaxMint),
    ExceededMaxWithdraw(ERC4626ExceededMaxWithdraw),
    ExceededMaxRedeem(ERC4626ExceededMaxRedeem),
    /// Error type from [`Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error)
}



