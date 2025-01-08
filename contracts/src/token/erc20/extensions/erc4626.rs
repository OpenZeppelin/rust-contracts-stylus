//! ERC-4626 Tokenized Vault Standard Implementation.
//!
//! Extends ERC-20 for vaults, enabling minting and burning of "shares"
//! (represented using the [`ERC-20`] inheritance) in exchange for underlying
//! assets. This contract provides standardized workflows for deposits, minting,
//! redemption, and burning of assets. Note: The shares are minted and burned in
//! relation to the assets via the `deposit`, `mint`, `redeem`, and `burn`
//! methods, affecting only the shares token, not the asset token itself.
//!
//! [ERC]: https://eips.ethereum.org/EIPS/eip-4626
//!
//! [CAUTION]
//! In empty (or nearly empty) ERC-4626 vaults, deposits are at high risk of
//! being stolen through frontrunning with a "donation" to the vault that
//! inflates the price of a share. This is variously known as a donation or
//! inflation attack and is essentially a problem of slippage. Vault deployers
//! can protect against this attack by making an initial deposit of a
//! non-trivial amount of the asset, such that price manipulation becomes
//! infeasible. Withdrawals may similarly be affected by slippage. Users can
//! protect against this attack as well as unexpected slippage in general by
//! verifying the amount received is as expected, using a wrapper that performs
//! these checks such as https://github.com/fei-protocol/ERC4626#erc4626router-and-base[ERC4626Router]
//!
//! The `_decimalsOffset()` corresponds to an offset in the decimal
//! representation between the underlying asset's decimals and the vault
//! decimals. This offset also determines the rate of virtual shares to virtual
//! assets in the vault, which itself determines the initial exchange rate.
//! While not fully preventing the attack, analysis shows that the default
//! offset (0) makes it non-profitable even if an attacker is able to capture
//! value from multiple user deposits, as a result of the value being captured
//! by the virtual shares (out of the attacker's donation) matching the
//! attacker's expected gains. With a larger offset, the attack becomes orders
//! of magnitude more expensive than it is profitable. More details about the
//! underlying math can be found xref:erc4626.adoc#inflation-attack[here].
//!
//! The drawback of this approach is that the virtual shares do capture (a very
//! small) part of the value being accrued to the vault. Also, if the vault
//! experiences losses, the users try to exit the vault, the virtual shares and
//! assets will cause the first user to exit to experience reduced losses in
//! detriment to the last users that will experience bigger losses.
//!
//! To learn more, check out our xref:ROOT:erc4626.adoc[ERC-4626 guide]..

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use alloy_sol_macro::sol;
use stylus_sdk::{
    contract, evm, msg,
    prelude::storage,
    storage::{StorageAddress, TopLevelStorage},
    stylus_proc::{public, SolidityError},
};

use crate::{
    token::erc20::{
        self,
        utils::{
            safe_erc20::{self, ISafeErc20},
            SafeErc20,
        },
        Erc20, IErc20,
    },
    utils::Metadata,
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
}

/// An [`Erc4626`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`SafeErc20`] contract [`safe_erc20::Error`].
    SafeErc20(safe_erc20::Error),
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
    /// Error type from [`Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error),
}

/// State of an [`Erc4626`] token.
#[storage]
pub struct Erc4626 {
    /// ERC-20 contract storage.
    pub _asset: Erc20,
    /// ERC20 metadata extenston .
    pub _metadata: Metadata,

    /// Token Address of the vault
    pub _token_address: StorageAddress,

    /// ERC-20 contract.
    pub erc20: Erc20,

    /// [`SafeErc20`] contract.
    pub _safe_erc20: SafeErc20,
}

/// ERC-4626 Tokenized Vault Standard Interface
pub trait IERC4626 {
    /// The error type associated to this ERC-4626 trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the address of the underlying asset that the vault manages.
    fn asset(&self) -> Address;

    /// Returns the total amount of the underlying asset held in the vault.
    fn total_assets(&self) -> U256;

