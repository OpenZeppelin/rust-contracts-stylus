//! A vesting wallet handles the vesting of Ether and ERC-20 tokens for a given
//! beneficiary.
//!
//! A vesting wallet is an ownable contract that can receive native currency and
//! [`crate::token::erc20::Erc20`] tokens, and release these assets to the
//! wallet owner, also referred to as "beneficiary", according to a vesting
//! schedule.
//!
//! Any assets transferred to this contract will follow the vesting schedule as
//! if they were locked from the beginning. Consequently, if the vesting has
//! already started, any amount of tokens sent to this contract will (at least
//! partly) be immediately releasable.
//!
//! By setting the duration to 0, one can configure this contract to behave like
//! an asset timelock that hold tokens for a beneficiary until a specified time.
//!
//! NOTE: Since the wallet is [`Ownable`], and ownership
//! can be transferred, it is possible to sell unvested tokens. Preventing this
//! in a smart contract is difficult, considering that: 1) a beneficiary address
//! could be a counterfactually deployed contract, 2) there is likely to be a
//! migration path for EOAs to become contracts in the near future.
//!
//! NOTE: When using this contract with any token whose balance is adjusted
//! automatically (i.e. a rebase token), make sure to account the supply/balance
//! adjustment in the vesting schedule to ensure the vested amount is as
//! intended.

use alloc::vec::Vec;

use alloy_primitives::{Address, U256, U64};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    block,
    call::{self, call, Call},
    contract, evm, function_selector,
    prelude::storage,
    storage::{StorageMap, StorageU256, StorageU64, TopLevelStorage},
    stylus_proc::{public, SolidityError},
};

use crate::{
    access::ownable::{self, IOwnable, Ownable},
    token::erc20::utils::{
        safe_erc20::{self, ISafeErc20, SafeErc20},
        IErc20,
    },
    utils::math::storage::AddAssignChecked,
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when `amount` of Ether has been released.
        ///
        /// * `amount` - Total Ether released.
        #[allow(missing_docs)]
        event EtherReleased(uint256 amount);

        /// Emitted when `amount` of ERC-20 `token` has been released.
        ///
        /// * `token` - Address of the token being released.
        /// * `amount` - Number of tokens released.
        #[allow(missing_docs)]
        event ERC20Released(address indexed token, uint256 amount);
    }

    sol! {
        /// Indicates an error related to the underlying Ether transfer.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ReleaseEtherFailed();

        /// The token address is not valid (eg. `Address::ZERO`).
        ///
        /// * `token` - Address of the token being released.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error InvalidToken(address token);
    }
}

/// An error that occurred in the [`VestingWallet`] contract.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`Ownable`] contract [`ownable::Error`].
    Ownable(ownable::Error),
    /// Indicates an error related to the underlying Ether transfer.
    ReleaseEtherFailed(call::Error),
    /// Error type from [`SafeErc20`] contract [`safe_erc20::Error`].
    SafeErc20(safe_erc20::Error),
    /// The token address is not valid. (eg. `Address::ZERO`).
    InvalidToken(InvalidToken),
}

/// State of a [`VestingWallet`] Contract.
#[storage]
pub struct VestingWallet {
    /// [`Ownable`] contract.
    pub ownable: Ownable,
    /// Amount of Ether already released.
    #[allow(clippy::used_underscore_binding)]
    pub _released: StorageU256,
    /// Amount of ERC-20 tokens already released.
    #[allow(clippy::used_underscore_binding)]
    pub _erc20_released: StorageMap<Address, StorageU256>,
    /// Start timestamp.
    #[allow(clippy::used_underscore_binding)]
    pub _start: StorageU64,
    /// Vesting duration.
    #[allow(clippy::used_underscore_binding)]
    pub _duration: StorageU64,
    /// [`SafeErc20`] contract.
    pub safe_erc20: SafeErc20,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for VestingWallet {}

/// Required interface of a [`VestingWallet`] compliant contract.
#[interface_id]
pub trait IVestingWallet {
    /// The error type associated to this trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the address of the current owner.
    ///
    /// Re-export of [`Ownable::owner`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn owner(&self) -> Address;

