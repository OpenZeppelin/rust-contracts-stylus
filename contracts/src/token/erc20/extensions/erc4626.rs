//! ERC-4626 Tokenized Vault Standard Implementation as defined in [ERC-4626].
//!
//! This extension allows the minting and burning of "shares" (represented using
//! the ERC-20 inheritance) in exchange for underlying "assets" through
//! standardized `deposit`, `mint`, `redeem` and `burn` workflows. This contract
//! extends the ERC-20 standard. Any additional extensions included along it
//! would affect the "shares" token represented by this contract and not the
//! "assets" token which is an independent contract.

use alloc::{vec, vec::Vec};
use core::ops::{Deref, DerefMut};

use alloy_primitives::{aliases::B32, uint, Address, U256, U8};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    call::{Call, MethodError},
    contract, evm, msg,
    prelude::*,
    storage::{
        StorageAddress, StorageMap, StorageString, StorageU256, StorageU8,
    },
};

use super::IErc20Metadata;
use crate::{
    token::erc20::{
        self,
        interface::{Erc20Interface, IErc20MetadataInterface},
        utils::{safe_erc20, ISafeErc20},
        Erc20, Erc20Internal, Erc20MetadataStorage, Erc20Storage, IErc20,
    },
    utils::{
        introspection::erc165::IErc165,
        math::alloy::{Math, Rounding},
    },
};

const ONE: U256 = uint!(1_U256);
const TEN: U256 = uint!(10_U256);

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when assets are deposited into the contract.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event Deposit(address indexed sender, address indexed owner, uint256 assets, uint256 shares);

        /// Emitted when assets are withdrawn from the contract.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event Withdraw(
            address indexed sender,
            address indexed receiver,
            address indexed owner,
            uint256 assets,
            uint256 shares
        );
    }

    sol! {
        /// Indicates an attempt to deposit more assets than the max amount for
        /// `receiver`.
        ///
        /// * `receiver` - Address of the asset's recipient.
        /// * `assets` - Amount of assets deposited.
        /// * `max` - Maximum amount of assets that can be deposited.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC4626ExceededMaxDeposit(address receiver, uint256 assets, uint256 max);

        /// Indicates an attempt to mint more shares than the max amount for
        /// `receiver`.
        ///
        /// * `receiver` - Address of share's recipient.
        /// * `shares` - Amount of shares to mint.
        /// * `max` - Maximum amount of shares that can be minted.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC4626ExceededMaxMint(address receiver, uint256 shares, uint256 max);

        /// Indicates an attempt to withdraw more assets than the max amount for
        /// `owner`.
        ///
        /// * `owner` - Address of the asset's owner.
        /// * `assets` - Amount of assets to withdraw.
        /// * `max` - Maximum amount of assets that can be withdrawn.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC4626ExceededMaxWithdraw(address owner, uint256 assets, uint256 max);

        /// Indicates an attempt to redeem more shares than the max amount for
        /// `owner`.
        ///
        /// * `owner` - Address of the share's owner.
        /// * `shares` - Amount of shares to redeem.
        /// * `max` - Maximum amount of shares that can be redeemed.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC4626ExceededMaxRedeem(address owner, uint256 shares, uint256 max);

        /// The address is not a valid ERC-20 token.
        ///
        /// * `asset` - Address of the invalid ERC-20 token.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error InvalidAsset(address asset);
    }
}

/// An [`Erc4626`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an attempt to deposit more assets than the max amount for
    /// `receiver`.
    ExceededMaxDeposit(ERC4626ExceededMaxDeposit),
    /// Indicates an attempt to mint more shares than the max amount for
    /// `receiver`.
    ExceededMaxMint(ERC4626ExceededMaxMint),
    /// Indicates an attempt to withdraw more assets than the max amount for
    /// `owner`.
    ExceededMaxWithdraw(ERC4626ExceededMaxWithdraw),
    /// Indicates an attempt to redeem more shares than the max amount for
    /// `owner`.
    ExceededMaxRedeem(ERC4626ExceededMaxRedeem),
    /// The address is not a valid ERC-20 token.
    InvalidAsset(InvalidAsset),
    /// An operation with an ERC-20 token failed.
    SafeErc20FailedOperation(safe_erc20::SafeErc20FailedOperation),
    /// Indicates a failed [`ISafeErc20::safe_decrease_allowance`] request.
    SafeErc20FailedDecreaseAllowance(
        safe_erc20::SafeErc20FailedDecreaseAllowance,
    ),
    /// Indicates an error related to the current balance of `sender`. Used in
    /// transfers.
    InsufficientBalance(erc20::ERC20InsufficientBalance),
    /// Indicates a failure with the token `sender`. Used in transfers.
    InvalidSender(erc20::ERC20InvalidSender),
    /// Indicates a failure with the token `receiver`. Used in transfers.
    InvalidReceiver(erc20::ERC20InvalidReceiver),
    /// Indicates a failure with the `spender`’s `allowance`. Used in
    /// transfers.
    InsufficientAllowance(erc20::ERC20InsufficientAllowance),
    /// Indicates a failure with the `spender` to be approved. Used in
    /// approvals.
    InvalidSpender(erc20::ERC20InvalidSpender),
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals. approver Address initiating an approval operation.
    InvalidApprover(erc20::ERC20InvalidApprover),
}