    /// Converts a given amount of assets into the equivalent number of shares.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset.
    ///
    /// # Returns
    /// The corresponding amount of shares.
    fn convert_to_shares(&self, assets: U256) -> U256;

    /// Converts a given number of shares into the equivalent amount of assets.
    ///
    /// # Parameters
    /// - `shares`: Number of shares.
    ///
    /// # Returns
    /// The corresponding amount of assets.
    fn convert_to_assets(&self, shares: U256) -> U256;

    /// Calculates the maximum amount of assets that can be deposited for a
    /// given receiver.
    ///
    /// # Parameters
    /// - `receiver`: The address of the entity receiving the shares.
    ///
    /// # Returns
    /// The maximum depositable amount.
    fn max_deposit(&self, receiver: Address) -> U256;

    /// Previews the outcome of depositing a specific amount of assets.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset to deposit.
    ///
    /// # Returns
    /// The number of shares that would be issued.
    fn preview_deposit(&self, assets: U256) -> U256;

    /// Deposits a specific amount of assets into the vault, issuing shares to
    /// the receiver.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset to deposit.
    /// - `receiver`: The address receiving the shares.
    ///
    /// # Returns
    /// The number of shares issued.
    fn deposit(
        &mut self,
        assets: U256,
        receiver: Address,
    ) -> Result<U256, Self::Error>;

    /// Calculates the maximum number of shares that can be minted for a given
    /// receiver.
    ///
    /// # Parameters
    /// - `receiver`: The address of the entity receiving the shares.
    ///
    /// # Returns
    /// The maximum mintable number of shares.
    fn max_mint(&self, receiver: Address) -> U256;

    /// Previews the outcome of minting a specific number of shares.
    ///
    /// # Parameters
    /// - `shares`: Number of shares to mint.
    ///
    /// # Returns
    /// The equivalent amount of assets required.
    fn preview_mint(&self, shares: U256) -> U256;

    /// Mints a specific number of shares for a given receiver.
    ///
    /// # Parameters
    /// - `shares`: Number of shares to mint.
    /// - `receiver`: The address receiving the shares.
    ///
    /// # Returns
    /// The amount of assets deposited.
    fn mint(
        &mut self,
        shares: U256,
        receiver: Address,
    ) -> Result<U256, Self::Error>;

    /// Calculates the maximum amount of assets that can be withdrawn by a given
    /// owner.
    ///
    /// # Parameters
    /// - `owner`: The address of the entity owning the shares.
    ///
    /// # Returns
    /// The maximum withdrawable amount.
    fn max_withdraw(&self, owner: Address) -> U256;

    /// Previews the outcome of withdrawing a specific amount of assets.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset to withdraw.
    ///
    /// # Returns
    /// The equivalent number of shares required.
    fn preview_withdraw(&self, assets: U256) -> U256;

