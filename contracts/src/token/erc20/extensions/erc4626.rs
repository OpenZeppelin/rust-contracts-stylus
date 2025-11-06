//! ERC-4626 Tokenized Vault Standard Implementation as defined in [ERC-4626].
//!
//! This extension allows the minting and burning of "shares" (represented using
//! the ERC-20 inheritance) in exchange for underlying "assets" through
//! standardized `deposit`, `mint`, `redeem` and `burn` workflows. This contract
//! extends the ERC-20 standard. Any additional extensions included along it
//! would affect the "shares" token represented by this contract and not the
//! "assets" token which is an independent contract.

use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, Address, U256, U8};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    call::{Call, MethodError},
    contract, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageU8},
};

use super::IErc20Metadata;
use crate::{
    token::erc20::{
        self,
        abi::{Erc20Interface, Erc20MetadataInterface},
        utils::{safe_erc20, ISafeErc20, SafeErc20},
        Erc20, IErc20,
    },
    utils::math::alloy::{Math, Rounding},
};

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

#[cfg_attr(coverage_nightly, coverage(off))]
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

#[cfg_attr(coverage_nightly, coverage(off))]
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

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of an [`Erc4626`] token.
#[storage]
pub struct Erc4626 {
    /// Token Address of the vault.
    pub(crate) asset: StorageAddress,
    /// Token decimals.
    pub(crate) underlying_decimals: StorageU8,
    /// Decimals offset.
    pub(crate) decimals_offset: StorageU8,
    /// [`SafeErc20`] contract.
    safe_erc20: SafeErc20,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc4626 {}

/// ERC-4626 Tokenized Vault Standard Interface
#[interface_id]
pub trait IErc4626: IErc20Metadata {
    /// The error type associated to the trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the address of the underlying token used for the Vault for
    /// accounting, depositing, and withdrawing.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[must_use]
    fn asset(&self) -> Address;

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
    fn total_assets(&self) -> Result<U256, Self::Error>;

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
    fn convert_to_shares(&self, assets: U256) -> Result<U256, Self::Error>;

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
    fn convert_to_assets(&self, shares: U256) -> Result<U256, Self::Error>;

    /// Returns the maximum amount of the underlying asset that can be deposited
    /// into the Vault for the receiver, through a deposit call.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `receiver` - The address of the entity receiving the shares.
    #[must_use]
    fn max_deposit(&self, receiver: Address) -> U256;

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
    fn preview_deposit(&self, assets: U256) -> Result<U256, Self::Error>;

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
    ) -> Result<U256, Self::Error>;

    /// Returns the maximum amount of the Vault shares that can be minted for
    /// the receiver, through a mint call.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `receiver` - The address of the entity receiving the shares.
    #[must_use]
    fn max_mint(&self, receiver: Address) -> U256;

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
    fn preview_mint(&self, shares: U256) -> Result<U256, Self::Error>;

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
    ) -> Result<U256, Self::Error>;

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
    fn max_withdraw(&self, owner: Address) -> Result<U256, Self::Error>;

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
    fn preview_withdraw(&self, assets: U256) -> Result<U256, Self::Error>;

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
    ) -> Result<U256, Self::Error>;

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
    fn max_redeem(&self, owner: Address) -> U256;

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
    fn preview_redeem(&self, shares: U256) -> Result<U256, Self::Error>;

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
    ) -> Result<U256, Self::Error>;
}

impl Erc4626 {
    /// See [`IErc4626::asset`].
    #[must_use]
    pub fn asset(&self) -> Address {
        self.asset.get()
    }

    /// See [`IErc4626::total_assets`].
    #[allow(clippy::missing_errors_doc)]
    pub fn total_assets(&self) -> Result<U256, Error> {
        let asset = self.asset();
        let erc20 = Erc20Interface::new(asset);
        erc20
            .balance_of(self, contract::address())
            .map_err(|_| InvalidAsset { asset }.into())
    }

    /// See [`IErc4626::convert_to_shares`].
    #[allow(clippy::missing_errors_doc)]
    pub fn convert_to_shares(
        &self,
        assets: U256,
        erc20: &Erc20,
    ) -> Result<U256, Error> {
        self._convert_to_shares(assets, Rounding::Floor, erc20)
    }

    /// See [`IErc4626::convert_to_assets`].
    #[allow(clippy::missing_errors_doc)]
    pub fn convert_to_assets(
        &self,
        shares: U256,
        erc20: &Erc20,
    ) -> Result<U256, Error> {
        self._convert_to_assets(shares, Rounding::Floor, erc20)
    }

    /// See [`IErc4626::max_deposit`].
    #[must_use]
    pub fn max_deposit(&self, _receiver: Address) -> U256 {
        U256::MAX
    }

    /// See [`IErc4626::max_mint`].
    #[must_use]
    pub fn max_mint(&self, _receiver: Address) -> U256 {
        U256::MAX
    }

    /// See [`IErc4626::max_withdraw`].
    #[allow(clippy::missing_errors_doc)]
    pub fn max_withdraw(
        &self,
        owner: Address,
        erc20: &Erc20,
    ) -> Result<U256, Error> {
        let balance = erc20.balance_of(owner);
        self._convert_to_assets(balance, Rounding::Floor, erc20)
    }