impl From<safe_erc20::Error> for Error {
    fn from(value: safe_erc20::Error) -> Self {
        match value {
            safe_erc20::Error::SafeErc20FailedOperation(e) => {
                Error::SafeErc20FailedOperation(e)
            }
            safe_erc20::Error::SafeErc20FailedDecreaseAllowance(e) => {
                Error::SafeErc20FailedDecreaseAllowance(e)
            }
        }
    }
}

impl From<erc20::Error> for Error {
    fn from(value: erc20::Error) -> Self {
        match value {
            erc20::Error::InsufficientBalance(e) => {
                Error::InsufficientBalance(e)
            }
            erc20::Error::InvalidSender(e) => Error::InvalidSender(e),
            erc20::Error::InvalidReceiver(e) => Error::InvalidReceiver(e),
            erc20::Error::InsufficientAllowance(e) => {
                Error::InsufficientAllowance(e)
            }
            erc20::Error::InvalidSpender(e) => Error::InvalidSpender(e),
            erc20::Error::InvalidApprover(e) => Error::InvalidApprover(e),
        }
    }
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// Storage of an [`Erc4626`] contract.
pub trait Erc4626Storage: TopLevelStorage {
    /// Return the address of the underlying token used for the Vault for
    /// accounting, depositing, and withdrawing.
    fn asset(&self) -> &StorageAddress;
    /// Return mutable address of the underlying token used for the Vault for
    /// accounting, depositing, and withdrawing.
    fn asset_mut(&mut self) -> &mut StorageAddress;
    /// Token decimals.
    fn underlying_decimals(&self) -> &StorageU8;
    /// Return mutable token decimals.
    fn underlying_decimals_mut(&mut self) -> &mut StorageU8;
}

/// ERC-4626 Tokenized Vault Standard Interface
#[interface_id]
pub trait IErc4626: Erc4626Internal {
    /// Returns the address of the underlying token used for the Vault for
    /// accounting, depositing, and withdrawing.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[must_use]
    fn asset(&self) -> Address {
        self._asset()
    }

    /// Returns the total amount of the underlying asset that is “managed” by
    /// Vault.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the [`IErc4626::asset()`] is not a ERC-20
    ///   Token address.
    fn total_assets(&self) -> Result<U256, Vec<u8>> {
        self._total_assets()
    }

    /// Returns the amount of shares that the Vault would exchange for the
    /// amount of assets provided, in an ideal scenario where all the conditions
    /// are met.
    ///
    /// NOTE:
    /// - This calculation MAY NOT reflect the “per-user” price-per-share, and
    ///   instead should reflect the “average-user’s” price-per-share, meaning
    ///   what the average user should expect to see when exchanging to and
    ///   from.
    /// - To expose this function in your contract's ABI, implement it as shown
    ///   in the Examples section below, accepting only the `assets` parameter.
    ///   The `erc20` reference should come from your contract's state. The
    ///   implementation should forward the call to your internal storage
    ///   instance along with the `erc20` reference.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `assets` - Amount of the underlying asset.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the [`IErc4626::asset()`] is not an
    ///   ERC-20 Token address.
    ///
    /// # Panics
    ///
    /// * If decimal offset calculation overflows.
    /// * If multiplication or division operations overflow.
    fn convert_to_shares(&self, assets: U256) -> Result<U256, Vec<u8>> {
        self._convert_to_shares(assets)
    }

    /// Returns the amount of assets that the Vault would exchange for the
    /// amount of shares provided, in an ideal scenario where all the conditions
    /// are met.
    ///
    /// NOTE:
    /// - This calculation MAY NOT reflect the “per-user” price-per-share, and
    ///   instead should reflect the “average-user’s” price-per-share, meaning
    ///   what the average user should expect to see when exchanging to and
    ///   from.
    /// - To expose this function in your contract's ABI, implement it as shown
    ///   in the Examples section below, accepting only the `shares` parameter.
    ///   The `erc20` reference should come from your contract's state. The
    ///   implementation should forward the call to your internal storage
    ///   instance along with the `erc20` reference.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `shares` - Number of shares.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the [`IErc4626::asset()`] is not an
    ///   ERC-20 Token address.
    ///
    /// # Panics
    ///
    /// * If decimal offset calculation overflows.
    /// * If multiplication or division operations overflow.
    fn convert_to_assets(&self, shares: U256) -> Result<U256, Vec<u8>> {
        self._convert_to_assets(shares)
    }

    /// Returns the maximum amount of the underlying asset that can be deposited
    /// into the Vault for the receiver, through a deposit call.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `receiver` - The address of the entity receiving the shares.
    #[must_use]
    fn max_deposit(&self, receiver: Address) -> U256 {
        self._max_deposit(receiver)
    }

