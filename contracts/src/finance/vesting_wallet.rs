//! A vesting wallet is an ownable contract that can receive native currency and
//! ERC20 tokens, and release these assets to the wallet owner, also referred to
//! as "beneficiary", according to a vesting schedule.
//!
//! Any assets transferred to this contract will follow the vesting schedule as
//! if they were locked from the beginning. Consequently, if the vesting has
//! already started, any amount of tokens sent to this contract will (at least
//! partly) be immediately releasable.
//!
//! By setting the duration to 0, one can configure this contract to behave like
//! an asset timelock that hold tokens for a beneficiary until a specified time.
//!
//! NOTE: Since the wallet is [`crate::access::ownable::Ownable`], and ownership
//! can be transferred, it is possible to sell unvested tokens. Preventing this
//! in a smart contract is difficult, considering that: 1) a beneficiary address
//! could be a counterfactually deployed contract, 2) there is likely to be a
//! migration path for EOAs to become contracts in the near future.
//!
//! NOTE: When using this contract with any token whose balance is adjusted
//! automatically (i.e. a rebase token), make sure to account the supply/balance
//! adjustment in the vesting schedule to ensure the vested amount is as
//! intended.
use alloy_primitives::{Address, U256, U64};
use alloy_sol_types::sol;
use stylus_sdk::{
    block, call,
    call::{transfer_eth, Call},
    contract, evm, function_selector,
    storage::TopLevelStorage,
    stylus_proc::{public, sol_storage, SolidityError},
};

use crate::access::{ownable, ownable::Ownable};

sol! {
    /// Emitted when `amount` of ether has been released.
    #[allow(missing_docs)]
    event EtherReleased(uint256 amount);

    /// Emitted when `amount` of ERC20 `token` has been released.
    #[allow(missing_docs)]
    event ERC20Released(address indexed token, uint256 amount);
}

sol! {
    /// Indicates an error related to the underlying Ether transfer.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ReleaseEtherFailed();

    /// Indicates an error related to the underlying ERC20 transfer.
    ///
    /// * `token` - Address of the token being released.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ReleaseTokenFailed(address token);
}

pub use token::IErc20;
#[allow(missing_docs)]
mod token {
    stylus_sdk::stylus_proc::sol_interface! {
        /// Interface of the ERC-20 token.
        interface IErc20 {
            function balanceOf(address account) external view returns (uint256);
            function transfer(address to, uint256 value) external returns (bool);
        }
    }
}

/// An error that occurred in the [`VestingWallet`] contract.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`Ownable`] contract [`ownable::Error`].
    Ownable(ownable::Error),
    /// Error type from [`call::Call`] contract [`call::Error`].
    StylusError(call::Error),
    /// Indicates an error related to the underlying Ether transfer.
    ReleaseEtherFailed(ReleaseEtherFailed),
    /// Indicates an error related to the underlying ERC20 transfer.
    ReleaseTokenFailed(ReleaseTokenFailed),
}