    /// Transfers ownership of the contract to a new account (`new_owner`). Can
    /// only be called by the current owner.
    ///
    /// Re-export of [`Ownable::transfer_ownership`].
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_owner` - The next owner of this contract.
    ///
    /// # Errors
    ///
    /// * [`ownable::Error::UnauthorizedAccount`] - If called by any account
    ///   other than the owner.
    /// * [`ownable::Error::InvalidOwner`] - If `new_owner` is the
    ///   `Address::ZERO`.
    ///
    /// # Events
    ///
    /// * [`ownable::OwnershipTransferred`].
    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Self::Error>;

    /// Leaves the contract without owner. It will not be possible to call
    /// [`Ownable::only_owner`] functions. Can only be called by the current
    /// owner.
    ///
    /// Re-export of [`Ownable::renounce_ownership`].
    ///
    /// NOTE: Renouncing ownership will leave the contract without an owner,
    /// thereby disabling any functionality that is only available to the owner.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`ownable::Error::UnauthorizedAccount`] - If not called by the owner.
    ///
    /// # Events
    ///
    /// * [`ownable::OwnershipTransferred`].
    fn renounce_ownership(&mut self) -> Result<(), Self::Error>;

    /// The contract should be able to receive Ether.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn receive_ether(&self);

    /// Getter for the start timestamp.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn start(&self) -> U256;

    /// Getter for the vesting duration.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn duration(&self) -> U256;

    /// Getter for the end timestamp.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn end(&self) -> U256;

    /// Amount of Ether already released.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[selector(name = "released")]
    fn released_eth(&self) -> U256;

    /// Amount of token already released.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token` - Address of the token being released.
    #[selector(name = "released")]
    fn released_erc20(&self, token: Address) -> U256;

    /// Getter for the amount of releasable Ether.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Panics
    ///
    /// * If total allocation exceeds `U256::MAX`.
    /// * If scaled, total allocation (mid calculation) exceeds `U256::MAX`.
    #[selector(name = "releasable")]
    fn releasable_eth(&self) -> U256;

    /// Getter for the amount of releasable `token` tokens. `token` should be
    /// the address of an ERC-20 contract.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the releasable token.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidToken`] - If the `token` address is not a contract.
    ///
    /// # Panics
    ///
    /// * If total allocation exceeds `U256::MAX`.
    /// * If scaled, total allocation (mid calculation) exceeds `U256::MAX`.
    #[selector(name = "releasable")]
    fn releasable_erc20(&mut self, token: Address)
        -> Result<U256, Self::Error>;

    /// Release the native tokens (Ether) that have already vested.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::ReleaseEtherFailed`] - If Ether transfer fails.
    ///
    /// # Events
    ///
    /// * [`EtherReleased`].
    ///
    /// # Panics
    ///
    /// * If total allocation exceeds `U256::MAX`.
    /// * If scaled total allocation (mid calculation) exceeds `U256::MAX`.
    #[selector(name = "release")]
    fn release_eth(&mut self) -> Result<(), Self::Error>;

    /// Release the tokens that have already vested.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the token being released.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidToken`] -  If the `token` address is not a contract.
    /// * [`safe_erc20::Error::SafeErc20FailedOperation`] - If the contract
    ///   fails to execute the call.
    ///
    /// # Events
    ///
    /// * [`ERC20Released`].
    ///
    /// # Panics
    ///
    /// * If total allocation exceeds `U256::MAX`.
    /// * If scaled, total allocation (mid calculation) exceeds `U256::MAX`.
    #[selector(name = "release")]
    fn release_erc20(&mut self, token: Address) -> Result<(), Self::Error>;

    /// Calculates the amount of Ether that has already vested.
    /// The Default implementation is a linear vesting curve.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `timestamp` - Point in time for which to check the vested amount.
    ///
    /// # Panics
    ///
    /// * If total allocation exceeds `U256::MAX`.
    /// * If scaled, total allocation (mid calculation) exceeds `U256::MAX`.
    #[selector(name = "vestedAmount")]
    fn vested_amount_eth(&self, timestamp: u64) -> U256;

