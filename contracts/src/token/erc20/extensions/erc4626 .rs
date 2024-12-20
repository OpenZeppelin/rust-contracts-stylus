//! ERC-4626 Tokenized Vault Standard Implementation
//!
//! Extends ERC-20 for vaults, enabling minting and burning of shares
//! in exchange for underlying assets. This contract provides standardized
//! workflows for deposits, minting, redemption, and burning of assets.
//! Note: The shares are minted and burned in relation to the assets via
//! the `deposit`, `mint`, `redeem`, and `burn` methods, affecting only
//! the shares token, not the asset token itself.
//! 
//! [ERC]: https://eips.ethereum.org/EIPS/eip-4626

use alloy_primitives::{  Address, U256};
use alloy_sol_macro::sol;
use stylus_sdk::{
     evm, prelude::*, 
     storage::{StorageAddress, StorageMap, StorageU8}, 
     stylus_proc::{public, SolidityError}
};
use crate::token::erc20::{
    self, utils::{safe_erc20, SafeErc20}, Erc20, IErc20
};
use crate::utils::math::alloy::Math;


pub trait IERC4626 {
    /// Error type associated with the operations, convertible to a vector of bytes.
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
    fn convert_to_shares(&self,assets: U256) -> U256;

    /// Converts a given number of shares into the equivalent amount of assets.
    ///
    /// # Parameters
    /// - `shares`: Number of shares.
    ///
    /// # Returns
    /// The corresponding amount of assets.
    fn convert_to_assets(&self,shares: U256) -> U256;

    /// Calculates the maximum amount of assets that can be deposited for a given receiver.
    ///
    /// # Parameters
    /// - `receiver`: The address of the entity receiving the shares.
    ///
    /// # Returns
    /// The maximum depositable amount.
    fn max_deposit(&self,receiver: Address) -> U256;

    /// Previews the outcome of depositing a specific amount of assets.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset to deposit.
    ///
    /// # Returns
    /// The number of shares that would be issued.
    fn preview_deposit(&self,assets: U256) -> U256;

    /// Deposits a specific amount of assets into the vault, issuing shares to the receiver.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset to deposit.
    /// - `receiver`: The address receiving the shares.
    ///
    /// # Returns
    /// The number of shares issued.
    fn deposit(&mut self,assets: U256, receiver: Address) -> Result<U256, Self::Error>;

    /// Calculates the maximum number of shares that can be minted for a given receiver.
    ///
    /// # Parameters
    /// - `receiver`: The address of the entity receiving the shares.
    ///
    /// # Returns
    /// The maximum mintable number of shares.
    fn max_mint(&self,receiver: Address) -> U256;

    /// Previews the outcome of minting a specific number of shares.
    ///
    /// # Parameters
    /// - `shares`: Number of shares to mint.
    ///
    /// # Returns
    /// The equivalent amount of assets required.
    fn preview_mint(&self,shares: U256) -> U256;

    /// Mints a specific number of shares for a given receiver.
    ///
    /// # Parameters
    /// - `shares`: Number of shares to mint.
    /// - `receiver`: The address receiving the shares.
    ///
    /// # Returns
    /// The amount of assets deposited.
    fn mint(&mut self, shares: U256, receiver: Address) -> Result<U256, Self::Error>;

    /// Calculates the maximum amount of assets that can be withdrawn by a given owner.
    ///
    /// # Parameters
    /// - `owner`: The address of the entity owning the shares.
    ///
    /// # Returns
    /// The maximum withdrawable amount.
    fn max_withdraw(&self,owner: Address) -> U256;

    /// Previews the outcome of withdrawing a specific amount of assets.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset to withdraw.
    ///
    /// # Returns
    /// The equivalent number of shares required.
    fn preview_withdraw( &self,assets: U256) -> U256;

    /// Withdraws a specific amount of assets from the vault, deducting shares from the owner.
    ///
    /// # Parameters
    /// - `assets`: Amount of the underlying asset to withdraw.
    /// - `receiver`: The address receiving the withdrawn assets.
    /// - `owner`: The address owning the shares to be deducted.
    ///
    /// # Returns
    /// The number of shares burned.
    fn withdraw(&mut self,assets: U256, receiver: Address, owner: Address) -> Result<U256, Error>;

    /// Calculates the maximum number of shares that can be redeemed by a given owner.
    ///
    /// # Parameters
    /// - `owner`: The address of the entity owning the shares.
    ///
    /// # Returns
    /// The maximum redeemable number of shares.
    fn max_redeem(&self,owner: Address) -> U256;