    /// See [`IErc4626::max_redeem`].
    #[must_use]
    pub fn max_redeem(&self, owner: Address, erc20: &Erc20) -> U256 {
        erc20.balance_of(owner)
    }

    /// See [`IErc4626::preview_deposit`].
    #[allow(clippy::missing_errors_doc)]
    pub fn preview_deposit(
        &self,
        assets: U256,
        erc20: &Erc20,
    ) -> Result<U256, Error> {
        self._convert_to_shares(assets, Rounding::Floor, erc20)
    }

    /// See [`IErc4626::preview_mint`].
    #[allow(clippy::missing_errors_doc)]
    pub fn preview_mint(
        &self,
        shares: U256,
        erc20: &Erc20,
    ) -> Result<U256, Error> {
        self._convert_to_assets(shares, Rounding::Ceil, erc20)
    }

    /// See [`IErc4626::preview_withdraw`].
    #[allow(clippy::missing_errors_doc)]
    pub fn preview_withdraw(
        &self,
        assets: U256,
        erc20: &Erc20,
    ) -> Result<U256, Error> {
        self._convert_to_shares(assets, Rounding::Ceil, erc20)
    }

    /// See [`IErc4626::preview_redeem`].
    #[allow(clippy::missing_errors_doc)]
    pub fn preview_redeem(
        &self,
        shares: U256,
        erc20: &Erc20,
    ) -> Result<U256, Error> {
        self._convert_to_assets(shares, Rounding::Floor, erc20)
    }

    /// See [`IErc4626::deposit`].
    #[allow(clippy::missing_errors_doc)]
    pub fn deposit(
        &mut self,
        assets: U256,
        receiver: Address,
        erc20: &mut Erc20,
    ) -> Result<U256, Error> {
        let max_assets = self.max_deposit(receiver);

        if assets > max_assets {
            return Err(Error::ExceededMaxDeposit(ERC4626ExceededMaxDeposit {
                receiver,
                assets,
                max: max_assets,
            }));
        }

        let shares = self.preview_deposit(assets, erc20)?;

        self._deposit(msg::sender(), receiver, assets, shares, erc20)?;

        Ok(shares)
    }

    /// See [`IErc4626::mint`].
    #[allow(clippy::missing_errors_doc)]
    pub fn mint(
        &mut self,
        shares: U256,
        receiver: Address,
        erc20: &mut Erc20,
    ) -> Result<U256, Error> {
        let max_shares = self.max_mint(receiver);

        if shares > max_shares {
            return Err(Error::ExceededMaxMint(ERC4626ExceededMaxMint {
                receiver,
                shares,
                max: max_shares,
            }));
        }

        let assets = self.preview_mint(shares, erc20)?;
        self._deposit(msg::sender(), receiver, assets, shares, erc20)?;

        Ok(assets)
    }

    /// See [`IErc4626::withdraw`].
    #[allow(clippy::missing_errors_doc)]
    pub fn withdraw(
        &mut self,
        assets: U256,
        receiver: Address,
        owner: Address,
        erc20: &mut Erc20,
    ) -> Result<U256, Error> {
        let max_assets = self.max_withdraw(owner, erc20)?;

        if assets > max_assets {
            return Err(Error::ExceededMaxWithdraw(
                ERC4626ExceededMaxWithdraw { owner, assets, max: max_assets },
            ));
        }

        let shares = self.preview_withdraw(assets, erc20)?;
        self._withdraw(msg::sender(), receiver, owner, assets, shares, erc20)?;

        Ok(shares)
    }

    /// See [`IErc4626::redeem`].
    #[allow(clippy::missing_errors_doc)]
    pub fn redeem(
        &mut self,
        shares: U256,
        receiver: Address,
        owner: Address,
        erc20: &mut Erc20,
    ) -> Result<U256, Error> {
        let max_shares = self.max_redeem(owner, erc20);
        if shares > max_shares {
            return Err(Error::ExceededMaxRedeem(ERC4626ExceededMaxRedeem {
                owner,
                shares,
                max: max_shares,
            }));
        }

        let assets = self.preview_redeem(shares, erc20)?;

        self._withdraw(msg::sender(), receiver, owner, assets, shares, erc20)?;

        Ok(assets)
    }
}

#[public]
impl Erc4626 {
    // TODO: remove `decimals_offset` once function overriding is possible.
    /// Constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `asset` - The underlying vault asset.
    /// * `decimals_offset` - The decimal offset of the vault shares.
    #[constructor]
    pub fn constructor(&mut self, asset: Address, decimals_offset: U8) {
        let underlying_decimals =
            self.try_get_asset_decimals(asset).unwrap_or(18);

        self.underlying_decimals.set(U8::from(underlying_decimals));
        self.asset.set(asset);
        self.decimals_offset.set(decimals_offset);
    }
}