    /// Calculates the amount of tokens that has already vested.
    /// The Default implementation is a linear vesting curve.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the token being released.
    /// * `timestamp` - Point in time for which to check the vested amount.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidToken`] - If the `token` address is not a contract.
    ///
    /// # Panics
    ///
    /// * If total allocation exceeds `U256::MAX`.
    /// * If scaled, total allocation (mid calculation) exceeds `U256::MAX`.
    #[selector(name = "vestedAmount")]
    fn vested_amount_erc20(
        &mut self,
        token: Address,
        timestamp: u64,
    ) -> Result<U256, Self::Error>;
}

#[public]
impl IVestingWallet for VestingWallet {
    type Error = Error;

    fn owner(&self) -> Address {
        self.ownable.owner()
    }

    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Self::Error> {
        Ok(self.ownable.transfer_ownership(new_owner)?)
    }

    fn renounce_ownership(&mut self) -> Result<(), Self::Error> {
        Ok(self.ownable.renounce_ownership()?)
    }

    #[payable]
    fn receive_ether(&self) {}

    fn start(&self) -> U256 {
        U256::from(self._start.get())
    }

    fn duration(&self) -> U256 {
        U256::from(self._duration.get())
    }

    fn end(&self) -> U256 {
        // SAFETY: both `start` and `duration` are stored as u64,
        // so they cannot exceed `U256::MAX`
        self.start() + self.duration()
    }

    #[selector(name = "released")]
    fn released_eth(&self) -> U256 {
        self._released.get()
    }

    #[selector(name = "released")]
    fn released_erc20(&self, token: Address) -> U256 {
        self._erc20_released.get(token)
    }

    #[selector(name = "releasable")]
    fn releasable_eth(&self) -> U256 {
        // SAFETY: total vested amount is by definition greater than or equal to
        // the released amount.
        self.vested_amount_eth(block::timestamp()) - self.released_eth()
    }

    #[selector(name = "releasable")]
    fn releasable_erc20(
        &mut self,
        token: Address,
    ) -> Result<U256, Self::Error> {
        let vested = self.vested_amount_erc20(token, block::timestamp())?;
        // SAFETY: total vested amount is by definition greater than or equal to
        // the released amount.
        Ok(vested - self.released_erc20(token))
    }

    #[selector(name = "release")]
    fn release_eth(&mut self) -> Result<(), Self::Error> {
        let amount = self.releasable_eth();

        self._released.add_assign_checked(
            amount,
            "total released should not exceed `U256::MAX`",
        );

        let owner = self.ownable.owner();

        call(Call::new_in(self).value(amount), owner, &[])?;

        evm::log(EtherReleased { amount });

        Ok(())
    }

    #[selector(name = "release")]
    fn release_erc20(&mut self, token: Address) -> Result<(), Self::Error> {
        let amount = self.releasable_erc20(token)?;
        let owner = self.ownable.owner();

        self._erc20_released.setter(token).add_assign_checked(
            amount,
            "total released should not exceed `U256::MAX`",
        );

        self.safe_erc20.safe_transfer(token, owner, amount)?;

        evm::log(ERC20Released { token, amount });

        Ok(())
    }

    #[selector(name = "vestedAmount")]
    fn vested_amount_eth(&self, timestamp: u64) -> U256 {
        let total_allocation = contract::balance()
            .checked_add(self.released_eth())
            .expect("total allocation should not exceed `U256::MAX`");

        self.vesting_schedule(total_allocation, U64::from(timestamp))
    }