sol_storage! {
    /// State of a VestingWallet Contract.
    pub struct VestingWallet {
        /// Ownable contract
        Ownable ownable;
        /// Amount of eth already released.
        uint256 _released;
        /// Amount of ERC20 tokens already released.
        mapping(address => uint256) _erc20_released;
        /// Start timestamp.
        uint64 _start;
        /// Vesting duration.
        uint64 _duration;
    }
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for VestingWallet {}

#[public]
impl VestingWallet {
    /// The contract should be able to receive Eth.
    #[payable]
    pub fn receive_ether(&self) {}

    /// Returns the address of the current owner.
    pub fn owner(&self) -> Address {
        self.ownable.owner()
    }

    /// Transfers ownership of the contract to a new account (`new_owner`). Can
    /// only be called by the current owner.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_owner` - The next owner of this contract.
    ///
    /// # Errors
    ///
    /// If `new_owner` is the zero address, then the error
    /// [`OwnableInvalidOwner`] is returned.
    pub fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Error> {
        Ok(self.ownable.transfer_ownership(new_owner)?)
    }

    /// Leaves the contract without owner. It will not be possible to call
    /// [`Self::only_owner`] functions. Can only be called by the current owner.
    ///
    /// NOTE: Renouncing ownership will leave the contract without an owner,
    /// thereby disabling any functionality that is only available to the owner.
    ///
    /// # Errors
    ///
    /// If not called by the owner, then the error
    /// [`Error::UnauthorizedAccount`] is returned.
    pub fn renounce_ownership(&mut self) -> Result<(), Error> {
        Ok(self.ownable.renounce_ownership()?)
    }

    /// Getter for the start timestamp.
    pub fn start(&self) -> U256 {
        U256::from(self._start.get())
    }

    /// Getter for the vesting duration.
    pub fn duration(&self) -> U256 {
        U256::from(self._duration.get())
    }

    /// Getter for the end timestamp.
    pub fn end(&self) -> U256 {
        self.start() + self.duration()
    }

    /// Amount of eth already released
    #[selector(name = "released")]
    pub fn released_eth(&self) -> U256 {
        self._released.get()
    }

    /// Amount of token already released
    #[selector(name = "released")]
    pub fn released_erc20(&self, token: Address) -> U256 {
        self._erc20_released.get(token)
    }

    /// Getter for the amount of releasable eth.
    #[selector(name = "releasable")]
    pub fn releasable_eth(&self) -> U256 {
        self.vested_amount_eth(block::timestamp()) - self.released_eth()
    }

    /// Getter for the amount of releasable `token` tokens. `token` should be
    /// the address of an [`crate::token::erc20::Erc20`] contract.
    #[selector(name = "releasable")]
    pub fn releasable_erc20(&mut self, token: Address) -> U256 {
        self.vested_amount_erc20(token, block::timestamp())
            - self.released_erc20(token)
    }

    /// Release the native token (ether) that have already vested.
    ///
    /// Emits an [`EtherReleased`] event.
    #[selector(name = "release")]
    pub fn release_eth(&mut self) -> Result<(), Error> {
        let amount = self.releasable_eth();
        let released = self
            .released_eth()
            .checked_add(amount)
            .expect("should not exceed `U256::MAX` for `_released`");
        self._released.set(released);

        evm::log(EtherReleased { amount });

        transfer_eth(self.ownable.owner(), amount)
            .map_err(|_| Error::ReleaseEtherFailed(ReleaseEtherFailed {}))
    }

    /// Release the tokens that have already vested.
    ///
    /// Emits an [`ERC20Released`] event.
    #[selector(name = "release")]
    pub fn release_erc20(&mut self, token: Address) -> Result<(), Error> {
        let amount = self.releasable_erc20(token);
        let released = self
            .released_erc20(token)
            .checked_add(amount)
            .expect("should not exceed `U256::MAX` for `_erc20Released`");
        self._erc20_released.setter(token).set(released);

        evm::log(ERC20Released { token, amount });

        let erc20 = IERC20::new(token);
        let owner = self.ownable.owner();
        let call = Call::new_in(self);
        let succeeded = erc20.transfer(call, owner, amount)?;
        if !succeeded {
            return Err(ReleaseTokenFailed { token }.into());
        }

        Ok(())
    }

    /// Calculates the amount of ether that has already vested. Default
    /// implementation is a linear vesting curve.
    #[selector(name = "vestedAmount")]
    pub fn vested_amount_eth(&self, timestamp: u64) -> U256 {
        self._vesting_schedule(
            contract::balance() + self.released_eth(),
            U64::from(timestamp),
        )
    }

    /// Calculates the amount of tokens that has already vested. Default
    /// implementation is a linear vesting curve.
    #[selector(name = "vestedAmount")]
    pub fn vested_amount_erc20(
        &mut self,
        token: Address,
        timestamp: u64,
    ) -> U256 {
        let erc20 = IERC20::new(token);
        let call = Call::new_in(self);
        let balance = erc20
            .balance_of(call, contract::address())
            .expect("should return the balance");

        self._vesting_schedule(
            balance + self.released_erc20(token),
            U64::from(timestamp),
        )
    }
}

impl VestingWallet {
    /// Virtual implementation of the vesting formula. This returns the amount
    /// vested, as a function of time, for an asset given its total
    /// historical allocation.
    pub fn _vesting_schedule(
        &self,
        total_allocation: U256,
        timestamp: U64,
    ) -> U256 {
        if U256::from(timestamp) < self.start() {
            U256::ZERO
        } else if U256::from(timestamp) >= self.end() {
            total_allocation
        } else {
            (total_allocation * (U256::from(timestamp) - self.start()))
                / self.duration()
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, uint, Address, U256, U64};
    use stylus_sdk::block;

    use super::VestingWallet;

    const TOKEN: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    const DURATION: u64 = 4 * 365 * 86400; // 4 years

    fn start() -> U64 {
        U64::from(block::timestamp() + 3600) // 1 hour
    }

    #[motsu::test]
    fn reads_start(contract: VestingWallet) {
        let start = start();
        contract._start.set(start);
        assert_eq!(U256::from(start), contract.start());
    }

    #[motsu::test]
    fn reads_duration(contract: VestingWallet) {
        contract._duration.set(U64::from(DURATION));
        assert_eq!(U256::from(DURATION), contract.duration());
    }

    #[motsu::test]
    fn reads_end(contract: VestingWallet) {
        let start = start();
        let duration = U64::from(DURATION);
        contract._start.set(start);
        contract._duration.set(duration);
        assert_eq!(U256::from(start + duration), contract.end());
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
    fn reads_vested_allocation(contract: VestingWallet) {
        let start = start();
        let duration = U64::from(DURATION);
        contract._start.set(start);
        contract._duration.set(duration);

        let one = uint!(1_U256);
        let two = uint!(2_U256);

        assert_eq!(
            U256::ZERO,
            contract._vesting_schedule(two, start - U64::from(1))
        );
        assert_eq!(
            one,
            contract._vesting_schedule(two, start + duration / U64::from(2))
        );
        assert_eq!(two, contract._vesting_schedule(two, start + duration));
    }
}
