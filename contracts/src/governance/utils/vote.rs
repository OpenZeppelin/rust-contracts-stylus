use alloc::string::String;
use alloc::vec::Vec;

use alloy_primitives::{b256, keccak256, Address, Uint, B256, U256, U32, U64};
use stylus_sdk::{
    block, call::MethodError, evm, msg, prelude::*,
};
use alloy_sol_types::{sol, SolType};

use crate::utils::cryptography::eip712::IEip712;
use crate::utils::structs::checkpoints::{Size, Trace, S208};
use crate::utils::cryptography::ecdsa;
use crate::utils::nonces::Nonces;

type U48 = <S208 as Size>::Key;
type U208 = <S208 as Size>::Value;


// keccak256("Delegation(address delegatee,uint256 nonce,uint256 expiry)")
const DELEGATION_TYPEHASH: B256 =
    b256!("e48329057bfd03d55e49b547132e39cffd9c1820ad7b9d4c5307691425d15adf");

type StructHashTuple = sol! {
    tuple(bytes32, address, uint256, uint256)
};

// Solidity event definitions
sol! {
    #[allow(missing_docs)]
    event DelegateChanged(address indexed delegator, address indexed fromDelegate, address indexed toDelegate);

    #[allow(missing_docs)]
    event DelegateVotesChanged(address indexed delegate, uint256 previousVotes, uint256 newVotes);
}

// Solidity error definitions
sol! {
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
    VoteExpiredSignature(VotesExpiredSignature),
    ERC6372InconsistentClock(ERC6372InconsistentClock),
    ERC5805FutureLookup(ERC5805FutureLookup),
     /// Error type from [`ecdsa`] contract [`ecdsa::Error`].
    ECDSA(ecdsa::Error),
}

impl MethodError for Error {
    fn encode(self) -> Vec<u8> {
        self.into()
    }
}


// Main contract structure
sol_storage! {
    pub struct Vote<T: IEip712 + StorageType > {
        mapping(address => address) _delegatee;
        mapping(address => Trace<S208>) _delegate_checkpoints;
        Trace<S208> _total_checkpoints;

        Nonces nonces;
        T eip712;
    }
}


unsafe impl<T: IEip712 + StorageType > TopLevelStorage for Vote<T> {}

#[public]
impl<T: IEip712 + StorageType> Vote<T> {
    
    /// Returns the current nonce for `owner`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - The address for which to return the nonce.
    #[must_use]
    pub fn nonces(&self, owner: Address) -> U256 {
        self.nonces.nonces(owner)
    }

    /// Returns the current voting power for `account`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - The address for which to return the current voting power.
    ///
    /// # Returns
    ///
    /// * `U256` - The current number of votes for `account`.
    fn get_votes(&self, account: Address) -> U256 {
        U256::from(self._delegate_checkpoints.get(account).latest())
    }

    /// Returns the voting power for `account` at a specific `timepoint`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - The address for which to return the voting power.
    /// * `timepoint` - The specific timepoint to look up the voting power.
    ///
    /// # Returns
    ///
    /// * `Result<U256, Error>` - The voting power at the given timepoint, or an error if the lookup is in the future.
    ///
    /// # Errors
    ///
    /// Returns `Error::ERC5805FutureLookup` if the `timepoint` is in the future.
    fn get_past_votes(&self, account: Address, timepoint: U256) -> Result<U256, Error> {
        let current_timepoint = self.clock();
        if timepoint >= U256::from(current_timepoint) {
            return Err(Error::ERC5805FutureLookup(ERC5805FutureLookup { timepoint, clock: current_timepoint }));
        }
        let key = self._delegate_checkpoints.get(account).upper_lookup_recent(U48::from(current_timepoint));
        Ok(U256::from(key))
    }

    /// Returns the total voting power at a specific `timepoint`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `timepoint` - The specific timepoint to look up the total voting power.
    ///
    /// # Returns
    ///
    /// * `Result<U256, Error>` - The total voting power at the given timepoint, or an error if the lookup is in the future.
    ///
    /// # Errors
    ///
    /// Returns `Error::ERC5805FutureLookup` if the `timepoint` is in the future.
    fn get_past_total_supply(&self, timepoint: U256) -> Result<U256, Error> {
        let current_timepoint = self.clock();
        if timepoint >= U256::from(current_timepoint) {
            return Err(Error::ERC5805FutureLookup(ERC5805FutureLookup { timepoint, clock: current_timepoint }));
        }
        let key = self._total_checkpoints.latest();
        Ok(U256::from(key))
    }

    /// Returns the current delegate for `account`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - The address for which to return the current delegate.
    ///
    /// # Returns
    ///
    /// * `Address` - The current delegate for `account`.
    fn delegates(&self, account: Address) -> Address {
        self._delegatee.get(account)
    }

    /// Sets the delegate for `msg.sender()` to `delegatee`.
    ///
    /// Emits a `DelegateChanged` event.
    fn delegate(&mut self, delegatee: Address) {
        self._delegate(msg::sender(), delegatee);
    }


