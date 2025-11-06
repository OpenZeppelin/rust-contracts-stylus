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

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use alloy_primitives::{aliases::B32, Address, U256, U64};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    block,
    call::{call, Call, MethodError},
    contract, evm,
    prelude::*,
    storage::{StorageMap, StorageU256, StorageU64},
};

use crate::{
    access::ownable::{self, Ownable},
    token::erc20::{
        abi::Erc20Interface,
        utils::{safe_erc20, ISafeErc20, SafeErc20},
    },
    utils::{introspection::erc165::IErc165, math::storage::AddAssignChecked},
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when `amount` of Ether has been released.
        ///
        /// * `amount` - Total Ether released.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event EtherReleased(uint256 amount);

        /// Emitted when `amount` of ERC-20 `token` has been released.
        ///
        /// * `token` - Address of the token being released.
        /// * `amount` - Number of tokens released.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event ERC20Released(address indexed token, uint256 amount);
    }

    sol! {
        /// Indicates an error related to the underlying Ether transfer.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ReleaseEtherFailed(string reason);

        /// Indicates that a low-level call failed.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error FailedCall();

        /// The token address is not valid (eg. [`Address::ZERO`]).
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
    /// The caller account is not authorized to perform an operation.
    UnauthorizedAccount(ownable::OwnableUnauthorizedAccount),
    /// The owner is not a valid owner account. (eg. [`Address::ZERO`]).
    InvalidOwner(ownable::OwnableInvalidOwner),
    /// Indicates an error related to the underlying Ether transfer.
    ReleaseEtherFailed(ReleaseEtherFailed),
    /// Indicates that a low-level call failed.
    FailedCall(FailedCall),
    /// An operation with an ERC-20 token failed.
    SafeErc20FailedOperation(safe_erc20::SafeErc20FailedOperation),
    /// Indicates a failed [`ISafeErc20::safe_decrease_allowance`] request.
    SafeErc20FailedDecreaseAllowance(
        safe_erc20::SafeErc20FailedDecreaseAllowance,
    ),
    /// The token address is not valid. (eg. [`Address::ZERO`]).
    InvalidToken(InvalidToken),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl From<ownable::Error> for Error {
    fn from(value: ownable::Error) -> Self {
        match value {
            ownable::Error::UnauthorizedAccount(e) => {
                Error::UnauthorizedAccount(e)
            }
            ownable::Error::InvalidOwner(e) => Error::InvalidOwner(e),
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl From<stylus_sdk::call::Error> for Error {
    fn from(value: stylus_sdk::call::Error) -> Self {
        match value {
            stylus_sdk::call::Error::AbiDecodingFailed(_) => {
                Error::FailedCall(FailedCall {})
            }
            stylus_sdk::call::Error::Revert(reason) => {
                Error::ReleaseEtherFailed(ReleaseEtherFailed {
                    reason: String::from_utf8_lossy(&reason).to_string(),
                })
            }
        }
    }
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
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of a [`VestingWallet`] Contract.
#[storage]
pub struct VestingWallet {
    /// [`Ownable`] contract.
    /// We leave the parent [`Ownable`] contract instance public, so that
    /// inheriting contract has access to its internal functions.
    pub ownable: Ownable,
    /// Amount of Ether already released.
    pub(crate) released: StorageU256,
    /// Amount of ERC-20 tokens already released.
    pub(crate) erc20_released: StorageMap<Address, StorageU256>,
    /// Start timestamp.
    pub(crate) start: StorageU64,
    /// Vesting duration.
    pub(crate) duration: StorageU64,
    /// [`SafeErc20`] contract.
    safe_erc20: SafeErc20,
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
    ///   [`Address::ZERO`].
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
    /// * If total allocation exceeds [`U256::MAX`].
    /// * If scaled, total allocation (mid calculation) exceeds [`U256::MAX`].
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
    /// * If total allocation exceeds [`U256::MAX`].
    /// * If scaled, total allocation (mid calculation) exceeds [`U256::MAX`].
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
    /// * If total allocation exceeds [`U256::MAX`].
    /// * If scaled total allocation (mid calculation) exceeds [`U256::MAX`].
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
    /// * If total allocation exceeds [`U256::MAX`].
    /// * If scaled, total allocation (mid calculation) exceeds [`U256::MAX`].
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
    /// * If total allocation exceeds [`U256::MAX`].
    /// * If scaled, total allocation (mid calculation) exceeds [`U256::MAX`].
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
    /// * If total allocation exceeds [`U256::MAX`].
    /// * If scaled, total allocation (mid calculation) exceeds [`U256::MAX`].
    #[selector(name = "vestedAmount")]
    fn vested_amount_erc20(
        &mut self,
        token: Address,
        timestamp: u64,
    ) -> Result<U256, Self::Error>;
}

#[public]
#[implements(IVestingWallet<Error = Error>, IErc165)]
impl VestingWallet {
    /// Constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `beneficiary` - The wallet owner.
    /// * `start_timestamp` - The point in time when token vesting starts.
    /// * `duration_seconds` - The vesting duration in seconds.
    ///
    /// # Errors
    ///
    /// * [`ownable::Error::InvalidOwner`] - If beneficiary is
    ///   [`Address::ZERO`].
    #[constructor]
    pub fn constructor(
        &mut self,
        beneficiary: Address,
        start_timestamp: U64,
        duration_seconds: U64,
    ) -> Result<(), Error> {
        self.ownable.constructor(beneficiary)?;
        self.start.set(start_timestamp);
        self.duration.set(duration_seconds);
        Ok(())
    }

    /// The contract should be able to receive Eth.
    ///
    /// # Errors
    ///
    /// * If the transaction includes data (non-zero calldata).
    /// * If the contract doesn't have enough gas to execute the function.
    #[receive]
    pub fn receive(&mut self) -> Result<(), Vec<u8>> {
        Ok(())
    }
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

    fn start(&self) -> U256 {
        U256::from(self.start.get())
    }

    fn duration(&self) -> U256 {
        U256::from(self.duration.get())
    }

    fn end(&self) -> U256 {
        // SAFETY: both `start` and `duration` are stored as [`U64`], so they
        // cannot exceed [`U256::MAX`].
        self.start() + self.duration()
    }

    #[selector(name = "released")]
    fn released_eth(&self) -> U256 {
        self.released.get()
    }

    #[selector(name = "released")]
    fn released_erc20(&self, token: Address) -> U256 {
        self.erc20_released.get(token)
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

        self.released.add_assign_checked(
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

        self.erc20_released.setter(token).add_assign_checked(
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
        let erc20 = Erc20Interface::new(token);
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
    /// * If scaled, total allocation (mid calculation) exceeds [`U256::MAX`].
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

#[public]
impl IErc165 for VestingWallet {
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IVestingWallet>::interface_id() == interface_id
            || self.ownable.supports_interface(interface_id)
            || <Self as IErc165>::interface_id() == interface_id
    }
}

#[cfg(test)]
mod tests {
    use motsu::prelude::*;
    use stylus_sdk::{
        alloy_primitives::{uint, Address, U256, U64},
        block,
    };

    use super::*;
    use crate::token::erc20::{Erc20, IErc20};

    const BALANCE: U256 = uint!(1000_U256);

    const DURATION: U64 = uint!(126144000_U64); // 4 years

    fn start() -> U64 {
        U64::from(block::timestamp() + 3600) // 1 hour
    }

    #[motsu::test]
    fn reads_start(contract: Contract<VestingWallet>, alice: Address) {
        let start = start();
        contract
            .sender(alice)
            .constructor(alice, start, DURATION)
            .motsu_expect("should construct");
        assert_eq!(U256::from(start), contract.sender(alice).start());
    }

    #[motsu::test]
    fn reads_duration(contract: Contract<VestingWallet>, alice: Address) {
        contract
            .sender(alice)
            .constructor(alice, U64::ZERO, DURATION)
            .motsu_expect("should construct");
        assert_eq!(U256::from(DURATION), contract.sender(alice).duration());
    }

    #[motsu::test]
    fn reads_end(contract: Contract<VestingWallet>, alice: Address) {
        contract
            .sender(alice)
            .constructor(alice, start(), DURATION)
            .motsu_expect("should construct");

        assert_eq!(
            U256::from(start()) + U256::from(DURATION),
            contract.sender(alice).end()
        );
    }

    #[motsu::test]
    fn reads_max_end(contract: Contract<VestingWallet>, alice: Address) {
        contract
            .sender(alice)
            .constructor(alice, U64::MAX, U64::MAX)
            .motsu_expect("should construct");
        assert_eq!(
            U256::from(U64::MAX) + U256::from(U64::MAX),
            contract.sender(alice).end()
        );
    }

    #[motsu::test]
    fn gets_vesting_schedule(
        contract: Contract<VestingWallet>,
        alice: Address,
    ) {
        let start = start();
        let duration = DURATION;

        contract
            .sender(alice)
            .constructor(alice, start, duration)
            .motsu_expect("should construct");

        let one = U256::ONE;

        let two = uint!(2_U256);

        assert_eq!(
            U256::ZERO,
            contract.sender(alice).vesting_schedule(two, start - U64::ONE)
        );
        assert_eq!(
            one,
            contract
                .sender(alice)
                .vesting_schedule(two, start + duration / uint!(2_U64))
        );
        assert_eq!(
            two,
            contract.sender(alice).vesting_schedule(two, start + duration)
        );
        assert_eq!(
            two,
            contract
                .sender(alice)
                .vesting_schedule(two, start + duration + U64::ONE)
        );
    }

    #[motsu::test]
    fn gets_vesting_schedule_zero_duration(
        contract: Contract<VestingWallet>,
        alice: Address,
    ) {
        let start = start();

        contract
            .sender(alice)
            .constructor(alice, start, U64::ZERO)
            .motsu_expect("should construct");

        let two = uint!(2_U256);

        assert_eq!(
            U256::ZERO,
            contract.sender(alice).vesting_schedule(two, start - U64::ONE)
        );
        assert_eq!(two, contract.sender(alice).vesting_schedule(two, start));
        assert_eq!(
            two,
            contract.sender(alice).vesting_schedule(two, start + U64::ONE)
        );
    }

    #[motsu::test]
    fn check_vested_amount_erc20(
        vesting_wallet: Contract<VestingWallet>,
        erc20: Contract<Erc20>,
        alice: Address,
    ) {
        vesting_wallet
            .sender(alice)
            .constructor(alice, start(), DURATION)
            .motsu_expect("should construct");
        erc20
            .sender(alice)
            ._mint(vesting_wallet.address(), U256::from(BALANCE))
            .motsu_unwrap();

        let start = start();
        for i in 0..64_u64 {
            let timestamp: u64 =
                i * DURATION.to::<u64>() / 60 + start.to::<u64>();
            let expected_amount = U256::from(std::cmp::min(
                BALANCE,
                BALANCE * (U256::from(timestamp) - U256::from(start))
                    / U256::from(DURATION),
            ));

            let vested_amount = vesting_wallet
                .sender(alice)
                .vested_amount_erc20(erc20.address(), timestamp)
                .motsu_unwrap();

            assert_eq!(
                expected_amount, vested_amount,
                "\n---\ni: {i}\nstart: {start}\ntimestamp: {timestamp}\n---\n"
            );
        }
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <VestingWallet as IVestingWallet>::interface_id();
        let expected: B32 = 0x23a2649d_u32.into();
        assert_ne!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(contract: Contract<VestingWallet>, alice: Address) {
        assert!(contract.sender(alice).supports_interface(
            <VestingWallet as IVestingWallet>::interface_id()
        ));
        assert!(contract
            .sender(alice)
            .supports_interface(<VestingWallet as IErc165>::interface_id()));

        let fake_interface_id: B32 = 0x12345678_u32.into();
        assert!(!contract.sender(alice).supports_interface(fake_interface_id));
    }

    #[motsu::test]
    fn released_initially_zero(
        vesting_wallet: Contract<VestingWallet>,
        erc20: Contract<Erc20>,
        alice: Address,
    ) {
        // No constructor call, no state changes.
        assert_eq!(U256::ZERO, vesting_wallet.sender(alice).released_eth());
        assert_eq!(
            U256::ZERO,
            vesting_wallet.sender(alice).released_erc20(erc20.address())
        );
    }

    #[motsu::test]
    fn releasable_erc20_reverts_on_invalid_token(
        contract: Contract<VestingWallet>,
        invalid_token: Contract<InvalidTokenMock>,
        alice: Address,
    ) {
        contract
            .sender(alice)
            .constructor(alice, U64::ZERO, DURATION)
            .motsu_expect("should construct");
        let err = contract
            .sender(alice)
            .releasable_erc20(invalid_token.address())
            .motsu_expect_err("should revert");
        assert!(matches!(
            err,
            Error::InvalidToken(InvalidToken {
                token
            }) if token == invalid_token.address()
        ));
    }

    #[storage]
    struct InvalidTokenMock;

    unsafe impl TopLevelStorage for InvalidTokenMock {}

    #[public]
    impl InvalidTokenMock {}

    #[motsu::test]
    fn vested_amount_erc20_reverts_on_invalid_token(
        contract: Contract<VestingWallet>,
        invalid_token: Contract<InvalidTokenMock>,
        alice: Address,
    ) {
        contract
            .sender(alice)
            .constructor(alice, U64::ZERO, DURATION)
            .motsu_expect("should construct");
        let err = contract
            .sender(alice)
            .vested_amount_erc20(invalid_token.address(), 0)
            .motsu_expect_err("should revert");
        assert!(matches!(
            err,
            Error::InvalidToken(InvalidToken {
                token
            }) if token == invalid_token.address()
        ));
    }

    #[motsu::test]
    fn release_erc20_transfers_all_and_emits_event(
        vesting_wallet: Contract<VestingWallet>,
        erc20: Contract<Erc20>,
        alice: Address,
    ) {
        // Set owner and configure vesting to release all immediately.
        vesting_wallet
            .sender(alice)
            .constructor(alice, U64::ZERO, U64::ZERO)
            .motsu_expect("should construct");

        // Mint tokens to the vesting wallet.
        erc20
            .sender(alice)
            ._mint(vesting_wallet.address(), U256::from(BALANCE))
            .motsu_expect("should mint");

        // Release ERC20 to owner (alice).
        vesting_wallet
            .sender(alice)
            .release_erc20(erc20.address())
            .motsu_expect("should release");

        // Owner received full balance.
        assert_eq!(U256::from(BALANCE), erc20.sender(alice).balance_of(alice));
        // Contract holds no remaining tokens.
        assert_eq!(
            U256::ZERO,
            erc20.sender(alice).balance_of(vesting_wallet.address())
        );
        // Released mapping increased and event emitted.
        assert_eq!(
            U256::from(BALANCE),
            vesting_wallet.sender(alice).released_erc20(erc20.address())
        );
        vesting_wallet.assert_emitted(&ERC20Released {
            token: erc20.address(),
            amount: U256::from(BALANCE),
        });
    }

    #[motsu::test]
    #[should_panic = "scaled allocation exceeds `U256::MAX`"]
    fn vesting_schedule_overflow_panics(
        contract: Contract<VestingWallet>,
        alice: Address,
    ) {
        // Configure a non-zero duration so we take the linear branch.
        let start = start();
        let duration = uint!(10_U64);

        contract
            .sender(alice)
            .constructor(alice, start, duration)
            .motsu_expect("should construct");

        // Choose timestamp strictly between start and end so we hit the
        // multiplication path: scaled_allocation = total * elapsed.
        let mid = start + uint!(2_U64); // elapsed >= 2

        // This should overflow: U256::MAX * elapsed > U256::MAX
        contract.sender(alice).vesting_schedule(U256::MAX, mid);
    }

    #[motsu::test]
    fn receive_accepts_eth_and_increases_balance(
        contract: Contract<VestingWallet>,
        alice: Address,
    ) {
        // Construct and send ETH to the contract via receive.
        contract
            .sender(alice)
            .constructor(alice, U64::ZERO, U64::ZERO)
            .motsu_expect("should construct");

        let value = uint!(101_U256);
        alice.fund(value);

        let before_alice = alice.balance();
        let before_contract = contract.balance();

        contract
            .sender_and_value(alice, value)
            .receive()
            .motsu_expect("should receive ETH");

        assert_eq!(before_alice - value, alice.balance());
        assert_eq!(before_contract + value, contract.balance());
    }

    #[motsu::test]
    fn owner_works(contract: Contract<VestingWallet>, alice: Address) {
        contract
            .sender(alice)
            .constructor(alice, U64::ZERO, U64::ZERO)
            .motsu_expect("should construct");
        assert_eq!(alice, contract.sender(alice).owner());
    }

    #[motsu::test]
    fn transfer_ownership_works(
        contract: Contract<VestingWallet>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            .constructor(alice, U64::ZERO, U64::ZERO)
            .motsu_expect("should construct");

        contract
            .sender(alice)
            .transfer_ownership(bob)
            .motsu_expect("owner should transfer");
        assert_eq!(bob, contract.sender(alice).owner());
    }

    #[motsu::test]
    fn renounce_ownership_works(
        contract: Contract<VestingWallet>,
        alice: Address,
    ) {
        contract
            .sender(alice)
            .constructor(alice, U64::ZERO, U64::ZERO)
            .motsu_expect("should construct");
        contract
            .sender(alice)
            .renounce_ownership()
            .motsu_expect("owner should renounce");
        assert_eq!(Address::ZERO, contract.sender(alice).owner());
    }

    #[motsu::test]
    fn releasable_eth_full_when_zero_duration(
        contract: Contract<VestingWallet>,
        alice: Address,
    ) {
        // Configure immediate vesting, deposit ETH, expect all releasable.
        contract
            .sender(alice)
            .constructor(alice, U64::ZERO, U64::ZERO)
            .motsu_expect("should construct");

        let value = U256::from(BALANCE);
        alice.fund(value);
        contract
            .sender_and_value(alice, value)
            .receive()
            .motsu_expect("should receive ETH");

        assert_eq!(value, contract.sender(alice).releasable_eth());
    }

    #[storage]
    struct PayableReceiver;

    unsafe impl TopLevelStorage for PayableReceiver {}

    #[public]
    impl PayableReceiver {
        #[receive]
        #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
        fn receive(&mut self) -> Result<(), Vec<u8>> {
            Ok(())
        }
    }

    #[motsu::test]
    fn release_eth_transfers_and_emits(
        contract: Contract<VestingWallet>,
        alice: Address,
        receiver: Contract<PayableReceiver>,
    ) {
        // Immediate vesting: all ETH becomes releasable and is sent to owner.
        contract
            .sender(alice)
            .constructor(alice, U64::ZERO, U64::ZERO)
            .motsu_expect("should construct");

        // Transfer ownership to a payable receiver contract so the low-level
        // ETH transfer succeeds in motsu unit tests.
        contract
            .sender(alice)
            .transfer_ownership(receiver.address())
            .motsu_expect("should transfer ownership to receiver");

        let value = U256::from(BALANCE);
        alice.fund(value);

        contract
            .sender_and_value(alice, value)
            .receive()
            .motsu_expect("should receive ETH");

        let before_receiver = receiver.balance();
        let before_contract = contract.balance();

        contract.sender(alice).release_eth().motsu_expect("should release ETH");

        let released = contract.sender(alice).released_eth();
        assert_eq!(released, value);
        assert_eq!(before_receiver + released, receiver.balance());
        assert_eq!(before_contract - released, contract.balance());

        contract.assert_emitted(&EtherReleased { amount: released });
    }

    #[motsu::test]
    fn check_vested_amount_eth(
        contract: Contract<VestingWallet>,
        alice: Address,
    ) {
        // Linear vesting for ETH mirrors ERC20 test logic.
        let start_ts = start();
        contract
            .sender(alice)
            .constructor(alice, start_ts, DURATION)
            .motsu_expect("should construct");

        // Deposit ETH into the contract.
        let value = U256::from(BALANCE);
        alice.fund(value);
        contract
            .sender_and_value(alice, value)
            .receive()
            .motsu_expect("should receive ETH");

        for i in 0..64_u64 {
            let timestamp: u64 =
                i * DURATION.to::<u64>() / 60 + start_ts.to::<u64>();
            let expected_amount = U256::from(std::cmp::min(
                BALANCE,
                BALANCE * (U256::from(timestamp) - U256::from(start_ts))
                    / U256::from(DURATION),
            ));

            let vested_amount =
                contract.sender(alice).vested_amount_eth(timestamp);
            assert_eq!(
                expected_amount, vested_amount,
                "\n---\ni: {i}\nstart: {start_ts}\ntimestamp: {timestamp}\n---\n",
            );
        }
    }
}
