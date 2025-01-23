//! ERC-4626 Tokenized Vault Standard Implementation as defined in [ERC-4626].
//!
//! This extension allows the minting and burning of "shares" (represented using
//! the ERC-20 inheritance) in exchange for underlying "assets" through
//! standardized `deposit`, `mint`, `redeem` and `burn` workflows. This contract
//! extends the ERC-20 standard. Any additional extensions included along it
//! would affect the "shares" token represented by this contract and not the
//! "assets" token which is an independent contract.

use alloy_primitives::{Address, U256};
use alloy_sol_macro::sol;
use stylus_sdk::{
    call::Call,
    contract, evm, msg,
    prelude::{public, storage},
    storage::{StorageAddress, StorageU8, TopLevelStorage},
    stylus_proc::SolidityError,
};

use crate::{
    token::erc20::{
        self,
        utils::{safe_erc20, IErc20 as IErc20Solidity, ISafeErc20, SafeErc20},
        Erc20, IErc20,
    },
    utils::math::alloy::{Math, Rounding},
};

sol! {
    /// Emitted when assets are deposited into the contract.
    ///
    /// * `sender` - Address of the entity initiating the deposit.
    /// * `owner` - Address of the recipient who owns the shares.
    /// * `assets` - Amount of assets deposited.
    /// * `shares` - Number of shares issued to the owner.
    #[allow(missing_docs)]
    event Deposit(address indexed sender, address indexed owner, uint256 assets, uint256 shares);


    /// Emitted when assets are withdrawn from the contract.
    ///
    /// * `sender` - Address of the entity initiating the withdrawal.
    /// * `receiver` - Address of the recipient receiving the assets.
    /// * `owner` - Address of the entity owning the shares.
    /// * `assets` - Amount of assets withdrawn.
    /// * `shares` - Number of shares burned.
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
    /// Indicates an error where depostit operation  failed because
    /// deposited  more assets than the max amount for `receiver
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC4626ExceededMaxDeposit(address receiver, uint256 assets, uint256 max);

    /// Indicates an error where a mint operation failed because the supplied
    /// `shares` exceeded the maximum allowed for the `receiver`.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC4626ExceededMaxMint(address receiver, uint256 shares, uint256 max);

    /// Indicates an error where a withdrawal operation failed because the
    /// supplied `assets` exceeded the maximum allowed for the `owner`.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC4626ExceededMaxWithdraw(address owner, uint256 assets, uint256 max);

    /// Indicates an error where a redemption operation failed because the
    /// supplied `shares` exceeded the maximum allowed for the `owner`.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC4626ExceededMaxRedeem(address owner, uint256 shares, uint256 max);

    /// The ERC-20 Token address is not valid (eg. `Address::ZERO`).
    ///
    /// * `token` - Address of the ERC-20 token.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error InvalidToken(address token);
}

/// An [`Erc4626`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error where a deposit operation failed because the
    /// supplied `assets` exceeded the maximum allowed for the `receiver`.
    ExceededMaxDeposit(ERC4626ExceededMaxDeposit),
    /// Indicates an error where a mint operation failed because the supplied
    /// `shares` exceeded the maximum allowed for the `receiver`.
    ExceededMaxMint(ERC4626ExceededMaxMint),
    /// Indicates an error where a withdrawal operation failed because the
    /// supplied `assets` exceeded the maximum allowed for the `owner`.
    ExceededMaxWithdraw(ERC4626ExceededMaxWithdraw),
    /// Indicates an error where a redemption operation failed because the
    /// supplied `shares` exceeded the maximum allowed for the `owner`.
    ExceededMaxRedeem(ERC4626ExceededMaxRedeem),
    /// The ERC-20 Token address is not valid. (eg. `Address::ZERO`).
    InvalidToken(InvalidToken),
    /// Error type from [`SafeErc20`] contract [`safe_erc20::Error`].
    SafeErc20(safe_erc20::Error),
    /// Error type from [`Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error),
}

/// State of an [`Erc4626`] token.
#[storage]
pub struct Erc4626 {
    /// Token Address of the vault.
    pub(crate) asset: StorageAddress,

    /// [`SafeErc20`] contract.
    pub(crate) safe_erc20: SafeErc20,