    /// Allows a user to delegate their voting power to a `delegatee` via a signature.
    ///
    /// This method allows the delegation of voting power using the EIP-712 signature scheme,
    /// enabling a delegatee to be specified by the signer without requiring an on-chain transaction
    /// from the signer themselves.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `delegatee` - The address to which the voting power will be delegated.
    /// * `nonce` - The current nonce of the signer, used to prevent replay attacks.
    /// * `expiry` - The expiration time for the signature, in block timestamp format.
    /// * `v`, `r`, `s` - Components of the ECDSA signature.
    ///
    /// # Returns
    ///
    /// * `Result<(), Error>` - Returns an error if the signature is expired or invalid.
    ///
    /// # Errors
    ///
    /// Returns `Error::VoteExpiredSignature` if the signature is expired.
    /// Returns `Error::ECDSA` if the signature cannot be recovered or is invalid.
    ///
    /// # Requirements
    ///
    /// * The current block timestamp must be less than or equal to `expiry`.
    /// * The signature must be valid and correspond to the signer's current nonce.
    ///
    /// # Events
    ///
    /// Emits a `DelegateChanged` event if the delegation is successful.
    fn delegate_by_sig(&mut self, delegatee: Address, nonce: U256, expiry: U256, v: u8, r: B256, s: B256) -> Result<(), Error> {
        if U256::from(block::timestamp()) > expiry {
            return Err(Error::VoteExpiredSignature(VotesExpiredSignature { expiry }));
        }

        let struct_hash = keccak256(StructHashTuple::abi_encode(
            &(*DELEGATION_TYPEHASH,delegatee, nonce, expiry)
        ));

        let hash: B256 = self.eip712.hash_typed_data_v4(struct_hash);

        let signer: Address = ecdsa::recover(self, hash, v, r, s)?;

        self.nonces.use_checked_nonce(signer, nonce);
        self._delegate(signer, delegatee);
        Ok(())
    }


    /// Returns the current block timestamp.
    fn clock(&self) -> u64 {
        block::timestamp()
    }

    /// Returns the clock mode as a string.
    ///
    /// This function checks if the current block timestamp matches the clock timestamp.
    /// If there is an inconsistency, it returns an `Error::ERC6372InconsistentClock`.
    /// Otherwise, it returns a string indicating the clock mode.
    ///
    /// # Returns
    ///
    /// * `Result<String, Error>` - A string representing the clock mode if consistent,
    ///   or an error if there is a clock inconsistency.
    fn clock_mode(&self) -> Result<String, Error> {
        if U64::from(block::timestamp()) != U64::from(self.clock()) {
            return Err(Error::ERC6372InconsistentClock(ERC6372InconsistentClock {}));
        }
        Ok(String::from("mode=blocknumber&from=default"))
    }
}

impl<T: IEip712 + StorageType> Vote<T> {
    /// Internal function to update the delegate for `account` to `delegatee` and
    /// move the voting power from the old delegate to the new delegate.
    ///
    /// Emits a `DelegateChanged` event.
    ///
    /// It is not safe to call this function directly. Instead, use either
    /// {delegate} or {delegate_by_sig}.
    fn _delegate(&mut self, account: Address, delegatee: Address) {
        let old_delegate = self.delegates(account);
        self._delegatee.setter(account).set(delegatee);
        evm::log(DelegateChanged {
            delegator: account,
            fromDelegate: old_delegate,
            toDelegate: delegatee,
        });
        self._move_delegate_votes(old_delegate, delegatee,Uint::<256,4>::from(0));
    }

    /// Internal function to move `amount` of voting power from `from` to `to`.
    /// If `from` is `Address::ZERO`, `to` is `Address::ZERO`, or `from` is equal
    /// to `to`, nothing happens.
    ///
    /// Emits two `DelegateVotesChanged` events, one for `from` and one for `to`.
    ///
    fn _move_delegate_votes(&mut self, from: Address, to: Address, amount: U256) {
        if from == to || from == Address::ZERO || to == Address::ZERO {
            return;
        }
        let clock = U48::from(self.clock());
        let from_latest = self._delegate_checkpoints.get(from).latest();
        let safe_amount = U208::from(amount);
        let subtracted_amount = safe_amount - from_latest;
        let _ = self._delegate_checkpoints.setter(from).push(clock, subtracted_amount);
        evm::log(DelegateVotesChanged   {
            delegate: from,
            previousVotes: U256::from(from_latest),
            newVotes: U256::from(subtracted_amount),
        });

        let to_latest = self._delegate_checkpoints.get(to).latest();
        let add_amount = safe_amount + from_latest;
        let _ = self._delegate_checkpoints.setter(to).push(clock, add_amount);
        evm::log(DelegateVotesChanged   {
            delegate: to,
            previousVotes: U256::from(to_latest),
            newVotes: U256::from(add_amount),
        });
    }

    /// Returns the number of checkpoints associated with a given account.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - The address of the account for which to retrieve the number of checkpoints.
    ///
    /// # Returns
    ///
    /// * `u32` - The number of checkpoints for the specified account.
    pub fn _num_checkpoints(&self, account: Address) -> u32 {
        self._delegate_checkpoints.get(account).length().to()
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
    pub fn _checkpoints(&self, account: Address, pos: U32) -> (U48, U208) {
        self._delegate_checkpoints.get(account).at(pos)
    }


}