    /// Allows an on-chain or off-chain user to simulate the effects of their
    /// deposit at the current block, given current on-chain conditions.
    ///
    /// NOTE:
    /// - Any unfavorable discrepancy between [`IErc4626::convert_to_shares`]
    ///   and [`IErc4626::preview_deposit`] SHOULD be considered slippage in
    ///   share price or some other type of condition, meaning the depositor
    ///   will lose assets by depositing.
    /// - To expose this function in your contract's ABI, implement it as shown
    ///   in the Examples section below, accepting only the `assets` parameter.
    ///   The `erc20` reference should come from your contract's state. The
    ///   implementation should forward the call to your internal storage
    ///   instance along with the `erc20` reference.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `assets` - Amount of the underlying asset to deposit.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the [`IErc4626::asset()`] is not an
    ///   ERC-20 Token address.
    ///
    /// # Panics
    ///
    /// * If decimal offset calculation overflows.
    /// * If multiplication or division operations overflow during conversion.
    fn preview_deposit(&self, assets: U256) -> Result<U256, Vec<u8>> {
        self._preview_deposit(assets)
    }

    /// Deposits exactly `assets` amount of underlying tokens into the Vault and
    /// mints corresponding Vault shares to `receiver`.
    ///
    /// Returns the amount of shares minted.
    ///
    /// NOTE: To expose this function in your contract's ABI, implement it as
    /// shown in the Examples section below, accepting only the `assets` and
    /// `receiver` parameters. The `erc20` reference should come from your
    /// contract's state. The implementation should forward the call to your
    /// internal storage instance along with the `erc20` reference.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `assets` - Amount of the underlying asset to deposit.
    /// * `receiver` - The address receiving the shares.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the [`IErc4626::asset()`] is not an
    ///   ERC-20 Token address.
    /// * [`Error::ExceededMaxDeposit`] - If deposit amount exceeds maximum
    ///   allowed.
    /// * [`safe_erc20::Error::SafeErc20FailedOperation`] - If caller lacks
    ///   sufficient balance or hasn't approved enough tokens to the Vault
    ///   contract.
    ///
    /// # Events
    ///
    /// * [`Deposit`]
    ///
    /// # Panics
    ///
    /// * If decimal offset calculation overflows.
    /// * If multiplication or division operations overflow during conversion.
    fn deposit(
        &mut self,
        assets: U256,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        self._deposit(assets, receiver)
    }

    /// Returns the maximum amount of the Vault shares that can be minted for
    /// the receiver, through a mint call.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `receiver` - The address of the entity receiving the shares.
    #[must_use]
    fn max_mint(&self, receiver: Address) -> U256 {
        self._max_mint(receiver)
    }

    /// Allows an on-chain or off-chain user to simulate the effects of their
    /// mint at the current block, given current on-chain conditions.
    ///
    /// NOTE:
    /// - Any unfavorable discrepancy between [`IErc4626::convert_to_assets`]
    ///   and [`IErc4626::preview_mint`] SHOULD be considered slippage in share
    ///   price or some other type of condition, meaning the depositor will lose
    ///   assets by minting.
    /// - To expose this function in your contract's ABI, implement it as shown
    ///   in the Examples section below, accepting only the `shares` parameter.
    ///   The `erc20` reference should come from your contract's state. The
    ///   implementation should forward the call to your internal storage
    ///   instance along with the `erc20` reference.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `shares` - Number of shares to mint.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the [`IErc4626::asset()`] is not an
    ///   ERC-20 Token address.
    ///
    /// # Panics
    ///
    /// * If decimal offset calculation overflows.
    /// * If multiplication or division operations overflow during conversion.
    fn preview_mint(&self, shares: U256) -> Result<U256, Vec<u8>> {
        self._preview_mint(shares)
    }

    /// Mints the specified number of shares to `receiver` by pulling the
    /// required amount of underlying tokens from caller.
    ///
    /// Returns amount of tokens deposited.
    ///
    /// NOTE:
    /// - Most implementations will require pre-approval of the Vault with the
    ///   Vault’s underlying asset token.
    /// - To expose this function in your contract's ABI, implement it as shown
    ///   in the Examples section below, accepting only the `shares` and
    ///   `receiver` parameters. The `erc20` reference should come from your
    ///   contract's state. The implementation should forward the call to your
    ///   internal storage instance along with the `erc20` reference.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `shares` - Number of shares to mint.
    /// * `receiver` - The address receiving the shares.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the [`IErc4626::asset()`] is not an
    ///   ERC-20 Token address.
    /// * [`Error::ExceededMaxMint`] - If requested shares amount exceeds
    ///   maximum mintable amount for `receiver`.
    /// * [`safe_erc20::Error::SafeErc20FailedOperation`] - If caller lacks
    ///   sufficient balance or hasn't approved enough tokens to the Vault
    ///   contract.
    ///
    /// # Events
    ///
    /// * [`Deposit`]
    ///
    /// # Panics
    ///
    /// * If decimal offset calculation overflows.
    /// * If multiplication or division operations overflow during conversion.
    fn mint(
        &mut self,
        shares: U256,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        Erc4626Internal::_mint(self, shares, receiver)
    }