    /// Withdraws a specific amount of assets from the vault, deducting shares
    /// from the owner.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset to withdraw.
    /// - `receiver`: The address receiving the withdrawn assets.
    /// - `owner`: The address owning the shares to be deducted.
    ///
    /// # Returns
    /// The number of shares burned.
    fn withdraw(
        &mut self,
        assets: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Error>;

    /// Calculates the maximum number of shares that can be redeemed by a given
    /// owner.
    ///
    /// # Parameters
    /// - `owner`: The address of the entity owning the shares.
    ///
    /// # Returns
    /// The maximum redeemable number of shares.
    fn max_redeem(&self, owner: Address) -> U256;

    /// Previews the outcome of redeeming a specific number of shares.
    ///
    /// # Parameters
    /// - `shares`: Number of shares to redeem.
    ///
    /// # Returns
    /// The equivalent amount of assets returned.
    fn preview_redeem(&self, shares: U256) -> U256;

    /// Redeems a specific number of shares for the underlying assets,
    /// transferring them to the receiver.
    ///
    /// # Parameters
    /// - `shares`: Number of shares to redeem.
    /// - `receiver`: The address receiving the underlying assets.
    /// - `owner`: The address owning the shares to be redeemed.
    ///
    /// # Returns
    /// The amount of assets transferred.
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
impl IERC4626 for Erc4626 {
    type Error = Error;

    // fn name(&self) -> String {
    //     self._metadata.name()
    // }

    // fn symbol(&self) -> String {
    //     self._metadata.symbol()
    // }

    //  /// Returns the number of tokens in existence.
    // ///
    // /// # Arguments
    // ///
    // /// * `&self` - Read access to the contract's state.
    // pub fn total_supply(&self) -> U256 {
    //     self.erc20.total_supply()
    // }

    // /// Returns the number of tokens owned by `account`.
    // ///
    // /// # Arguments
    // ///
    // /// * `&self` - Read access to the contract's state.
    // /// * `account` - Account to get balance from.
    // pub fn balance_of(&self, account: Address) -> U256 {
    //     self.erc20.balance_of(account)
    // }

    // /// Moves a `value` amount of tokens from the caller's account to `to`.
    // ///
    // /// Returns a boolean value indicating whether the operation succeeded.
    // ///
    // /// # Arguments
    // ///
    // /// * `&mut self` - Write access to the contract's state.
    // /// * `to` - Account to transfer tokens to.
    // /// * `value` - Number of tokens to transfer.
    // ///
    // /// # Errors
    // ///
    // /// * If the `to` address is `Address::ZERO`, then the error
    // /// [`crate::token::erc20::Error::InvalidReceiver`] is returned.
    // /// * If the caller doesn't have a balance of at least `value`, then the
    // /// error [`crate::token::erc20::Error::InsufficientBalance`] is
    // returned. ///
    // /// # Events
    // ///
    // /// Emits a [`crate::token::erc20::Transfer`] event.
    // pub fn transfer(
    //     &mut self,
    //     to: Address,
    //     value: U256,
    // ) -> Result<bool, crate::token::erc20::Error> {
    //     self.erc20.transfer(to, value)
    // }

    // /// Returns the remaining number of tokens that `spender` will be allowed
    // /// to spend on behalf of `owner` through `transfer_from`. This is zero
    // by /// default.
    // ///
    // /// This value changes when `approve` or `transfer_from` are called.
    // ///
    // /// # Arguments
    // ///
    // /// * `&self` - Read access to the contract's state.
    // /// * `owner` - Account that owns the tokens.
    // /// * `spender` - Account that will spend the tokens.
    // pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
    //     self.erc20.allowance(owner, spender)
    // }

    // /// Sets a `value` number of tokens as the allowance of `spender` over
    // the /// caller's tokens.
    // ///
    // /// Returns a boolean value indicating whether the operation succeeded.
    // ///
    // /// WARNING: Beware that changing an allowance with this method brings
    // the /// risk that someone may use both the old and the new allowance
    // by /// unfortunate transaction ordering. One possible solution to
    // mitigate /// this race condition is to first reduce the `spender`'s
    // allowance to 0 /// and set the desired value afterwards:
    // /// <https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729>
    // ///
    // /// # Arguments
    // ///
    // /// * `&mut self` - Write access to the contract's state.
    // /// * `owner` - Account that owns the tokens.
    // /// * `spender` - Account that will spend the tokens.
    // /// * `value` - The number of tokens being allowed to transfer by
    // `spender`. ///
    // /// # Errors
    // ///
    // /// If the `spender` address is `Address::ZERO`, then the error
    // /// [`crate::token::erc20::Error::InvalidSpender`] is returned.
    // ///
    // /// # Events
    // ///
    // /// Emits an [`crate::token::erc20::Approval`] event.
    // pub fn approve(
    //     &mut self,
    //     spender: Address,
    //     value: U256,
    // ) -> Result<bool, crate::token::erc20::Error> {
    //     self.erc20.approve(spender, value)
    // }

    // /// Moves a `value` number of tokens from `from` to `to` using the
    // /// allowance mechanism. `value` is then deducted from the caller's
    // /// allowance.
    // ///
    // /// Returns a boolean value indicating whether the operation succeeded.
    // ///
    // /// NOTE: If `value` is the maximum `U256::MAX`, the allowance is not
    // /// updated on `transfer_from`. This is semantically equivalent to
    // /// an infinite approval.
    // ///
    // /// # Arguments
    // ///
    // /// * `&mut self` - Write access to the contract's state.
    // /// * `from` - Account to transfer tokens from.
    // /// * `to` - Account to transfer tokens to.
    // /// * `value` - Number of tokens to transfer.
    // ///
    // /// # Errors
    // ///
    // /// * If the `from` address is `Address::ZERO`, then the error
    // /// [`crate::token::erc20::Error::InvalidSender`] is returned.
    // /// * If the `to` address is `Address::ZERO`, then the error
    // /// [`crate::token::erc20::Error::InvalidReceiver`] is returned.
    // /// * If not enough allowance is available, then the error
    // /// [`crate::token::erc20::Error::InsufficientAllowance`] is returned.
    // ///
    // /// # Events
    // ///
    // /// Emits a [`crate::token::erc20::Transfer`] event.
    // pub fn transfer_from(
    //     &mut self,
    //     from: Address,
    //     to: Address,
    //     value: U256,
    // ) -> Result<bool, crate::token::erc20::Error> {
    //     self.erc20.transfer_from(from, to, value)
    // }

    fn asset(&self) -> Address {
        self._token_address.get()
    }

    fn total_assets(&self) -> U256 {
        self._asset.balance_of(contract::address())
    }

    fn convert_to_shares(&self, assets: U256) -> U256 {
        self._convert_to_shares(assets)
    }

    fn convert_to_assets(&self, shares: U256) -> U256 {
        self._convert_to_assets(shares)
    }

    fn max_deposit(&self, _receiver: Address) -> U256 {
        U256::MAX
    }

    fn preview_deposit(&self, assets: U256) -> U256 {
        self._convert_to_shares(assets)
    }

    fn deposit(
        &mut self,
        assets: U256,
        receiver: Address,
    ) -> Result<U256, Error> {
        let max_assets = self.max_deposit(receiver);
        if assets > max_assets {
            return Err(Error::ExceededMaxDeposit(ERC4626ExceededMaxDeposit {
                receiver,
                assets,
                max: max_assets,
            }));
        }

        let shares = self.preview_deposit(assets);
        self._deposit(msg::sender(), receiver, assets, shares)?;
        Ok(shares)
    }

    fn max_mint(&self, _receiver: Address) -> U256 {
        U256::MAX
    }

    fn preview_mint(&self, shares: U256) -> U256 {
        self._convert_to_assets(shares)
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
        let assets = self.preview_mint(shares);
        self._deposit(msg::sender(), receiver, assets, shares)?;
        Ok(assets)
    }

    fn max_withdraw(&self, owner: Address) -> U256 {
        self._convert_to_assets(self._asset.balance_of(owner))
    }

    fn preview_withdraw(&self, assets: U256) -> U256 {
        self._convert_to_shares(assets)
    }

    fn withdraw(
        &mut self,
        assets: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Error> {
        let max_assets = self.max_withdraw(owner);
        if assets > max_assets {
            return Err(Error::ExceededMaxWithdraw(
                ERC4626ExceededMaxWithdraw { owner, assets, max: max_assets },
            ));
        }

        let shares = self.preview_redeem(assets);
        self._withdraw(msg::sender(), receiver, owner, assets, shares)?;
        Ok(shares)
    }

    fn max_redeem(&self, owner: Address) -> U256 {
        self._asset.balance_of(owner)
    }

    fn preview_redeem(&self, shares: U256) -> U256 {
        self._convert_to_assets(shares)
    }

    fn redeem(
        &mut self,
        shares: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Error> {
        let max_shares = self.max_redeem(owner);
        if shares > max_shares {
            return Err(Error::ExceededMaxRedeem(ERC4626ExceededMaxRedeem {
                owner,
                shares,
                max: max_shares,
            }));
        }

        let assets = self.preview_redeem(shares);
        self._withdraw(msg::sender(), receiver, owner, assets, shares)?;
        Ok(assets)
    }
}

impl Erc4626 {
    fn _convert_to_shares(&self, assets: U256) -> U256 {
        let adjusted_total_supply = self._asset.total_supply()
            + U256::from(10u32.pow(self._decimals_offset()));
        let adjusted_total_assets = self.total_assets() + U256::from(1);
        self._mul_div(assets, adjusted_total_supply, adjusted_total_assets)
    }

    fn _convert_to_assets(&self, shares: U256) -> U256 {
        let adjusted_total_supply = self._asset.total_supply()
            + U256::from(10u32.pow(self._decimals_offset()));
        let adjusted_total_assets = self.total_assets() + U256::from(1);
        self._mul_div(shares, adjusted_total_assets, adjusted_total_supply)
    }

    fn _mul_div(&self, x: U256, y: U256, dominator: U256) -> U256 {
        x.saturating_mul(y).checked_div(dominator).unwrap_or(U256::ZERO)
    }

    fn _deposit(
        &mut self,
        caller: Address,
        receiver: Address,
        assets: U256,
        shares: U256,
    ) -> Result<(), Error> {
        // If _asset is ERC-777, `transferFrom` can trigger a reentrancy BEFORE
        // the transfer happens through the `tokensToSend` hook. On the
        // other hand, the `tokenReceived` hook, that is triggered after the
        // transfer, calls the vault, which is assumed not malicious.
        //
        // Conclusion: we need to do the transfer before we mint so that any
        // reentrancy would happen before the assets are transferred and
        // before the shares are minted, which is a valid state.
        // slither-disable-next-line reentrancy-no-eth

        self._asset._mint(receiver, shares)?;
        evm::log(Deposit { sender: caller, owner: receiver, assets, shares });
        Ok(())
    }

    fn _withdraw(
        &mut self,
        caller: Address,
        receiver: Address,
        owner: Address,
        assets: U256,
        shares: U256,
    ) -> Result<(), Error> {
        if caller != owner {
            self._asset._spend_allowance(owner, caller, shares)?;
        }

        // If _asset is ERC-777, `transfer` can trigger a reentrancy AFTER the
        // transfer happens through the `tokensReceived` hook. On the
        // other hand, the `tokensToSend` hook, that is triggered before the
        // transfer, calls the vault, which is assumed not malicious.
        //
        // Conclusion: we need to do the transfer after the burn so that any
        // reentrancy would happen after the shares are burned and after
        // the assets are transferred, which is a valid state.
        self._asset._burn(owner, shares)?;
        self._safe_erc20.safe_transfer(
            contract::address(),
            receiver,
            assets,
        )?;

        evm::log(Withdraw { sender: caller, receiver, owner, assets, shares });
        Ok(())
    }

    /// Offset of the decimals of the ERC-20 asset from the decimals of the
    /// Vault.
    ///
    /// This value is used to calculate the number of shares that can be minted
    /// for a given amount of assets, and to calculate the number of assets
    /// that can be withdrawn for a given amount of shares.
    ///
    /// The value is set to 0 by default, which means that the decimals of the
    /// ERC-20 asset and the Vault are the same.
    ///
    /// To change this value, you must override this function in your contract.
    fn _decimals_offset(&self) -> u32 {
        0
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, U256};

    use super::Erc4626;
    use crate::token::erc20::extensions::erc4626::IERC4626;

    #[motsu::test]
    fn max_mint(contract: Erc4626) {
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
        let max_mint = contract.max_mint(bob);
        assert_eq!(max_mint, U256::MAX);
    }

    #[motsu::test]
    fn max_deposit(contract: Erc4626) {
        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
        let max_deposit = contract.max_deposit(bob);
        assert_eq!(max_deposit, U256::MAX);
    }

    #[motsu::test]
    fn convert_to_shares(contract: Erc4626) {
        let assets = U256::from(100);
        let shares = contract.convert_to_shares(assets);
        assert_eq!(shares, U256::from(100));
    }

    #[motsu::test]
    fn convert_to_assets(contract: Erc4626) {
        let shares = U256::from(100);
        let assets = contract.convert_to_assets(shares);
        assert_eq!(assets, U256::from(100));
    }
}
