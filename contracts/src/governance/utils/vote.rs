use alloc::string::String;


use alloy_primitives::{Address, FixedBytes, U64, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    alloy_sol_types::sol, block, call:: MethodError, evm, msg, prelude::*
};

use crate::utils::structs::checkpoints::{Size, Trace, S208};

type U48 = <S208 as Size>::Key;

sol! {
    /// Emitted when a delegator changes their delegate, or when tokens are moved from one delegate (`fromDelegate`) to
    /// another (`toDelegate`).
    #[allow(missing_docs)]
    event DelegateChanged(address indexed delegator, address indexed fromDelegate, address indexed toDelegate);
    
    /// Emitted when a token transfer or delegate change results in changes to a delegate's number of voting units.
    /// from (`previousVotes`) to (`newVotes`).
    #[allow(missing_docs)]
    event DelegateVotesChanged(address indexed delegate, uint256 previousVotes, uint256 newVotes);

}

sol! {
    /// Indicates an invaild signature
    #[derive(Debug)]
    #[allow(missing_docs)]
    error VotesExpiredSignature(uint256 expiry);
    
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC6372InconsistentClock();

    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC5805FutureLookup(uint256 timepoint, uint48 clock);
}

#[derive(SolidityError, Debug)]
pub enum Error {
    ExpiredSignature(VotesExpiredSignature),
    ERC6372InconsistentClock(ERC6372InconsistentClock),
    ERC5805FutureLookup(ERC5805FutureLookup)
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

pub trait IERC6372 {
     type Error: Into<alloc::vec::Vec<u8>>;

     fn clock(&self) -> u64;

     fn clock_mode(self) -> Result<String,Error>;

}


/// Required interface of an [`Erc20`] compliant contract.
#[interface_id]
pub trait IVote {
    /// The error type associated to this IVote trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    fn get_votes(&self, account:Address) -> U256;

    fn get_past_votes(&self, account: Address, timepoint:U256) -> Result<U256, Error>;

    fn  get_past_total_supply(&self,timepoint:U256) -> Result<U256, Error>;

    fn delegates(&self,account:Address) -> Address;

    fn delegate(&mut self , delegatee: Address);

    fn delegate_by_sig(&mut self, delegatee:Address,nonce:U256, expiry:U256, v:u8,  r:FixedBytes<32>, s:FixedBytes<32>);
}

sol_storage! {
    /// State of an [`Vote`] contract.
    pub struct Vote {
        /// Maps tokens to owners.
        mapping(address  => address) _delegatee;
        /// Maps users to balances.
        mapping(address => Trace<S208>) _delegate_checkpoints;
        /// Maps tokens to approvals.
        Trace<S208> _total_checkpoints;
    }
}


impl IVote for  Vote {

    type Error = Error;

    fn get_votes(&self,account:Address) -> U256 {
        U256::from(self._delegate_checkpoints.get(account).latest())
    }

    fn get_past_votes(&self,account:Address,timepoint:U256) -> Result<U256, Self::Error> {
        let current_timepoint = self.clock();
        if  timepoint >= U256::from(current_timepoint) {
            return  Err(Error::ERC5805FutureLookup(ERC5805FutureLookup { timepoint, clock: current_timepoint}));
        }
        let key = self._delegate_checkpoints.get(account).upper_lookup_recent(U48::from(current_timepoint));
        Ok(U256::from(key))
    }

    fn  get_past_total_supply(&self,timepoint:U256) -> Result<U256, Self::Error> {
        let current_timepoint = self.clock();
        if  timepoint >= U256::from(current_timepoint) {
            return  Err(Error::ERC5805FutureLookup(ERC5805FutureLookup { timepoint, clock: current_timepoint}));
        }
        let key = self._total_checkpoints.latest();
        Ok(U256::from(key))
    }


    fn delegates(&self,account:Address) -> Address {
        self._delegatee.get(account)
    }

    fn delegate(&mut self,delegatee:Address) {
        //self._delegatee.get(msg::sender())
    }

    fn delegate_by_sig(&mut self,delegatee:Address,nonce:U256,expiry:U256,v:u8,r:FixedBytes<32> ,s:FixedBytes<32>) {
        todo!()
    }
}


impl IERC6372 for Vote {
    type Error = Error;

    fn clock (&self) -> u64 {
      block::timestamp() 
    }
    
    fn clock_mode(self) -> Result<String,self::Error> {
        if U64::from(block::timestamp()) != U64::from(self.clock()) {
            return  Err(Error::ERC6372InconsistentClock(ERC6372InconsistentClock { }));
        }
        Ok(String::from("mode=blocknumber&from=defaul"))
    }
}


impl Vote {
    fn _delegate(&mut self,account:Address, delegatee:Address) {
        let old_delegate = self.delegates(account);
        self._delegatee.setter(account).set(delegatee);
        evm::log(DelegateChanged {
            delegator:account,
            fromDelegate:old_delegate,
            toDelegate:delegatee
        });
        self._move_delegate_votes(from, to, amount);
    }

    fn _move_delegate_votes(&mut self, from:Address,to:Address, amount:U256){

    }

    fn _push()
}