    /// Returns the maximum amount of the underlying asset that can be withdrawn
    /// from the owner balance in the Vault, through a withdraw call.
    ///
    /// NOTE: To expose this function in your contract's ABI, implement it as
    /// shown in the Examples section below, accepting only the `owner`
    /// parameter. The `erc20` reference should come from your contract's state.
    /// The implementation should forward the call to your internal storage
    /// instance along with the `erc20` reference.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - The address of the entity owning the shares.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the [`IErc4626::asset()`] is not an
    ///   ERC-20 Token address.
    ///
    /// # Panics
    ///
    /// * If decimal offset calculation overflows.
    /// * If multiplication or division operations overflow during conversion.
    fn max_withdraw(&self, owner: Address) -> Result<U256, Vec<u8>> {
        self._max_withdraw(owner)
    }

    /// Allows an on-chain or off-chain user to simulate the effects of their
    /// withdrawal at the current block, given current on-chain conditions.
    ///
    /// NOTE: To expose this function in your contract's ABI, implement it as
    /// shown in the Examples section below, accepting only the `assets`
    /// parameter. The `erc20` reference should come from your contract's state.
    /// The implementation should forward the call to your internal storage
    /// instance along with the `erc20` reference.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `assets` - Amount of the underlying asset to withdraw.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the [`IErc4626::asset()`] is not an
    ///   ERC-20 Token address.
    ///
    /// # Panics
    ///
    /// * If decimal offset calculation overflows.
    /// * If multiplication or division operations overflow during conversion.
    fn preview_withdraw(&self, assets: U256) -> Result<U256, Vec<u8>> {
        self._preview_withdraw(assets)
    }