    /// [`Erc20`] contract.
    pub(crate) erc20: Erc20,

    /// Token decimals.
    pub(crate) _underlying_decimals: StorageU8,
}

/// ERC-4626 Tokenized Vault Standard Interface
pub trait IErc4626 {
    /// The error type associated to the trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the address of the underlying token used for the Vault for
    /// accounting, depositing, and withdrawing.
    ///
    /// # Requirements
    ///
    /// * MUST be an ERC-20 token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn asset(&self) -> Address;

    /// Returns the total amount of the underlying asset that is “managed” by
    /// Vault.
    ///
    /// # Requirements
    ///
    /// * SHOULD include any compounding that occurs from yield.
    /// * MUST be inclusive of any fees that are charged against assets in the
    ///   Vault.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidToken`] - If the [`IErc4626::asset()`] is not an
    ///   ERC-20 Token address.
    fn total_assets(&mut self) -> Result<U256, Self::Error>;

    /// Returns the amount of shares that the Vault would exchange for the
    /// amount of assets provided, in an ideal scenario where all the conditions
    /// are met.
    ///
    /// NOTE: This calculation MAY NOT reflect the “per-user” price-per-share,
    /// and instead should reflect the “average-user’s” price-per-share, meaning
    /// what the average user should expect to see when exchanging to and from.
    ///
    /// # Requirements
    ///
    /// * MUST NOT be inclusive of any fees that are charged against assets in
    ///   the Vault.
    /// * MUST NOT show any variations depending on the caller.
    /// * MUST NOT reflect slippage or other on-chain conditions, when
    ///   performing the actual exchange.
    ///
    ///  # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `assets` - Amount of the underlying asset.
    ///
    /// # Errors
    ///
    /// *
    fn convert_to_shares(&mut self, assets: U256) -> Result<U256, Self::Error>;

    /// Returns the amount of assets that the Vault would exchange for the
    /// amount of shares provided, in an ideal scenario where all the conditions
    /// are met.
    ///
    /// NOTE: This calculation MAY NOT reflect the “per-user” price-per-share,
    /// and instead should reflect the “average-user’s” price-per-share, meaning
    /// what the average user should expect to see when exchanging to and from.
    ///
    /// # Requirements
    ///
    /// * MUST NOT be inclusive of any fees that are charged against assets in
    ///   the Vault.
    /// * MUST NOT show any variations depending on the caller.
    /// * MUST NOT reflect slippage or other on-chain conditions, when
    ///   performing the actual exchange.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `shares` - Number of shares.
    ///
    /// # Errors
    ///
    /// *
    fn convert_to_assets(&mut self, shares: U256) -> Result<U256, Self::Error>;

    /// Returns the maximum amount of the underlying asset that can be deposited
    /// into the Vault for the receiver, through a deposit call.
    ///
    /// # Requirements
    ///
    /// * MUST return a limited value if receiver is subject to some deposit
    ///   limit.
    /// * MUST return 2 ** 256 - 1 if there is no limit on the maximum amount of
    ///   assets that may be deposited.
    /// * MUST NOT revert.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `receiver` - The address of the entity receiving the shares.
    fn max_deposit(&self, receiver: Address) -> U256;

    /// Allows an on-chain or off-chain user to simulate the effects of their
    /// deposit at the current block, given current on-chain conditions.
    ///
    /// NOTE: Any unfavorable discrepancy between convertToShares and
    /// previewDeposit SHOULD be considered slippage in share price or some
    /// other type of condition, meaning the depositor will lose assets by
    /// depositing.
    ///
    /// # Requirements
    ///
    /// * MUST return as close to and no more than the exact amount of Vault
    ///   shares that would be minted in a deposit call in the same transaction.
    ///   I.e. deposit should return the same or more shares as previewDeposit
    ///   if called in the same transaction.
    /// * MUST NOT account for deposit limits like those returned from
    ///   maxDeposit and should always act as though the deposit would be
    ///   accepted, regardless if the user has enough tokens approved, etc.
    /// * MUST be inclusive of deposit fees. Integrators should be aware of the
    ///   existence of deposit fees.
    /// * MUST NOT revert.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `assets` - Amount of the underlying asset to deposit.
    ///
    /// # Errors
    ///
    /// *
    fn preview_deposit(&mut self, assets: U256) -> Result<U256, Self::Error>;