    #[selector(name = "vestedAmount")]
    fn vested_amount_erc20(
        &mut self,
        token: Address,
        timestamp: u64,
    ) -> Result<U256, Self::Error> {
        let erc20 = IErc20::new(token);
        let balance = erc20
            .balance_of(Call::new_in(self), contract::address())
            .map_err(|_| InvalidToken { token })?;

        let total_allocation = balance
            .checked_add(self.released_erc20(token))
            .expect("total allocation should not exceed `U256::MAX`");

        Ok(self.vesting_schedule(total_allocation, U64::from(timestamp)))
    }
}

impl VestingWallet {
    /// Virtual implementation of the vesting formula. This returns the amount
    /// vested, as a function of time, for an asset given its total
    /// historical allocation.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `total_allocation` - Total vested amount.
    /// * `timestamp` - Point in time for which to calculate the vested amount.
    ///
    /// # Panics
    ///
    /// * If scaled, total allocation (mid calculation) exceeds `U256::MAX`.
    fn vesting_schedule(&self, total_allocation: U256, timestamp: U64) -> U256 {
        let timestamp = U256::from(timestamp);

        if timestamp < self.start() {
            U256::ZERO
        } else if timestamp >= self.end() {
            total_allocation
        } else {
            // SAFETY: `timestamp` is guaranteed to be greater than
            // `self.start()` as checked by earlier bounds.
            let elapsed = timestamp - self.start();

            let scaled_allocation = total_allocation
                .checked_mul(elapsed)
                .expect("scaled allocation exceeds `U256::MAX`");

            // SAFETY: `self.duration()` is non-zero. If `self.duration()` were
            // zero, then `end == start`, meaning that `timestamp >= self.end()`
            // and the function would have returned earlier.
            scaled_allocation / self.duration()
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, uint, Address, U256, U64};
    use stylus_sdk::block;

    use super::{IVestingWallet, VestingWallet};

    const TOKEN: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    const DURATION: u64 = 4 * 365 * 86400; // 4 years

    fn start() -> u64 {
        block::timestamp() + 3600 // 1 hour
    }

    fn init(
        contract: &mut VestingWallet,
        start: u64,
        duration: u64,
    ) -> (U64, U64) {
        let start = U64::from(start);
        let duration = U64::from(duration);
        contract._start.set(start);
        contract._duration.set(duration);
        (start, duration)
    }

    #[motsu::test]
    fn reads_start(contract: VestingWallet) {
        let (start, _) = init(contract, start(), 0);
        assert_eq!(U256::from(start), contract.start());
    }

    #[motsu::test]
    fn reads_duration(contract: VestingWallet) {
        let (_, duration) = init(contract, 0, DURATION);
        assert_eq!(U256::from(duration), contract.duration());
    }

    #[motsu::test]
    fn reads_end(contract: VestingWallet) {
        let (start, duration) = init(contract, start(), DURATION);
        assert_eq!(U256::from(start + duration), contract.end());
    }

    #[motsu::test]
    fn reads_max_end(contract: VestingWallet) {
        init(contract, u64::MAX, u64::MAX);
        assert_eq!(U256::from(U64::MAX) + U256::from(U64::MAX), contract.end());
    }

    #[motsu::test]
    fn reads_released_eth(contract: VestingWallet) {
        let one = uint!(1_U256);
        contract._released.set(one);
        assert_eq!(one, contract.released_eth());
    }

    #[motsu::test]
    fn reads_released_erc20(contract: VestingWallet) {
        let one = uint!(1_U256);
        contract._erc20_released.setter(TOKEN).set(one);
        assert_eq!(one, contract.released_erc20(TOKEN));
    }

    #[motsu::test]
    fn gets_vesting_schedule(contract: VestingWallet) {
        let (start, duration) = init(contract, start(), DURATION);

        let one = uint!(1_U256);
        let two = uint!(2_U256);

        assert_eq!(
            U256::ZERO,
            contract.vesting_schedule(two, start - U64::from(1))
        );
        assert_eq!(
            one,
            contract.vesting_schedule(two, start + duration / U64::from(2))
        );
        assert_eq!(two, contract.vesting_schedule(two, start + duration));
        assert_eq!(
            two,
            contract.vesting_schedule(two, start + duration + U64::from(1))
        );
    }

    #[motsu::test]
    fn gets_vesting_schedule_zero_duration(contract: VestingWallet) {
        let (start, _) = init(contract, start(), 0);

        let two = uint!(2_U256);

        assert_eq!(
            U256::ZERO,
            contract.vesting_schedule(two, start - U64::from(1))
        );
        assert_eq!(two, contract.vesting_schedule(two, start));
        assert_eq!(two, contract.vesting_schedule(two, start + U64::from(1)));
    }
}