    /// Previews the outcome of redeeming a specific number of shares.
    ///
    /// # Parameters
    /// - `shares`: Number of shares to redeem.
    ///
    /// # Returns
    /// The equivalent amount of assets returned.
    fn preview_redeem(&self,shares: U256) -> U256;

    /// Redeems a specific number of shares for the underlying assets, transferring them to the receiver.
    ///
    /// # Parameters
    /// - `shares`: Number of shares to redeem.
    /// - `receiver`: The address receiving the underlying assets.
    /// - `owner`: The address owning the shares to be redeemed.
    ///
    /// # Returns
    /// The amount of assets transferred.
    fn redeem(&mut self,shares: U256, receiver: Address, owner: Address) -> Result<U256, Self::Error>;
}

#[storage]
pub struct Erc4262 {
    pub __underlying_decimals: StorageU8,
    pub  _asset:StorageAddress,
}

sol! {
    
    #[allow(missing_docs)]
    event Deposit(address indexed sender, address indexed owner, uint256 assets, uint256 shares);

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
    /// Indicates an error where a deposit operation failed
    /// because the supplied `assets` exceeded the maximum allowed for the `receiver`.
    #[derive(Debug)]
    #[allow(missing_docs)]
     error ERC4626ExceededMaxDeposit(address receiver, uint256 assets, uint256 max);

    /// Indicates an error where a mint operation failed
    /// because the supplied `shares` exceeded the maximum allowed for the `receiver`.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC4626ExceededMaxMint(address receiver, uint256 shares, uint256 max);

    /// Indicates an error where a withdrawal operation failed
    /// because the supplied `assets` exceeded the maximum allowed for the `owner`
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC4626ExceededMaxWithdraw(address owner, uint256 assets, uint256 max);

    /// Indicates an error where a redemption operation failed
    /// because the supplied `shares` exceeded the maximum allowed for the `owner`.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC4626ExceededMaxRedeem(address owner, uint256 shares, uint256 max);
}

#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error where a deposit operation failed
    /// because the supplied `assets` exceeded the maximum allowed for the `receiver`.
    ExceededMaxDeposit(ERC4626ExceededMaxDeposit),

    /// Indicates an error where a mint operation failed
    /// because the supplied `shares` exceeded the maximum allowed for the `receiver`
    ExceededMaxMint(ERC4626ExceededMaxMint),

    /// Indicates an error where a withdrawal operation failed
    /// because the supplied `assets` exceeded the maximum allowed for the `owner`
    ExceededMaxWithdraw(ERC4626ExceededMaxWithdraw),

    /// Indicates an error where a redemption operation failed
    /// because the supplied `shares` exceeded the maximum allowed for the `owner`.
    ExceededMaxRedeem(ERC4626ExceededMaxRedeem),

    /// Error type from [`SafeErc20`] contract [`safe_erc20::Error`].
    SafeErc20(safe_erc20::Error),

    /// Error type from [`Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error)
}


#[storage]
pub struct Erc4626 {

    pub _asset: Erc20,

    pub _safe_erc20: SafeErc20,
    
    pub _underlying_decimals: StorageU8,
}

#[public]
impl IERC4626 for Erc4626 {
    type Error = Error;
    fn asset(&self) -> Address {
        contract::address()
    }
    
    fn total_assets(&self) -> U256 {
        self._asset.balance_of(contract::address())
    }
    

    fn convert_to_shares(&self,assets: U256) -> U256 {
        todo!()
    }
    

    fn convert_to_assets(&self,shares: U256) -> U256 {
        todo!()
    }
    
    
    fn max_deposit(&self,_receiver: Address) -> U256 {
        U256::MAX
    }
    
    fn preview_deposit(&self,assets: U256) -> U256 {
        todo!()
    }

    fn deposit(&mut self,assets: U256, receiver: Address) -> Result<U256, Error> {
        let max_assets = self.max_deposit(receiver);
        if assets > max_assets {
            return Err(Error::ExceededMaxDeposit(ERC4626ExceededMaxDeposit{ 
                receiver, 
                assets, 
                max: max_assets
            }));   
        }

        let shares = self.preview_deposit(assets);
        self._deposit(msg::sender(), receiver, assets, shares)?;
        Ok(shares)
    }
    
   fn max_mint(&self, _receiver: Address) -> U256 {
        U256::MAX
   }
    
    fn preview_mint(&self,shares: U256) -> U256 {
        todo!()
    }
    