impl Erc4626 {
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
    ///         self.erc4626.decimals()
    ///     }
    /// ```
    #[must_use]
    pub fn decimals(&self) -> U8 {
        self.underlying_decimals
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
    pub fn _convert_to_shares(
        &self,
        assets: U256,
        rounding: Rounding,
        erc20: &Erc20,
    ) -> Result<U256, Error> {
        let total_supply = erc20.total_supply();

        let multiplier = total_supply
            .checked_add(
                TEN.checked_pow(U256::from(self._decimals_offset())).expect(
                    "decimal offset overflow in `Erc4626::_convert_to_shares`",
                ),
            )
            .expect("multiplier overflow in `Erc4626::_convert_to_shares`");

        let denominator = self
            .total_assets()?
            .checked_add(U256::ONE)
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
    pub fn _convert_to_assets(
        &self,
        shares: U256,
        rounding: Rounding,
        erc20: &Erc20,
    ) -> Result<U256, Error> {
        let multiplier = self
            .total_assets()?
            .checked_add(U256::ONE)
            .expect("multiplier overflow in `Erc4626::_convert_to_assets`");

        let total_supply = erc20.total_supply();

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
    pub fn _deposit(
        &mut self,
        caller: Address,
        receiver: Address,
        assets: U256,
        shares: U256,
        erc20: &mut Erc20,
    ) -> Result<(), Error> {
        // If asset() is ERC-777, `transfer_from` can trigger a reentrancy
        // BEFORE the transfer happens through the `tokens_to_send` hook. On the
        // other hand, the `token_received` hook, that is triggered after the
        // transfer, calls the vault, which is assumed not malicious.
        //
        // Conclusion: we need to do the transfer before we mint so that any
        // reentrancy would happen before the assets are transferred and before
        // the shares are minted, which is a valid state.

        self.safe_erc20.safe_transfer_from(
            self.asset(),
            caller,
            contract::address(),
            assets,
        )?;

        erc20._mint(receiver, shares)?;

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
    pub fn _withdraw(
        &mut self,
        caller: Address,
        receiver: Address,
        owner: Address,
        assets: U256,
        shares: U256,
        erc20: &mut Erc20,
    ) -> Result<(), Error> {
        if caller != owner {
            erc20._spend_allowance(owner, caller, shares)?;
        }

        // If asset() is ERC-777, `transfer` can trigger a reentrancy AFTER the
        // transfer happens through the `tokens_received` hook. On the other
        // hand, the `tokens_to_send` hook, that is triggered before the
        // transfer, calls the vault, which is assumed not malicious.
        //
        // Conclusion: we need to do the transfer after the burn so that any
        // reentrancy would happen after the shares are burned and after the
        // assets are transferred, which is a valid state.

        erc20._burn(owner, shares)?;

        self.safe_erc20.safe_transfer(self.asset(), receiver, assets)?;

        evm::log(Withdraw { sender: caller, receiver, owner, assets, shares });

        Ok(())
    }

    /// Returns the decimals offset between the underlying asset and vault
    /// shares.
    /// Currently, always returns [`U8::ZERO`].
    #[must_use]
    pub fn _decimals_offset(&self) -> U8 {
        self.decimals_offset.get()
    }
}

impl Erc4626 {
    /// Attempts to fetch the asset decimals. Returns None if the attempt failed
    /// in any way. This follows Rust's idiomatic Option pattern rather than
    /// Solidity's boolean tuple return.
    fn try_get_asset_decimals(&mut self, asset: Address) -> Option<u8> {
        let erc20 = Erc20MetadataInterface::new(asset);
        let call = Call::new_in(self);
        erc20.decimals(call).ok()
    }
}

// TODO: implement `IErc165` once `IErc4626` is implemented for `Erc4626`.
// #[public]
// impl IErc165 for Erc4626 {
//     fn supports_interface(&self, interface_id: B32) -> bool {
//         <Self as IErc4626>::interface_id() == interface_id
//             || <Self as IErc165>::interface_id() == interface_id
//     }
// }

#[cfg(test)]
mod tests {
    #![allow(clippy::unused_self)]
    use alloy_primitives::{address, aliases::B32, Address, U256, U8};
    use motsu::prelude::*;
    use stylus_sdk::{prelude::*, storage::StorageU8};

    use super::*;
    use crate::{
        token::erc20::{
            extensions::{Erc20Metadata, IErc20Metadata},
            ERC20InsufficientAllowance, ERC20InvalidReceiver, Erc20,
        },
        utils::introspection::erc165::IErc165,
    };

    #[storage]
    struct Erc4626TestExample {
        erc4626: Erc4626,
        erc20: Erc20,
        metadata: Erc20Metadata,
    }

    unsafe impl TopLevelStorage for Erc4626TestExample {}

    #[public]
    #[implements(IErc4626<Error = Error>, IErc20Metadata, IErc165)]
    impl Erc4626TestExample {
        #[constructor]
        fn constructor(&mut self, asset: Address, decimals_offset: U8) {
            self.erc4626.constructor(asset, decimals_offset);
        }
    }

    #[public]
    impl IErc4626 for Erc4626TestExample {
        type Error = Error;

        fn asset(&self) -> Address {
            self.erc4626.asset()
        }

        fn total_assets(&self) -> Result<U256, Self::Error> {
            self.erc4626.total_assets()
        }

        fn convert_to_shares(&self, assets: U256) -> Result<U256, Self::Error> {
            self.erc4626.convert_to_shares(assets, &self.erc20)
        }

        fn convert_to_assets(&self, shares: U256) -> Result<U256, Self::Error> {
            self.erc4626.convert_to_assets(shares, &self.erc20)
        }

        fn max_deposit(&self, receiver: Address) -> U256 {
            self.erc4626.max_deposit(receiver)
        }

        fn preview_deposit(&self, assets: U256) -> Result<U256, Self::Error> {
            self.erc4626.preview_deposit(assets, &self.erc20)
        }

        fn deposit(
            &mut self,
            assets: U256,
            receiver: Address,
        ) -> Result<U256, Self::Error> {
            self.erc4626.deposit(assets, receiver, &mut self.erc20)
        }

        fn max_mint(&self, receiver: Address) -> U256 {
            self.erc4626.max_mint(receiver)
        }

        fn preview_mint(&self, shares: U256) -> Result<U256, Self::Error> {
            self.erc4626.preview_mint(shares, &self.erc20)
        }

        fn mint(
            &mut self,
            shares: U256,
            receiver: Address,
        ) -> Result<U256, Self::Error> {
            self.erc4626.mint(shares, receiver, &mut self.erc20)
        }

        fn max_withdraw(&self, owner: Address) -> Result<U256, Self::Error> {
            self.erc4626.max_withdraw(owner, &self.erc20)
        }

        fn preview_withdraw(&self, assets: U256) -> Result<U256, Self::Error> {
            self.erc4626.preview_withdraw(assets, &self.erc20)
        }

        fn withdraw(
            &mut self,
            assets: U256,
            receiver: Address,
            owner: Address,
        ) -> Result<U256, Self::Error> {
            self.erc4626.withdraw(assets, receiver, owner, &mut self.erc20)
        }

        fn max_redeem(&self, owner: Address) -> U256 {
            self.erc4626.max_redeem(owner, &self.erc20)
        }

        fn preview_redeem(&self, shares: U256) -> Result<U256, Self::Error> {
            self.erc4626.preview_redeem(shares, &self.erc20)
        }

        fn redeem(
            &mut self,
            shares: U256,
            receiver: Address,
            owner: Address,
        ) -> Result<U256, Self::Error> {
            self.erc4626.redeem(shares, receiver, owner, &mut self.erc20)
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[public]
    impl IErc20Metadata for Erc4626TestExample {
        fn name(&self) -> String {
            self.metadata.name()
        }

        fn symbol(&self) -> String {
            self.metadata.symbol()
        }

        fn decimals(&self) -> U8 {
            self.erc4626.decimals()
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[public]
    impl IErc165 for Erc4626TestExample {
        fn supports_interface(&self, interface_id: B32) -> bool {
            <Self as IErc4626>::interface_id() == interface_id
                || self.erc20.supports_interface(interface_id)
                || self.metadata.supports_interface(interface_id)
        }
    }

    #[storage]
    struct Erc20DecimalsMock {
        decimals: StorageU8,
    }

    unsafe impl TopLevelStorage for Erc20DecimalsMock {}

    #[public]
    impl Erc20DecimalsMock {
        #[constructor]
        fn constructor(&mut self, decimals: U8) {
            self.decimals.set(decimals);
        }

        fn decimals(&self) -> U8 {
            self.decimals.get()
        }
    }

    #[motsu::test]
    fn decimals_inherited_from_asset(alice: Address) {
        for decimals in [0, 9, 12, 18, 36].map(U8::from) {
            let token = Contract::<Erc20DecimalsMock>::from_tag("erc20");
            token.sender(alice).constructor(decimals);

            let vault = Contract::<Erc4626TestExample>::from_tag("erc4626");
            vault.sender(alice).constructor(token.address(), U8::ZERO);

            assert_eq!(vault.sender(alice).decimals(), decimals);
        }
    }

    #[storage]
    struct InvalidAssetMock;

    unsafe impl TopLevelStorage for InvalidAssetMock {}

    #[public]
    impl InvalidAssetMock {}

    #[motsu::test]
    fn decimals_returns_default_value_when_asset_has_not_yet_been_created(
        vault: Contract<Erc4626TestExample>,
        invalid_asset: Contract<InvalidAssetMock>,
        alice: Address,
    ) {
        vault.sender(alice).constructor(invalid_asset.address(), U8::ZERO);
        assert_eq!(vault.sender(alice).decimals(), uint!(18_U8));
    }

    #[storage]
    struct Erc20ExcessDecimalsMock;

    unsafe impl TopLevelStorage for Erc20ExcessDecimalsMock {}

    #[public]
    impl Erc20ExcessDecimalsMock {
        fn decimals(&self) -> U256 {
            U256::MAX
        }
    }

    #[motsu::test]
    fn constructor(
        vault: Contract<Erc4626TestExample>,
        token: Contract<Erc20AssetSimpleMock>,
        alice: Address,
    ) {
        vault.sender(alice).constructor(token.address(), U8::ZERO);
        assert_eq!(vault.sender(alice).decimals(), uint!(18_U8));
        assert_eq!(vault.sender(alice).asset(), token.address());

        vault.sender(alice).erc4626.decimals_offset.set(uint!(12_U8));
        assert_eq!(vault.sender(alice).decimals(), uint!(30_U8));
    }

    #[motsu::test]
    fn decimals_returns_default_value_when_underlying_decimals_exceeds_u8_max(
        vault: Contract<Erc4626TestExample>,
        token: Contract<Erc20ExcessDecimalsMock>,
        alice: Address,
    ) {
        vault.sender(alice).constructor(token.address(), U8::ZERO);
        assert_eq!(vault.sender(alice).decimals(), uint!(18_U8));
    }

    #[motsu::test]
    #[should_panic = "Decimals should not be greater than `U8::MAX`"]
    fn decimals_revert_when_overflowing(
        vault: Contract<Erc4626TestExample>,
        token: Contract<Erc20DecimalsMock>,
        alice: Address,
    ) {
        token.sender(alice).constructor(uint!(18_U8));
        vault.sender(alice).constructor(token.address(), U8::MAX);
        _ = vault.sender(alice).decimals();
    }

    #[motsu::test]
    fn asset_works(vault: Contract<Erc4626TestExample>, alice: Address) {
        let asset = address!("DeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF");
        vault.sender(alice).erc4626.asset.set(asset);
        assert_eq!(vault.sender(alice).erc4626.asset(), asset);
    }

    #[storage]
    struct Erc20BalanceOfRevertMock;

    unsafe impl TopLevelStorage for Erc20BalanceOfRevertMock {}

    #[public]
    impl Erc20BalanceOfRevertMock {
        fn balance_of(&self, _owner: Address) -> Result<U256, Vec<u8>> {
            Err("balance_of reverted".into())
        }
    }

    #[motsu::test]
    fn total_assets_returns_invalid_asset_err(
        vault: Contract<Erc4626TestExample>,
        token: Contract<Erc20BalanceOfRevertMock>,
        alice: Address,
    ) {
        vault.sender(alice).constructor(token.address(), U8::ZERO);
        let res = vault.sender(alice).total_assets();
        assert!(
            matches!(res, Err(Error::InvalidAsset(InvalidAsset { asset })) if asset == token.address()),
            "expected InvalidAsset error"
        );
    }

    #[motsu::test]
    fn convert_and_preview_functions_revert_with_invalid_asset(
        vault: Contract<Erc4626TestExample>,
        token: Contract<Erc20BalanceOfRevertMock>,
        alice: Address,
    ) {
        vault.sender(alice).constructor(token.address(), U8::ZERO);

        // All functions that depend on total_assets should error with
        // InvalidAsset.
        let assets = uint!(1000_U256);
        let shares = uint!(500_U256);

        let r1 = vault.sender(alice).convert_to_shares(assets);
        assert!(
            matches!(r1, Err(Error::InvalidAsset(InvalidAsset { asset })) if asset == token.address()),
            "convert_to_shares should err"
        );

        let r2 = vault.sender(alice).convert_to_assets(shares);
        assert!(
            matches!(r2, Err(Error::InvalidAsset(InvalidAsset { asset })) if asset == token.address()),
            "convert_to_assets should err"
        );

        let r3 = vault.sender(alice).preview_deposit(assets);
        assert!(
            matches!(r3, Err(Error::InvalidAsset(InvalidAsset { asset })) if asset == token.address()),
            "preview_deposit should err"
        );

        let r4 = vault.sender(alice).preview_mint(shares);
        assert!(
            matches!(r4, Err(Error::InvalidAsset(InvalidAsset { asset })) if asset == token.address()),
            "preview_mint should err"
        );

        let r5 = vault.sender(alice).preview_withdraw(assets);
        assert!(
            matches!(r5, Err(Error::InvalidAsset(InvalidAsset { asset })) if asset == token.address()),
            "preview_withdraw should err"
        );

        let r6 = vault.sender(alice).preview_redeem(shares);
        assert!(
            matches!(r6, Err(Error::InvalidAsset(InvalidAsset { asset })) if asset == token.address()),
            "preview_redeem should err"
        );
    }

    #[motsu::test]
    fn redeem_exceeds_max_redeem(
        vault: Contract<Erc4626TestExample>,
        invalid_asset: Contract<InvalidAssetMock>,
        alice: Address,
        bob: Address,
    ) {
        // Use non-contract asset to avoid external interactions; redeem's
        // ExceededMaxRedeem check happens before any asset calls.
        vault.sender(alice).constructor(invalid_asset.address(), U8::ZERO);

        // Mint some shares to Alice using the internal ERC-20 share token.
        let initial_shares = uint!(100_U256);
        vault
            .sender(alice)
            .erc20
            ._mint(alice, initial_shares)
            .motsu_expect("mint shares to alice");

        // Try to redeem more than balance to trigger ExceededMaxRedeem.
        let attempt =
            vault.sender(alice).redeem(initial_shares + U256::ONE, bob, alice);
        matches!(attempt, Err(Error::ExceededMaxRedeem(ERC4626ExceededMaxRedeem { owner, shares, max })) if owner == alice && shares == initial_shares + U256::ONE && max == initial_shares)
            .then_some(())
            .expect("expected ExceededMaxRedeem error");
    }

    // Minimal ERC-20-like mock to satisfy [`SafeErc20`] and
    // [`Erc20Interface::balance_of`].
    #[storage]
    struct Erc20AssetSimpleMock {
        erc20: Erc20,
        metadata: Erc20Metadata,
    }

    unsafe impl TopLevelStorage for Erc20AssetSimpleMock {}

    #[public]
    #[implements(IErc20<Error = Error>, IErc20Metadata, IErc165)]
    impl Erc20AssetSimpleMock {}

    #[public]
    impl IErc20 for Erc20AssetSimpleMock {
        type Error = Error;

        fn total_supply(&self) -> U256 {
            self.erc20.total_supply()
        }

        fn balance_of(&self, owner: Address) -> U256 {
            self.erc20.balance_of(owner)
        }

        fn transfer(
            &mut self,
            to: Address,
            amount: U256,
        ) -> Result<bool, Error> {
            Ok(self.erc20.transfer(to, amount)?)
        }

        fn allowance(&self, owner: Address, spender: Address) -> U256 {
            self.erc20.allowance(owner, spender)
        }

        fn approve(
            &mut self,
            spender: Address,
            amount: U256,
        ) -> Result<bool, Error> {
            Ok(self.erc20.approve(spender, amount)?)
        }

        fn transfer_from(
            &mut self,
            from: Address,
            to: Address,
            amount: U256,
        ) -> Result<bool, Error> {
            Ok(self.erc20.transfer_from(from, to, amount)?)
        }
    }

    #[public]
    impl IErc20Metadata for Erc20AssetSimpleMock {
        fn name(&self) -> String {
            self.metadata.name()
        }

        fn symbol(&self) -> String {
            self.metadata.symbol()
        }

        fn decimals(&self) -> U8 {
            self.metadata.decimals()
        }
    }

    #[public]
    impl IErc165 for Erc20AssetSimpleMock {
        fn supports_interface(&self, interface_id: B32) -> bool {
            self.erc20.supports_interface(interface_id)
                || self.metadata.supports_interface(interface_id)
        }
    }

    #[motsu::test]
    fn convert_math_and_previews_rounding(
        vault: Contract<Erc4626TestExample>,
        asset: Contract<Erc20AssetSimpleMock>,
        alice: Address,
    ) {
        // Configure asset with total_assets = 1000 and 18 decimals, offset 0
        vault.sender(alice).constructor(asset.address(), U8::ZERO);

        let total_assets = uint!(1000_U256);
        asset
            .sender(alice)
            .erc20
            ._mint(vault.address(), total_assets)
            .motsu_expect("mint assets");

        // Set total_supply by minting shares to Alice
        let supply = uint!(100_U256);
        vault
            .sender(alice)
            .erc20
            ._mint(alice, supply)
            .motsu_expect("mint shares");

        // convert_to_shares(assets=1000) with rounding floor should be:
        // shares = 1000 * (supply + 10^offset) / (assets_under_mgmt + 1)
        // = 1000 * (100 + 1) / (1000 + 1) = floor(101000/1001) = 100
        let shares = vault
            .sender(alice)
            .convert_to_shares(total_assets)
            .motsu_expect("convert_to_shares");
        assert_eq!(shares, uint!(100_U256));

        // convert_to_assets(shares=100), floor: 100 * (1000+1) / (100+1) =
        // floor(100100/101) = 991
        let assets = vault
            .sender(alice)
            .convert_to_assets(uint!(100_U256))
            .motsu_expect("convert_to_assets");
        assert_eq!(assets, uint!(991_U256));

        // preview_mint uses Ceil rounding for shares->assets:
        // ceil(100100/101)     // = 992
        let assets_ceiled = vault
            .sender(alice)
            .preview_mint(uint!(100_U256))
            .motsu_expect("preview_mint");
        assert_eq!(assets_ceiled, uint!(992_U256));

        // preview_withdraw uses Ceil rounding for assets->shares:
        // ceil(1000*101/1001) = 101
        let shares_ceiled = vault
            .sender(alice)
            .preview_withdraw(total_assets)
            .motsu_expect("preview_withdraw");
        assert_eq!(shares_ceiled, uint!(101_U256));
    }

    #[motsu::test]
    fn deposit_success_and_receiver_gets_shares(
        vault: Contract<Erc4626TestExample>,
        asset: Contract<Erc20AssetSimpleMock>,
        alice: Address,
        bob: Address,
    ) {
        // Asset supports transferFrom; set managed assets to 1000 so math
        // works.
        vault.sender(alice).constructor(asset.address(), U8::ZERO);

        let assets = uint!(1000_U256);
        asset
            .sender(alice)
            .erc20
            ._mint(alice, assets)
            .motsu_expect("mint assets");
        asset
            .sender(alice)
            .erc20
            .approve(vault.address(), assets)
            .motsu_expect("approve assets");

        let expected_shares = vault
            .sender(alice)
            .preview_deposit(assets)
            .motsu_expect("preview_deposit");
        let minted =
            vault.sender(alice).deposit(assets, bob).motsu_expect("deposit");
        assert_eq!(minted, expected_shares);
        assert_eq!(vault.sender(alice).erc20.balance_of(bob), expected_shares);
    }

    #[motsu::test]
    fn deposit_with_zero_receiver_reverts(
        vault: Contract<Erc4626TestExample>,
        asset: Contract<Erc20AssetSimpleMock>,
        alice: Address,
    ) {
        vault.sender(alice).constructor(asset.address(), U8::ZERO);

        let assets = uint!(1000_U256);
        asset
            .sender(alice)
            .erc20
            ._mint(alice, assets)
            .motsu_expect("mint assets");
        asset
            .sender(alice)
            .erc20
            .approve(vault.address(), assets)
            .motsu_expect("approve assets");

        let attempt = vault.sender(alice).deposit(U256::ONE, Address::ZERO);
        assert!(
            matches!(attempt, Err(Error::InvalidReceiver(ERC20InvalidReceiver { receiver })) if receiver.is_zero()),
            "expected InvalidReceiver error"
        );
    }

    #[motsu::test]
    fn withdraw(
        vault: Contract<Erc4626TestExample>,
        asset: Contract<Erc20AssetSimpleMock>,
        alice: Address,
        bob: Address,
    ) {
        vault.sender(alice).constructor(asset.address(), U8::ZERO);

        let assets = uint!(1000_U256);
        asset
            .sender(alice)
            .erc20
            ._mint(vault.address(), assets)
            .motsu_expect("mint assets");

        // Give Alice some shares to withdraw against
        vault
            .sender(alice)
            .erc20
            ._mint(alice, uint!(200_U256))
            .motsu_expect("mint shares");

        let shares = vault
            .sender(alice)
            .withdraw(uint!(10_U256), bob, alice)
            .motsu_expect("withdraw");

        assert_eq!(shares, uint!(3_U256));
    }

    #[motsu::test]
    fn withdraw_requires_allowance_when_caller_not_owner(
        vault: Contract<Erc4626TestExample>,
        asset: Contract<Erc20AssetSimpleMock>,
        alice: Address,
        bob: Address,
    ) {
        vault.sender(alice).constructor(asset.address(), U8::ZERO);

        let assets = uint!(1000_U256);
        asset
            .sender(alice)
            .erc20
            ._mint(vault.address(), assets)
            .motsu_expect("mint assets");

        // Give Alice some shares to withdraw against
        vault
            .sender(alice)
            .erc20
            ._mint(alice, uint!(200_U256))
            .motsu_expect("mint shares");

        // Bob tries to withdraw on behalf of Alice without allowance
        let attempt = vault.sender(bob).withdraw(U256::ONE, bob, alice);
        assert!(
            matches!(attempt, Err(Error::InsufficientAllowance(ERC20InsufficientAllowance { spender, allowance, needed })) if spender == bob && allowance.is_zero() && needed == U256::ONE),
            "expected InsufficientAllowance error"
        );
    }

    #[motsu::test]
    fn withdraw_exceeds_max_withdraw(
        vault: Contract<Erc4626TestExample>,
        asset: Contract<Erc20AssetSimpleMock>,
        alice: Address,
        bob: Address,
    ) {
        vault.sender(alice).constructor(asset.address(), U8::ZERO);

        // Alice has zero shares; any positive withdrawal should exceed max.
        let attempt = vault.sender(alice).withdraw(U256::ONE, bob, alice);
        assert!(
            matches!(attempt, Err(Error::ExceededMaxWithdraw(ERC4626ExceededMaxWithdraw { owner, assets, max })) if owner == alice && assets == U256::ONE && max.is_zero()),
            "expected ExceededMaxWithdraw error"
        );
    }

    #[motsu::test]
    fn deposit_fails_when_safe_transfer_from_fails(
        vault: Contract<Erc4626TestExample>,
        asset: Contract<Erc20AssetSimpleMock>,
        alice: Address,
    ) {
        vault.sender(alice).constructor(asset.address(), U8::ZERO);

        let assets = uint!(1000_U256);
        asset
            .sender(alice)
            .erc20
            ._mint(alice, assets)
            .motsu_expect("mint assets");

        // missing approval to vault to take out assets
        let attempt = vault.sender(alice).deposit(U256::ONE, alice);
        assert!(
            matches!(attempt, Err(Error::SafeErc20FailedOperation(safe_erc20::SafeErc20FailedOperation { token })) if token == asset.address()),
            "expected SafeErc20FailedOperation error"
        );
    }

    #[motsu::test]
    fn mint_success_and_receiver_gets_shares(
        vault: Contract<Erc4626TestExample>,
        asset: Contract<Erc20AssetSimpleMock>,
        alice: Address,
        bob: Address,
    ) {
        vault.sender(alice).constructor(asset.address(), U8::ZERO);

        let assets = uint!(1000_U256);
        asset
            .sender(alice)
            .erc20
            ._mint(alice, assets)
            .motsu_expect("mint assets");
        asset
            .sender(alice)
            .erc20
            .approve(vault.address(), assets)
            .motsu_expect("approve assets");

        let shares = uint!(25_U256);
        let expected_assets = vault
            .sender(alice)
            .preview_mint(shares)
            .motsu_expect("preview_mint");
        let spent_assets =
            vault.sender(alice).mint(shares, bob).motsu_expect("mint");

        assert_eq!(spent_assets, expected_assets);
        assert_eq!(vault.sender(alice).erc20.balance_of(bob), shares);
    }

    #[motsu::test]
    fn mint_fails_when_safe_transfer_from_fails(
        vault: Contract<Erc4626TestExample>,
        asset: Contract<Erc20AssetSimpleMock>,
        alice: Address,
    ) {
        vault.sender(alice).constructor(asset.address(), U8::ZERO);

        let assets = uint!(1000_U256);
        asset
            .sender(alice)
            .erc20
            ._mint(alice, assets)
            .motsu_expect("mint assets");

        // missing approval to vault to take out assets
        let attempt = vault.sender(alice).mint(uint!(5_U256), alice);
        assert!(
            matches!(attempt, Err(Error::SafeErc20FailedOperation(safe_erc20::SafeErc20FailedOperation { token })) if token == asset.address()),
            "expected SafeErc20FailedOperation error"
        );
    }

    #[motsu::test]
    fn redeem_success_flow(
        vault: Contract<Erc4626TestExample>,
        asset: Contract<Erc20AssetSimpleMock>,
        alice: Address,
        bob: Address,
    ) {
        vault.sender(alice).constructor(asset.address(), U8::ZERO);

        let assets = uint!(1000_U256);
        asset
            .sender(alice)
            .erc20
            ._mint(vault.address(), assets)
            .motsu_expect("mint assets");

        // Give Alice some shares to withdraw against
        vault
            .sender(alice)
            .erc20
            ._mint(alice, uint!(200_U256))
            .motsu_expect("mint shares");

        let redeem_shares = uint!(10_U256);
        let expected_assets = vault
            .sender(alice)
            .preview_redeem(redeem_shares)
            .motsu_expect("preview_redeem");
        let assets_out = vault
            .sender(alice)
            .redeem(redeem_shares, bob, alice)
            .motsu_expect("redeem");
        assert_eq!(assets_out, expected_assets);
        // Shares decreased for Alice
        assert_eq!(
            vault.sender(alice).erc20.balance_of(alice),
            uint!(190_U256)
        );
    }

    #[motsu::test]
    fn max_deposit(vault: Contract<Erc4626TestExample>, alice: Address) {
        let max_deposit = vault.sender(alice).max_deposit(alice);
        assert_eq!(max_deposit, U256::MAX);
    }

    #[motsu::test]
    fn max_withdraw(
        vault: Contract<Erc4626TestExample>,
        token: Contract<Erc20AssetSimpleMock>,
        alice: Address,
    ) {
        vault.sender(alice).constructor(token.address(), U8::ZERO);
        let max_withdraw = vault
            .sender(alice)
            .max_withdraw(alice)
            .motsu_expect("should get max withdraw");
        assert_eq!(max_withdraw, U256::ZERO);
    }

    #[motsu::test]
    fn max_mint(vault: Contract<Erc4626TestExample>, alice: Address) {
        let max_mint = vault.sender(alice).max_mint(alice);
        assert_eq!(max_mint, U256::MAX);
    }

    #[motsu::test]
    fn max_redeem_works(vault: Contract<Erc4626TestExample>, alice: Address) {
        let assets = uint!(1000_U256);
        vault
            .sender(alice)
            .erc20
            ._mint(alice, assets)
            .motsu_expect("should mint assets");
        let max_redeem = vault.sender(alice).max_redeem(alice);
        assert_eq!(assets, max_redeem);
    }

    #[motsu::test]
    fn decimals_offset(vault: Contract<Erc4626TestExample>, alice: Address) {
        let decimals_offset = vault.sender(alice).erc4626._decimals_offset();
        assert_eq!(decimals_offset, U8::ZERO);

        let new_decimal_offset = uint!(10_U8);
        vault.sender(alice).erc4626.decimals_offset.set(new_decimal_offset);

        let decimals_offset = vault.sender(alice).erc4626._decimals_offset();
        assert_eq!(decimals_offset, new_decimal_offset);
    }

    #[motsu::test]
    fn decimals(vault: Contract<Erc4626TestExample>, alice: Address) {
        let underlying_decimals = uint!(17_U8);
        vault
            .sender(alice)
            .erc4626
            .underlying_decimals
            .set(underlying_decimals);
        let decimals = vault.sender(alice).erc4626.decimals();
        assert_eq!(decimals, underlying_decimals);

        let new_decimal_offset = uint!(10_U8);
        vault.sender(alice).erc4626.decimals_offset.set(new_decimal_offset);

        let decimals = vault.sender(alice).erc4626.decimals();
        assert_eq!(decimals, underlying_decimals + new_decimal_offset);
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc4626TestExample as IErc4626>::interface_id();
        let expected: B32 = 0x87dfe5a0_u32.into();
        assert_eq!(actual, expected);
    }
}