    /// Mints shares Vault shares to receiver by depositing exactly amount of
    /// underlying tokens.
    ///
    /// NOTE: Most implementations will require pre-approval of the Vault with
    /// the Vault’s underlying asset token.
    ///
    /// # Requirements
    ///
    /// * MUST emit the Deposit event.
    /// * MAY support an additional flow in which the underlying tokens are
    ///   owned by the Vault contract before the deposit execution, and are
    ///   accounted for during deposit.
    /// * MUST revert if all of assets cannot be deposited (due to deposit limit
    ///   being reached, slippage, the user not approving enough underlying
    ///   tokens to the Vault contract, etc).
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `assets` - Amount of the underlying asset to deposit.
    /// * `receiver` - The address receiving the shares.
    ///
    /// # Errors
    ///
    /// *
    fn deposit(
        &mut self,
        assets: U256,
        receiver: Address,
    ) -> Result<U256, Self::Error>;

    /// Returns the maximum amount of the Vault shares that can be minted for
    /// the receiver, through a mint call.
    ///
    /// # Requirements
    ///
    /// * MUST return a limited value if receiver is subject to some mint limit.
    /// * MUST return 2 ** 256 - 1 if there is no limit on the maximum amount of
    ///   shares that may be minted.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `receiver` - The address of the entity receiving the shares.
    fn max_mint(&self, receiver: Address) -> U256;

    /// Allows an on-chain or off-chain user to simulate the effects of their
    /// mint at the current block, given current on-chain conditions.
    ///
    /// NOTE: Any unfavorable discrepancy between convertToAssets and
    /// previewMint SHOULD be considered slippage in share price or some other
    /// type of condition, meaning the depositor will lose assets by minting.
    ///
    /// # Requirements
    ///
    /// * MUST return as close to and no fewer than the exact amount of assets
    ///   that would be deposited in a mint call in the same transaction. I.e.
    ///   mint should return the same or fewer assets as previewMint if called
    ///   in the same transaction.
    /// * MUST NOT account for mint limits like those returned from maxMint and
    ///   should always act as though the mint would be accepted, regardless if
    ///   the user has enough tokens approved, etc.
    /// * MUST be inclusive of deposit fees. Integrators should be aware of the
    ///   existence of deposit fees.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `shares` - Number of shares to mint.
    ///
    /// # Errors
    ///
    /// *
    fn preview_mint(&mut self, shares: U256) -> Result<U256, Self::Error>;

    /// Mints exactly shares Vault shares to receiver by depositing amount of
    /// underlying tokens.
    ///
    /// NOTE: Most implementations will require pre-approval of the Vault with
    /// the Vault’s underlying asset token.
    ///
    /// # Requirements
    ///
    /// * MUST emit the [`Deposit`] event.
    /// * MAY support an additional flow in which the underlying tokens are
    ///   owned by the Vault contract before the mint execution, and are
    ///   accounted for during mint.
    /// * MUST revert if all of shares cannot be minted (due to deposit limit
    ///   being reached, slippage, the user not approving enough underlying
    ///   tokens to the Vault contract, etc).
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `shares` - Number of shares to mint.
    /// * `receiver` - The address receiving the shares.
    ///
    /// # Errors
    ///
    /// *
    fn mint(
        &mut self,
        shares: U256,
        receiver: Address,
    ) -> Result<U256, Self::Error>;

    /// Returns the maximum amount of the underlying asset that can be withdrawn
    /// from the owner balance in the Vault, through a withdraw call.
    ///
    /// # Requirements
    ///
    /// * MUST return a limited value if owner is subject to some withdrawal
    ///   limit or timelock.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - The address of the entity owning the shares.
    ///
    /// # Errors
    ///
    /// *
    fn max_withdraw(&mut self, owner: Address) -> Result<U256, Self::Error>;