    fn mint(&mut self,shares: U256, receiver: Address) -> Result<U256, Error> {
        let  max_shares = self.max_mint(receiver);
        if shares > max_shares {
               return Err(Error::ExceededMaxMint(ERC4626ExceededMaxMint{
                receiver,
                shares,
                max: max_shares
            }));
        }
        let assets  = self.preview_mint(shares);
        self._deposit(msg::sender(), receiver, assets, shares)?;
        Ok(assets)
    }
    

    fn max_withdraw(&self,owner: Address) -> U256 {
        todo!()
    }
    
    fn preview_withdraw( &self,assets: U256) -> U256 {
        todo!()
    }
    
    fn withdraw(&mut self,assets: U256, receiver: Address, owner: Address) -> Result<U256, Error> {
        let  max_assets = self.max_withdraw(owner);
        if assets > max_assets {
               return Err(Error::ExceededMaxWithdraw(ERC4626ExceededMaxWithdraw{
                owner,
                assets,
                max: max_assets
            }));
        }

        let  shares = self.preview_redeem(assets);
        self._withdraw(msg::sender(), receiver, owner, assets, shares)?;
        Ok(assets)
    }
    
    fn max_redeem(&self,owner: Address) -> U256 {
        self._asset.balance_of(owner)
    }
    
    fn preview_redeem(&self,shares: U256) -> U256 {
        self.
    }
    
    fn redeem(&mut self,shares: U256, receiver: Address, owner: Address) -> Result<U256, Error> {
        let  max_shares = self.max_redeem(owner);
        if shares > max_shares {
               return Err(Error::ExceededMaxRedeem(ERC4626ExceededMaxRedeem{ 
                owner, shares,
                max: max_shares
            }));
        }

        let  assets = self.preview_redeem(shares);
        self._withdraw(msg::sender(), receiver, owner, assets, shares)?;
        Ok(assets)
    }
}


impl Erc4626 {
       fn _convert_to_shares(&self, assets: U256, rounding:bool) -> U256  {
         assets.
       }

       fn _convert_to_assets(&self, shares: U256, rounding:bool) -> U256  { 
          self.
       }

        fn _deposit(&mut self,caller:Address,receiver:Address, assets:U256, shares:U256) -> Result<(), Error> {
            // _SafeERC20.safeTransferFrom(_asset, caller, address(this), assets);
            self._asset._mint(receiver, shares)?;
            evm::log( Deposit { sender:caller,  owner:receiver, assets, shares });
            Ok(())
        }

        fn _withdraw(&mut self,caller:Address,receiver:Address,owner: Address, assets: U256, shares:U256) -> Result<(), Error> {
                if caller != owner {
                    self._asset._spend_allowance(owner, caller, shares)?;
                }

                // If _asset is ERC-777, `transfer` can trigger a reentrancy AFTER the transfer happens through the
                // `tokensReceived` hook. On the other hand, the `tokensToSend` hook, that is triggered before the transfer,
                // calls the vault, which is assumed not malicious.
                //
                // Conclusion: we need to do the transfer after the burn so that any reentrancy would happen after the
                // shares are burned and after the assets are transferred, which is a valid state.
                self._asset._burn(owner, shares)?;
                //SafeERC20.safeTransfer(_asset, receiver, assets);

                evm::log(Withdraw {sender: caller, receiver, owner, assets, shares});
                Ok(())
            }

        /// Offset of the decimals of the ERC-20 asset from the decimals of the Vault.
        ///
        /// This value is used to calculate the number of shares that can be minted
        /// for a given amount of assets, and to calculate the number of assets
        /// that can be withdrawn for a given amount of shares.
        ///
        /// The value is set to 0 by default, which means that the decimals of the
        /// ERC-20 asset and the Vault are the same.
        ///
        /// To change this value, you must override this function in your contract.
        fn _decimals_offset(&self)  -> u32 {
            0
        }

        //fn _try_get_asset_decimals(asset_: Address) private view returns (bool ok, uint8 assetDecimals) {
        // (bool success, bytes memory encodedDecimals) = address(asset_).staticcall(
        //     abi.encodeCall(IERC20Metadata.decimals, ())
        // );
        // if (success && encodedDecimals.length >= 32) {
        //     uint256 returnedDecimals = abi.decode(encodedDecimals, (uint256));
        //     if (returnedDecimals <= type(uint8).max) {
        //         return (true, uint8(returnedDecimals));
        //     }
        // }
        // return (false, 0);
    //}
}