    /// Withdraws the specified amount of underlying tokens to `receiver` by
    /// burning the required number of shares from `owner`.
    ///
    /// Returns number of shares burned.
    ///
    /// NOTE:
    /// - Some implementations will require pre-requesting to the Vault before a
    ///   withdrawal may be performed. Those methods should be performed
    ///   separately.
    /// - To expose this function in your contract's ABI, implement it as shown
    ///   in the Examples section below, accepting only the `assets`, `receiver`
    ///   and `owner` parameters. The `erc20` reference should come from your
    ///   contract's state. The implementation should forward the call to your
    ///   internal storage instance along with the `erc20` reference.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `assets` - Amount of the underlying asset to withdraw.
    /// * `receiver` - The address receiving the withdrawn assets.
    /// * `owner` - The address owning the shares to be deducted.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the [`IErc4626::asset()`] is not an
    ///   ERC-20 Token address.
    /// * [`Error::ExceededMaxWithdraw`] - If requested assets amount exceeds
    ///   maximum withdrawable amount for `owner`.
    /// * [`erc20::Error::InsufficientAllowance`] - If caller is not `owner` and
    ///   lacks sufficient allowance for shares.
    /// * [`erc20::Error::InvalidApprover`] - If `owner` address is
    ///   [`Address::ZERO`] when burning shares.
    /// * [`safe_erc20::Error::SafeErc20FailedOperation`] - If underlying token
    ///   transfer fails or returns false.
    ///
    /// # Events
    ///
    /// * [`Withdraw`]
    ///
    /// # Panics
    ///
    /// * If decimal offset calculation overflows.
    /// * If multiplication or division operations overflow during conversion.
    fn withdraw(
        &mut self,
        assets: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Vec<u8>> {
        self._withdraw(assets, receiver, owner)
    }

    /// Returns the maximum amount of Vault shares that can be redeemed from the
    /// owner balance in the Vault, through a redeem call.
    ///
    /// NOTE: To expose this function in your contract's ABI, implement it as
    /// shown in the Examples section below, accepting only the `owner`
    /// parameter. The `erc20` reference should come from your contract's state.
    /// The implementation should forward the call to your internal storage
    /// instance along with the `erc20` reference.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - The address of the entity owning the shares.
    #[must_use]
    fn max_redeem(&self, owner: Address) -> U256 {
        self._max_redeem(owner)
    }

    /// Allows an on-chain or off-chain user to simulate the effects of their
    /// redemption at the current block, given current on-chain conditions.
    ///
    /// NOTE: To expose this function in your contract's ABI, implement it as
    /// shown in the Examples section below, accepting only the `shares`
    /// parameter. The `erc20` reference should come from your contract's state.
    /// The implementation should forward the call to your internal storage
    /// instance along with the `erc20` reference.
    ///
    /// NOTE: Any unfavorable discrepancy between
    /// [`IErc4626::convert_to_assets`] and [`IErc4626::preview_redeem`] SHOULD
    /// be considered slippage in share price or some other type of condition,
    /// meaning the depositor will lose assets by redeeming.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `shares` - Number of shares to redeem.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the [`IErc4626::asset()`] is not an
    ///   ERC-20 Token address.
    ///
    /// # Panics
    ///
    /// * If decimal offset calculation overflows.
    /// * If multiplication or division operations overflow during conversion.
    fn preview_redeem(&self, shares: U256) -> Result<U256, Vec<u8>> {
        self._preview_redeem(shares)
    }

    /// Burns the specified number of shares from `owner` and sends the
    /// corresponding amount of underlying tokens to `receiver`.
    ///
    /// Returns amount of tokens transferred.
    ///
    /// NOTE: To expose this function in your contract's ABI, implement it as
    /// shown in the Examples section below, accepting only the `shares`,
    /// `receiver` and `owner` parameters. The `erc20` reference should come
    /// from your contract's state. The implementation should forward the call
    /// to your internal storage instance along with the `erc20` reference.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `shares` - Number of shares to redeem.
    /// * `receiver` - The address receiving the underlying assets.
    /// * `owner` - The address owning the shares to be redeemed.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the [`IErc4626::asset()`] is not an
    ///   ERC-20 Token address.
    /// * [`Error::ExceededMaxRedeem`] - If requested shares amount exceeds
    ///   maximum redeemable amount for owner.
    /// * [`erc20::Error::InsufficientAllowance`] - If caller is not `owner` and
    ///   lacks sufficient allowance for shares.
    /// * [`safe_erc20::Error::SafeErc20FailedOperation`] - If underlying token
    ///   transfer fails or returns false.
    ///
    /// # Events
    ///
    /// * [`Withdraw`]
    ///
    /// # Panics
    ///
    /// * If multiplication or division operations overflow during conversion.
    fn redeem(
        &mut self,
        shares: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Vec<u8>> {
        self._redeem(shares, receiver, owner)
    }
}

/// Internal trait for the ERC-4626 standard.
pub trait Erc4626Internal: Erc4626Storage + IErc20 + IErc20Metadata {
    /// See [`IErc4626::asset`].
    #[must_use]
    fn _asset(&self) -> Address {
        Erc4626Storage::asset(self).get()
    }

    /// See [`IErc4626::total_assets`].
    #[allow(clippy::missing_errors_doc)]
    fn _total_assets(&self) -> Result<U256, Vec<u8>> {
        let asset = Erc4626Storage::asset(self).get();
        let erc20 = Erc20Interface::new(asset);
        erc20
            .balance_of(Call::new(), contract::address())
            .map_err(|_| InvalidAsset { asset }.encode())
    }

    /// See [`IErc4626::convert_to_shares`].
    #[allow(clippy::missing_errors_doc)]
    fn _convert_to_shares(&self, assets: U256) -> Result<U256, Vec<u8>> {
        self._convert_to_shares_internal(assets, Rounding::Floor)
    }

    /// See [`IErc4626::convert_to_assets`].
    #[allow(clippy::missing_errors_doc)]
    fn _convert_to_assets(&self, shares: U256) -> Result<U256, Vec<u8>> {
        self._convert_to_assets_internal(shares, Rounding::Floor)
    }

    /// See [`IErc4626::max_deposit`].
    #[must_use]
    fn _max_deposit(&self, _receiver: Address) -> U256 {
        U256::MAX
    }

    /// See [`IErc4626::max_mint`].
    #[must_use]
    fn _max_mint(&self, _receiver: Address) -> U256 {
        U256::MAX
    }

    /// See [`IErc4626::max_withdraw`].
    #[allow(clippy::missing_errors_doc)]
    fn _max_withdraw(&self, owner: Address) -> Result<U256, Vec<u8>> {
        let balance = self.balance_of(owner);
        self._convert_to_assets_internal(balance, Rounding::Floor)
    }

    /// See [`IErc4626::max_redeem`].
    #[must_use]
    fn _max_redeem(&self, owner: Address) -> U256 {
        self.balance_of(owner)
    }

    /// See [`IErc4626::preview_deposit`].
    #[allow(clippy::missing_errors_doc)]
    fn _preview_deposit(&self, assets: U256) -> Result<U256, Vec<u8>> {
        self._convert_to_shares_internal(assets, Rounding::Floor)
    }

    /// See [`IErc4626::preview_mint`].
    #[allow(clippy::missing_errors_doc)]
    fn _preview_mint(&self, shares: U256) -> Result<U256, Vec<u8>> {
        self._convert_to_assets_internal(shares, Rounding::Ceil)
    }

    /// See [`IErc4626::preview_withdraw`].
    #[allow(clippy::missing_errors_doc)]
    fn _preview_withdraw(&self, assets: U256) -> Result<U256, Vec<u8>> {
        self._convert_to_shares_internal(assets, Rounding::Ceil)
    }

    /// See [`IErc4626::preview_redeem`].
    #[allow(clippy::missing_errors_doc)]
    fn _preview_redeem(&self, shares: U256) -> Result<U256, Vec<u8>> {
        self._convert_to_assets_internal(shares, Rounding::Floor)
    }

    /// See [`IErc4626::deposit`].
    #[allow(clippy::missing_errors_doc)]
    fn _deposit(
        &mut self,
        assets: U256,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        let max_assets = self._max_deposit(receiver);

        if assets > max_assets {
            return Err(Error::ExceededMaxDeposit(ERC4626ExceededMaxDeposit {
                receiver,
                assets,
                max: max_assets,
            })
            .encode());
        }

        let shares = self._preview_deposit(assets)?;

        self._deposit_internal(msg::sender(), receiver, assets, shares)?;

        Ok(shares)
    }

    /// See [`IErc4626::mint`].
    #[allow(clippy::missing_errors_doc)]
    fn _mint(
        &mut self,
        shares: U256,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        let max_shares = self._max_mint(receiver);

        if shares > max_shares {
            return Err(Error::ExceededMaxMint(ERC4626ExceededMaxMint {
                receiver,
                shares,
                max: max_shares,
            })
            .encode());
        }

        let assets = self._preview_mint(shares)?;
        self._deposit_internal(msg::sender(), receiver, assets, shares)?;

        Ok(assets)
    }

    /// See [`IErc4626::withdraw`].
    #[allow(clippy::missing_errors_doc)]
    fn _withdraw(
        &mut self,
        assets: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Vec<u8>> {
        let max_assets = self._max_withdraw(owner)?;

        if assets > max_assets {
            return Err(Error::ExceededMaxWithdraw(
                ERC4626ExceededMaxWithdraw { owner, assets, max: max_assets },
            )
            .encode());
        }

        let shares = self._preview_withdraw(assets)?;
        self._withdraw_internal(
            msg::sender(),
            receiver,
            owner,
            assets,
            shares,
        )?;

        Ok(shares)
    }

    /// See [`IErc4626::redeem`].
    #[allow(clippy::missing_errors_doc)]
    fn _redeem(
        &mut self,
        shares: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Vec<u8>> {
        let max_shares = self._max_redeem(owner);
        if shares > max_shares {
            return Err(Error::ExceededMaxRedeem(ERC4626ExceededMaxRedeem {
                owner,
                shares,
                max: max_shares,
            })
            .encode());
        }

        let assets = self._preview_redeem(shares)?;

        self._withdraw_internal(
            msg::sender(),
            receiver,
            owner,
            assets,
            shares,
        )?;

        Ok(assets)
    }

    /// Returns the number of decimals used in representing vault shares. Adds
    /// the decimals offset to the underlying token's decimals.
    ///
    /// NOTE: To expose this function in your contract's ABI, implement it as
    /// shown in the Examples section below. The implementation should forward
    /// the call to your internal storage instance.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Panics
    ///
    /// * When adding the offset decimals to the underlying token's decimals
    ///   would exceed [`U8::MAX`].
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    ///     fn decimals(&self) -> U8 {
    ///         self.decimals()
    ///     }
    /// ```
    #[must_use]
    fn _decimals(&self) -> U8 {
        self.underlying_decimals()
            .get()
            .checked_add(self._decimals_offset())
            .expect("Decimals should not be greater than `U8::MAX`")
    }

    /// Converts a given amount of assets to shares using the specified
    /// `rounding` mode.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `assets` - The amount of assets to convert.
    /// * `rounding` - The [`Rounding`] mode to use for the conversion.
    /// * `erc20` - Read access to an [`Erc20`] contract.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the token address is not a valid ERC-20
    ///   token.
    ///
    /// # Panics
    ///
    /// * If decimal offset calculation overflows in the power operation.
    /// * If multiplication or division operations overflow during conversion.
    fn _convert_to_shares_internal(
        &self,
        assets: U256,
        rounding: Rounding,
    ) -> Result<U256, Vec<u8>> {
        let total_supply = IErc20::total_supply(self);

        let multiplier = total_supply
            .checked_add(
                TEN.checked_pow(U256::from(self._decimals_offset())).expect(
                    "decimal offset overflow in `Erc4626::_convert_to_shares`",
                ),
            )
            .expect("multiplier overflow in `Erc4626::_convert_to_shares`");

        let denominator = self
            ._total_assets()?
            .checked_add(ONE)
            .expect("denominator overflow in `Erc4626::_convert_to_shares`");

        let shares = assets.mul_div(multiplier, denominator, rounding);

        Ok(shares)
    }

    /// Converts a given amount of shares to assets using the specified
    /// `rounding` mode.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `shares` - The amount of shares to convert.
    /// * `rounding` - The [`Rounding`] mode to use for the conversion.
    /// * `erc20` - Read access to an [`Erc20`] contract.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAsset`] - If the token address is not a valid ERC-20
    ///   token.
    ///
    /// # Panics
    ///
    /// * If decimal offset calculation overflows.
    /// * If multiplication or division operations overflow.
    fn _convert_to_assets_internal(
        &self,
        shares: U256,
        rounding: Rounding,
    ) -> Result<U256, Vec<u8>> {
        let multiplier = self
            ._total_assets()?
            .checked_add(ONE)
            .expect("multiplier overflow in `Erc4626::_convert_to_assets`");

        let total_supply = IErc20::total_supply(self);

        let denominator = total_supply
            .checked_add(
                TEN.checked_pow(U256::from(self._decimals_offset())).expect(
                    "decimal offset overflow in `Erc4626::_convert_to_assets`",
                ),
            )
            .expect("denominator overflow in `Erc4626::_convert_to_assets`");

        let assets = shares.mul_div(multiplier, denominator, rounding);

        Ok(assets)
    }

    /// Deposit/mint common workflow.
    ///
    /// # Arguments
    ///
    /// * `caller` - Address initiating the deposit.
    /// * `receiver` - Address receiving the minted shares.
    /// * `assets` - Amount of underlying tokens to transfer.
    /// * `shares` - Amount of shares to mint.
    /// * `erc20` - Write access to an [`Erc20`] contract.
    ///
    /// # Errors
    ///
    /// * [`safe_erc20::Error::SafeErc20FailedOperation`] - If token transfer
    ///   fails.
    /// * [`erc20::Error::InvalidReceiver`] - If `receiver` is
    ///   [`Address::ZERO`].
    ///
    /// # Events
    ///
    /// * [`Deposit`]
    fn _deposit_internal(
        &mut self,
        caller: Address,
        receiver: Address,
        assets: U256,
        shares: U256,
    ) -> Result<(), Vec<u8>> {
        // If asset() is ERC-777, `transfer_from` can trigger a reentrancy
        // BEFORE the transfer happens through the `tokens_to_send` hook. On the
        // other hand, the `token_received` hook, that is triggered after the
        // transfer, calls the vault, which is assumed not malicious.
        //
        // Conclusion: we need to do the transfer before we mint so that any
        // reentrancy would happen before the assets are transferred and before
        // the shares are minted, which is a valid state.

        Erc4626Internal::_asset(self).safe_transfer_from(
            caller,
            contract::address(),
            assets,
        )?;

        Erc20Internal::_mint(self, receiver, shares)?;

        evm::log(Deposit { sender: caller, owner: receiver, assets, shares });

        Ok(())
    }

    /// Withdraw/redeem common workflow.
    ///
    /// # Arguments
    ///
    /// * `caller` - Address initiating the withdrawal.
    /// * `receiver` - Address receiving the assets.
    /// * `owner` - Address owning the shares.
    /// * `assets` - Amount of underlying tokens to transfer.
    /// * `shares` - Amount of shares to burn.
    /// * `erc20` - Write access to an [`Erc20`] contract.
    ///
    /// # Errors
    ///
    /// * [`erc20::Error::InsufficientAllowance`] - If `caller` needs allowance.
    /// * [`erc20::Error::InvalidApprover`] - If `owner` is [`Address::ZERO`].
    /// * [`erc20::Error::InsufficientBalance`] - If `owner` lacks shares.
    /// * [`safe_erc20::Error::SafeErc20FailedOperation`] - If transfer fails.
    ///
    /// # Events
    ///
    /// * [`Withdraw`]
    fn _withdraw_internal(
        &mut self,
        caller: Address,
        receiver: Address,
        owner: Address,
        assets: U256,
        shares: U256,
    ) -> Result<(), Vec<u8>> {
        if caller != owner {
            self._spend_allowance(owner, caller, shares)?;
        }

        // If asset() is ERC-777, `transfer` can trigger a reentrancy AFTER the
        // transfer happens through the `tokens_received` hook. On the other
        // hand, the `tokens_to_send` hook, that is triggered before the
        // transfer, calls the vault, which is assumed not malicious.
        //
        // Conclusion: we need to do the transfer after the burn so that any
        // reentrancy would happen after the shares are burned and after the
        // assets are transferred, which is a valid state.

        self._burn(owner, shares)?;

        Erc4626Internal::_asset(self).safe_transfer(receiver, assets)?;

        evm::log(Withdraw { sender: caller, receiver, owner, assets, shares });

        Ok(())
    }

    /// Returns the decimals offset between the underlying asset and vault
    /// shares.
    /// Currently, always returns [`U8::ZERO`].
    #[must_use]
    fn _decimals_offset(&self) -> U8 {
        U8::ZERO
    }
}

/// State of an [`Erc4626`] token.
#[storage]
pub struct Erc4626 {
    /// [`Erc20`] token.
    pub erc20: Erc20,
    /// Token Address of the vault.
    pub(crate) asset: StorageAddress,
    /// Token decimals.
    pub(crate) underlying_decimals: StorageU8,
}

impl Deref for Erc4626 {
    type Target = Erc20;

    fn deref(&self) -> &Self::Target {
        &self.erc20
    }
}

impl DerefMut for Erc4626 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.erc20
    }
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc4626 {}

#[public]
#[implements(IErc4626, IErc20, IErc20Metadata)]
impl Erc4626 {
    /// Constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `asset` - The underlying vault asset.
    #[constructor]
    pub fn constructor(&mut self, asset: Address) {
        let underlying_decimals =
            self.try_get_asset_decimals(asset).unwrap_or(18);

        self.underlying_decimals.set(U8::from(underlying_decimals));
        self.asset.set(asset);
    }
}

impl Erc4626 {
    /// Attempts to fetch the asset decimals. Returns None if the attempt failed
    /// in any way. This follows Rust's idiomatic Option pattern rather than
    /// Solidity's boolean tuple return.
    fn try_get_asset_decimals(&self, asset: Address) -> Option<u8> {
        let erc20 = IErc20MetadataInterface::new(asset);
        erc20.decimals(self).ok()
    }
}

#[public]
impl IErc4626 for Erc4626 {}

#[public]
impl IErc20 for Erc4626 {}

#[public]
impl IErc20Metadata for Erc4626 {
    fn decimals(&self) -> U8 {
        Erc4626Internal::_decimals(self)
    }
}

impl Erc4626Internal for Erc4626 {}

impl Erc4626Storage for Erc4626 {
    fn asset(&self) -> &StorageAddress {
        &self.asset
    }

    fn asset_mut(&mut self) -> &mut StorageAddress {
        &mut self.asset
    }

    fn underlying_decimals(&self) -> &StorageU8 {
        &self.underlying_decimals
    }

    fn underlying_decimals_mut(&mut self) -> &mut StorageU8 {
        &mut self.underlying_decimals
    }
}

impl Erc20Internal for Erc4626 {}

impl Erc20Storage for Erc4626 {
    fn balances(&self) -> &StorageMap<Address, StorageU256> {
        Erc20Storage::balances(&self.erc20)
    }

    fn balances_mut(&mut self) -> &mut StorageMap<Address, StorageU256> {
        Erc20Storage::balances_mut(&mut self.erc20)
    }

    fn allowances(
        &self,
    ) -> &StorageMap<Address, StorageMap<Address, StorageU256>> {
        Erc20Storage::allowances(&self.erc20)
    }

    fn allowances_mut(
        &mut self,
    ) -> &mut StorageMap<Address, StorageMap<Address, StorageU256>> {
        Erc20Storage::allowances_mut(&mut self.erc20)
    }

    fn total_supply(&self) -> &StorageU256 {
        Erc20Storage::total_supply(&self.erc20)
    }

    fn total_supply_mut(&mut self) -> &mut StorageU256 {
        Erc20Storage::total_supply_mut(&mut self.erc20)
    }
}

impl Erc20MetadataStorage for Erc4626 {
    fn name(&self) -> &StorageString {
        Erc20MetadataStorage::name(&self.erc20)
    }

    fn symbol(&self) -> &StorageString {
        Erc20MetadataStorage::symbol(&self.erc20)
    }
}

impl IErc165 for Erc4626 {
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IErc4626>::interface_id() == interface_id
            || self.erc20.supports_interface(interface_id)
    }
}

// TODO: Add missing tests once `motsu` supports calling external contracts.
#[cfg(test)]
mod tests {
    use core::ops::Deref;

    use alloy_primitives::{address, aliases::B32, Address, U256, U8};
    use motsu::prelude::*;

    use super::*;

    #[motsu::test]
    fn asset_works(contract: Contract<Erc4626>, alice: Address) {
        let asset = address!("DeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF");
        contract.init(alice, |contract| contract.asset.set(asset));
        assert_eq!(contract.sender(alice)._asset(), asset);
    }

    #[motsu::test]
    fn max_deposit(contract: Contract<Erc4626>, alice: Address) {
        let max_deposit = contract.sender(alice)._max_deposit(alice);
        assert_eq!(max_deposit, U256::MAX);
    }

    #[motsu::test]
    fn max_mint(contract: Contract<Erc4626>, alice: Address) {
        let max_mint = contract.sender(alice)._max_mint(alice);
        assert_eq!(max_mint, U256::MAX);
    }

    #[motsu::test]
    fn max_redeem_works(contract: Contract<Erc4626>, alice: Address) {
        let assets = U256::from(1000);
        contract.init(alice, |contract| {
            contract
                .erc20
                ._mint(alice, assets)
                .motsu_expect("should mint assets");
        });
        let max_redeem = contract.sender(alice)._max_redeem(alice);
        assert_eq!(assets, max_redeem);
    }

    #[motsu::test]
    fn decimals_offset(contract: Contract<Erc4626>, alice: Address) {
        let decimals_offset = contract.sender(alice)._decimals_offset();
        assert_eq!(decimals_offset, U8::ZERO);
    }

    #[motsu::test]
    fn decimals(contract: Contract<Erc4626>, alice: Address) {
        let underlying_decimals = U8::from(17);

        let decimals =
            Erc4626Internal::_decimals(contract.sender(alice).deref());
        assert_eq!(decimals, U8::ZERO);

        contract.sender(alice).underlying_decimals.set(underlying_decimals);

        let decimals =
            Erc4626Internal::_decimals(contract.sender(alice).deref());
        assert_eq!(decimals, underlying_decimals);
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc4626 as IErc4626>::interface_id();
        let expected: B32 = 0x87dfe5a0_u32.into();
        assert_eq!(actual, expected);
    }
}