    /// Allows an on-chain or off-chain user to simulate the effects of their
    /// withdrawal at the current block, given current on-chain conditions.
    ///
    /// # Requirements
    ///
    /// * MUST return as close to and no fewer than the exact amount of Vault
    ///   shares that would be burned in a withdraw call in the same
    ///   transaction. I.e. withdraw should return the same or fewer shares as
    ///   previewWithdraw if called in the same transaction.
    /// * MUST NOT account for withdrawal limits like those returned from
    ///   maxWithdraw and should always act as though the withdrawal would be
    ///   accepted, regardless if the user has enough shares, etc.
    /// * MUST be inclusive of withdrawal fees. Integrators should be aware of
    ///   the existence of withdrawal fees.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `assets` - Amount of the underlying asset to withdraw.
    ///
    /// # Errors
    ///
    /// *
    fn preview_withdraw(&mut self, assets: U256) -> Result<U256, Self::Error>;

    /// Burns shares from owner and sends exactly assets of underlying tokens to
    /// receiver.
    ///
    /// Note that some implementations will require pre-requesting to the Vault
    /// before a withdrawal may be performed. Those methods should be performed
    /// separately.
    ///
    /// # Requirements
    ///
    /// * MUST emit the [`Withdraw`] event.
    /// * MAY support an additional flow in which the underlying tokens are
    ///   owned by the Vault contract before the withdraw execution, and are
    ///   accounted for during withdraw.
    /// * MUST revert if all of assets cannot be withdrawn (due to withdrawal
    ///   limit being reached, slippage, the owner not having enough shares,
    ///   etc).
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
    /// *
    fn withdraw(
        &mut self,
        assets: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Error>;

    /// Returns the maximum amount of Vault shares that can be redeemed from the
    /// owner balance in the Vault, through a redeem call.
    ///
    /// # Requirements
    ///
    /// * MUST return a limited value if owner is subject to some withdrawal
    ///   limit or timelock.
    /// * MUST return balanceOf(owner) if owner is not subject to any withdrawal
    ///   limit or timelock.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - The address of the entity owning the shares.
    fn max_redeem(&self, owner: Address) -> U256;

    /// Allows an on-chain or off-chain user to simulate the effects of their
    /// redeemption at the current block, given current on-chain conditions.
    ///
    /// NOTE: Any unfavorable discrepancy between convertToAssets and
    /// previewRedeem SHOULD be considered slippage in share price or some other
    /// type of condition, meaning the depositor will lose assets by redeeming.
    ///
    /// # Requirements
    ///
    /// * MUST return as close to and no more than the exact amount of assets
    ///   that would be withdrawn in a redeem call in the same transaction. I.e.
    ///   redeem should return the same or more assets as previewRedeem if
    ///   called in the same transaction.
    /// * MUST NOT account for redemption limits like those returned from
    ///   maxRedeem and should always act as though the redemption would be
    ///   accepted, regardless if the user has enough shares, etc.
    /// * MUST be inclusive of withdrawal fees. Integrators should be aware of
    ///   the existence of withdrawal fees.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `shares` - Number of shares to redeem.
    ///
    /// # Errors
    ///
    /// *
    fn preview_redeem(&mut self, shares: U256) -> Result<U256, Self::Error>;

    /// Burns exactly shares from owner and sends assets of underlying tokens to
    /// receiver.
    ///
    /// # Requirements
    ///
    /// * MUST emit the [`Withdraw`] event.
    /// * MAY support an additional flow in which the underlying tokens are
    ///   owned by the Vault contract before the redeem execution, and are
    ///   accounted for during redeem.
    /// * MUST revert if all of shares cannot be redeemed (due to withdrawal
    ///   limit being reached, slippage, the owner not having enough shares,
    ///   etc).
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
    /// *
    fn redeem(
        &mut self,
        shares: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Self::Error>;
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc4626 {}

#[public]
impl IErc4626 for Erc4626 {
    type Error = Error;

    fn asset(&self) -> Address {
        *self.asset
    }

    fn total_assets(&mut self) -> Result<U256, Self::Error> {
        self.erc20_balance_of(self.asset())
    }

    fn convert_to_shares(&mut self, assets: U256) -> Result<U256, Self::Error> {
        self._convert_to_shares(assets, Rounding::Floor)
    }

    fn convert_to_assets(&mut self, shares: U256) -> Result<U256, Self::Error> {
        self._convert_to_assets(shares, Rounding::Floor)
    }

    fn max_deposit(&self, _receiver: Address) -> U256 {
        U256::MAX
    }

    fn preview_deposit(&mut self, assets: U256) -> Result<U256, Self::Error> {
        self._convert_to_shares(assets, Rounding::Floor)
    }

    fn deposit(
        &mut self,
        assets: U256,
        receiver: Address,
    ) -> Result<U256, Self::Error> {
        let max_assets = self.max_deposit(receiver);

        if assets > max_assets {
            return Err(Error::ExceededMaxDeposit(ERC4626ExceededMaxDeposit {
                receiver,
                assets,
                max: max_assets,
            }));
        }

        let shares = self.preview_deposit(assets)?;

        self._deposit(msg::sender(), receiver, assets, shares)?;

        Ok(shares)
    }

    fn max_mint(&self, _receiver: Address) -> U256 {
        U256::MAX
    }

    fn preview_mint(&mut self, shares: U256) -> Result<U256, Self::Error> {
        self._convert_to_assets(shares, Rounding::Ceil)
    }

    fn mint(&mut self, shares: U256, receiver: Address) -> Result<U256, Error> {
        let max_shares = self.max_mint(receiver);

        if shares > max_shares {
            return Err(Error::ExceededMaxMint(ERC4626ExceededMaxMint {
                receiver,
                shares,
                max: max_shares,
            }));
        }

        let assets = self.preview_mint(shares)?;
        self._deposit(msg::sender(), receiver, assets, shares)?;

        Ok(assets)
    }

    fn max_withdraw(&mut self, owner: Address) -> Result<U256, Self::Error> {
        let balance = self.erc20.balance_of(owner);
        self._convert_to_assets(balance, Rounding::Floor)
    }

    fn preview_withdraw(&mut self, assets: U256) -> Result<U256, Self::Error> {
        self._convert_to_shares(assets, Rounding::Ceil)
    }

    fn withdraw(
        &mut self,
        assets: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Error> {
        let max_assets = self.max_withdraw(owner)?;

        if assets > max_assets {
            return Err(Error::ExceededMaxWithdraw(
                ERC4626ExceededMaxWithdraw { owner, assets, max: max_assets },
            ));
        }

        let shares = self.preview_redeem(assets)?;
        self._withdraw(msg::sender(), receiver, owner, assets, shares)?;

        Ok(shares)
    }

    fn max_redeem(&self, owner: Address) -> U256 {
        self.erc20.balance_of(owner)
    }

    fn preview_redeem(&mut self, shares: U256) -> Result<U256, Self::Error> {
        self._convert_to_assets(shares, Rounding::Floor)
    }

    fn redeem(
        &mut self,
        shares: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Self::Error> {
        let max_shares = self.max_redeem(owner);
        if shares > max_shares {
            return Err(Error::ExceededMaxRedeem(ERC4626ExceededMaxRedeem {
                owner,
                shares,
                max: max_shares,
            }));
        }

        let assets = self.preview_redeem(shares)?;

        self._withdraw(msg::sender(), receiver, owner, assets, shares)?;

        Ok(assets)
    }
}

impl Erc4626 {
    /// TODO: Rust docs
    fn erc20_total_supply(&mut self, token: Address) -> Result<U256, Error> {
        let erc20 = IErc20Solidity::new(token);
        Ok(erc20
            .total_supply(Call::new_in(self))
            .map_err(|_| InvalidToken { token: self.asset() })?)
    }

    /// TODO: Rust docs
    fn erc20_balance_of(&mut self, token: Address) -> Result<U256, Error> {
        let erc20 = IErc20Solidity::new(token);
        Ok(erc20
            .balance_of(Call::new_in(self), contract::address())
            .map_err(|_| InvalidToken { token: self.asset() })?)
    }
}

impl Erc4626 {
    /// TODO: Rust docs
    fn _convert_to_shares(
        &mut self,
        assets: U256,
        rounding: Rounding,
    ) -> Result<U256, Error> {
        let total_supply = self.erc20_total_supply(self.asset())?;

        let shares = assets.mul_div(
            total_supply
                + U256::from(10)
                    .checked_pow(U256::from(Self::_decimals_offset()))
                    .expect("overflow in `Erc4626::_convert_to_shares`"),
            self.total_assets()? + U256::from(1),
            rounding,
        );

        Ok(shares)
    }

    /// TODO: Rust docs
    fn _convert_to_assets(
        &mut self,
        shares: U256,
        rounding: Rounding,
    ) -> Result<U256, Error> {
        let total_supply = self.erc20_total_supply(self.asset())?;

        let assets = shares.mul_div(
            self.total_assets()? + U256::from(1),
            total_supply
                + U256::from(10)
                    .checked_pow(U256::from(Self::_decimals_offset()))
                    .expect("overflow in `Erc4626::_convert_to_assets`"),
            rounding,
        );

        Ok(assets)
    }

    /// TODO: Rust docs
    fn _deposit(
        &mut self,
        caller: Address,
        receiver: Address,
        assets: U256,
        shares: U256,
    ) -> Result<(), Error> {
        // If asset() is ERC-777, `transferFrom` can trigger a reentrancy BEFORE
        // the transfer happens through the `tokensToSend` hook. On the
        // other hand, the `tokenReceived` hook, that is triggered after the
        // transfer, calls the vault, which is assumed not malicious.
        //
        // Conclusion: we need to do the transfer before we mint so that any
        // reentrancy would happen before the assets are transferred and
        // before the shares are minted, which is a valid state.
        // slither-disable-next-line reentrancy-no-eth

        self.safe_erc20.safe_transfer_from(
            self.asset(),
            caller,
            contract::address(),
            assets,
        )?;

        self.erc20._mint(receiver, shares)?;

        evm::log(Deposit { sender: caller, owner: receiver, assets, shares });

        Ok(())
    }

    /// TODO: Rust docs
    fn _withdraw(
        &mut self,
        caller: Address,
        receiver: Address,
        owner: Address,
        assets: U256,
        shares: U256,
    ) -> Result<(), Error> {
        if caller != owner {
            self.erc20._spend_allowance(owner, caller, shares)?;
        }

        // If asset() is ERC-777, `transfer` can trigger a reentrancy AFTER the
        // transfer happens through the `tokensReceived` hook. On the
        // other hand, the `tokensToSend` hook, that is triggered before the
        // transfer, calls the vault, which is assumed not malicious.
        //
        // Conclusion: we need to do the transfer after the burn so that any
        // reentrancy would happen after the shares are burned and after
        // the assets are transferred, which is a valid state.

        self.erc20._burn(owner, shares)?;

        self.safe_erc20.safe_transfer(self.asset(), receiver, assets)?;

        evm::log(Withdraw { sender: caller, receiver, owner, assets, shares });

        Ok(())
    }

    /// TODO: Rust docs
    fn _decimals_offset() -> u8 {
        0
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, U256};
    use stylus_sdk::prelude::storage;

    use super::{Erc4626, IErc4626};
    use crate::token::erc20::Erc20;

    // const ALICE: Address =
    // address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    // const TOKEN_ADDRESS: Address =
    // address!("dce82b5f92c98f27f116f70491a487effdb6a2a9");
    // const INVALID_TOKEN_ADDRESS: Address =
    // address!("dce82b5f92c98f27f116f70491a487effdb6a2aa");

    #[storage]
    struct Erc4626TestExample {
        pub erc4626: Erc4626,
        pub erc20: Erc20,
    }

    /*
    #[motsu::test]
    fn max_mint(contract: Erc4626TestExample) {
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
        let max_mint = contract.erc4626.max_mint(bob);
        assert_eq!(max_mint, U256::MAX);
    }

    #[motsu::test]
    fn max_deposit(contract: Erc4626TestExample) {
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
        let max_deposit = contract.erc4626.max_deposit(bob);
        assert_eq!(max_deposit, U256::MAX);
    }

    #[motsu::test]
    fn convert_to_shares(contract: Erc4626TestExample) {
        let assets = U256::from(100);
        let shares =
            contract.erc4626.convert_to_shares(assets, &mut contract.erc20);
        assert_eq!(shares, U256::from(100));
    }

    #[motsu::test]
    fn convert_to_assets(contract: Erc4626TestExample) {
        let shares = U256::from(100);
        let assets =
            contract.erc4626.convert_to_assets(shares, &mut contract.erc20);
        assert_eq!(assets, U256::from(100));
    }
    */
}
