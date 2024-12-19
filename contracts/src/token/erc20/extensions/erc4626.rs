use alloy_primitives::{ keccak256, Address, B256, U256};
use alloy_sol_macro::sol;
use stylus_sdk::{
     contract, evm, msg, stylus_proc::{public, sol_storage, SolidityError}
};
use crate::token::erc20::{
    utils::SafeErc20,
    self, Erc20, IErc20
};

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
    fn deposit(&self,assets: U256, receiver: Address) -> U256;

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
    fn mint(&mut self, shares: U256, receiver: Address) -> U256;

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
    fn withdraw(&self,assets: U256, receiver: Address, owner: Address) -> U256;

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
    fn redeem(&self,shares: U256, receiver: Address, owner: Address) -> U256;
}


sol_storage! {
    #[allow(clippy::pub_underscore_fields)]
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


#[public]
impl IERC4626 for ERC4626 {
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
    
    /// Returns the maximum amount of assets that can be deposited for the given receiver.
    /// 
    /// # Parameters
    /// - `receiver`: The address intended to receive the deposited assets.
    ///
    /// # Returns
    /// The maximum number of assets that can be deposited, which is `U256::MAX`.
    
    fn max_deposit(&self,_receiver: Address) -> U256 {
        U256::MAX
    }
    
    fn preview_deposit(&self,assets: U256) -> U256 {
        todo!()
    }
    
    fn deposit(&self,assets: U256, receiver: Address) -> U256 {
        todo!()
    }
    
    /// Returns the maximum amount of shares that can be minted for the given receiver.
    /// 
    /// # Parameters
    /// - `_receiver`: The address intended to receive the minted shares.
    ///
    /// # Returns
    /// The maximum number of shares that can be minted, which is `U256::MAX`.
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
        self._deposit(msg::sender(), receiver, assets, shares);
        Ok(assets)
    }
    
    fn max_withdraw(&self,owner: Address) -> U256 {
        todo!()
    }
    
    fn preview_withdraw( &self,assets: U256) -> U256 {
        todo!()
    }
    
    fn withdraw(&self,assets: U256, receiver: Address, owner: Address) -> U256 {
        todo!()
    }
    
    fn max_redeem(&self,owner: Address) -> U256 {
        self._asset.balance_of(owner)
    }
    
    fn preview_redeem(&self,shares: U256) -> U256 {
        todo!()
    }
    
    fn redeem(&self,shares: U256, receiver: Address, owner: Address) -> U256 {
        todo!()
    }
}


impl  ERC4626 {
       fn _convert_to_shares(&self, assets: U256, rounding:bool) -> U256  {
         let confersion_factor = self._asset._total_supply.get()  + U256::from(self._decimals_offset());
         let dominitor = self.total_assets() + U256::from(1);
         assets * confersion_factor / dominitor
       }

       fn _convert_to_assets(&self, shares: U256, rounding:bool) -> U256  { 
         let confersion_factor = self._asset._total_supply.get()  + U256::from(self._decimals_offset());
         let dominitor = self.total_assets() + U256::from(1);
         shares * confersion_factor / dominitor
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
}