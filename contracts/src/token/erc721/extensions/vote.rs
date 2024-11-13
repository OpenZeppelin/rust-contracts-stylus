use core::u16;

use alloy_primitives::{b256, keccak256, Address, Uint, B256, U256, U32, U64};
use alloy_sol_types::{sol, SolType};
use stylus_sdk::{
    block,
    prelude::StorageType,
    storage::TopLevelStorage,
    stylus_proc::{public, sol_storage, SolidityError},
};

use crate::{
        governance::utils::vote::{self,  Vote},
        token::erc721::{self, Erc721, IErc721}, 
        utils::{cryptography::{ecdsa, eip712::IEip712}, structs::checkpoints::{self, Checkpoint, Size, S208}},
};


type U48 = <S208 as Size>::Key;
type U208 = <S208 as Size>::Value;



/// A Permit error.
#[derive(SolidityError, Debug)]
pub enum Error {
    // Error type from [`Vote`] contract [`vote::Error`].
    Vote(vote::Error),
    /// Error type from [`Erc721`] contract [`erc721::Error`].
    Erc721(erc721::Error),
    /// Error type from [`ecdsa`] contract [`ecdsa::Error`].
    ECDSA(ecdsa::Error),
}

sol_storage! {
    pub struct Erc721Vote<T: IEip712 + StorageType> {
        Erc721 erc721;
        /// Vote contract.
        Vote<T > vote;
    }
   
}


// impl Voting for  VoteType {
//     fn _get_voting_units(&self,account: Address) -> U256 {
//         //self
//         todo!()
//     }
    
//     // fn _increase_balance(&mut self, account: Address, amount: U256) {
//     //     todo!()
//     // }
    
//     // fn _update(to:Address, token_id:U256, auth:Address) {
//     //     todo!()
//     // }
// }

unsafe impl<T: IEip712 + StorageType> TopLevelStorage for  Erc721Vote<T> {}

#[public]
impl <T: IEip712 + StorageType> Erc721Vote<T> {
    /// Returns the number of checkpoints associated with a given account.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - The account for which to retrieve the number of checkpoints.
    ///
    /// # Returns
    ///
    /// * `u32` - The number of checkpoints for the specified account.
    fn  num_checkpoints(&self, account:Address) -> u32 { 
        self.vote._num_checkpoints(account)
    } 

    /// Returns a checkpoint at given position for the given account.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - The account for which to return the checkpoint.
    /// * `pos` - Index of the checkpoint.
    ///
    /// # Returns
    ///
    /// Tuple of `(key, value)` where `key` is the block number at which the
    /// checkpoint was recorded and `value` is the number of votes the account
    /// had at that time.
    fn checkpoints(&self, account:Address, pos:u32) ->  (u64, U256) {
       let (key, value) =   self.vote._checkpoints(account, U32::from(pos));
       let key:u64 =  key.to();
       let value:U256 = value.to(); 
       (key, value)
    }
}


impl <T: IEip712 + StorageType> Erc721Vote<T> {
    
    /// Returns the maximum possible supply of tokens.
    ///
    /// The maximum possible supply of tokens is the maximum value that can be
    /// represented by the `Uint<208,4>` type, which is `2^208 - 1`.
    ///
    /// # Returns
    ///
    /// The maximum possible supply of tokens as a `U256` value.
     pub fn  _max_supply(&self) -> U256 { 
        U256::from(Uint::<208,4>::MAX)
    }   
}