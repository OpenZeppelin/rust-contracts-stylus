//! A vesting wallet is an ownable contract that can receive native currency and ERC20 tokens, and release these
//! assets to the wallet owner, also referred to as "beneficiary", according to a vesting schedule.
//!
//! Any assets transferred to this contract will follow the vesting schedule as if they were locked from the beginning.
//! Consequently, if the vesting has already started, any amount of tokens sent to this contract will (at least partly)
//! be immediately releasable.
//!
//! By setting the duration to 0, one can configure this contract to behave like an asset timelock that hold tokens for
//! a beneficiary until a specified time.
//!
//! NOTE: Since the wallet is [`crate::access::ownable::Ownable`], and ownership can be transferred, it is possible to sell unvested tokens.
//! Preventing this in a smart contract is difficult, considering that: 1) a beneficiary address could be a
//! counterfactually deployed contract, 2) there is likely to be a migration path for EOAs to become contracts in the
//! near future.
//!
//! NOTE: When using this contract with any token whose balance is adjusted automatically (i.e. a rebase token), make
//! sure to account the supply/balance adjustment in the vesting schedule to ensure the vested amount is as intended.
use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolValue};
use stylus_sdk::{
    call::{Call, RawCall},
    contract::address,
    evm::gas_left,
    function_selector,
    storage::TopLevelStorage,
    stylus_proc::{public, sol_interface, sol_storage, SolidityError},
    types::AddressVM,
};

use crate::token::erc20;

sol! {
    /// Emitted when `amount` of ether has been released.
    #[allow(missing_docs)]
    event EtherReleased(uint256 amount);

    /// Emitted when `amount` of ERC20 `token` has been released.
    #[allow(missing_docs)]
    event ERC20Released(address indexed token, uint256 amount);
}

/// A VestingWallet error
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`erc20::Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error),
}

sol_interface! {
    /// Interface of the [`erc20::Erc20`] standard as defined in the ERC.
    interface IERC20 {
        /// Moves a `value` amount of tokens from the caller's account to `to`.
        ///
        /// Returns a boolean value indicating whether the operation succeeded.
        ///
        /// Emits a [`erc20::Transfer`] event.
        function transfer(address recipient, uint256 amount) external returns (bool);
    }
}

sol_storage! {
    /// Wrappers around ERC-20 operations that throw on failure (when the token
    /// contract returns false). Tokens that return no value (and instead revert or
    /// throw on failure) are also supported, non-reverting calls are assumed to be
    /// successful.
    /// To use this library you can add a `#[inherit(SafeErc20)]` attribute to
    /// your contract, which allows you to call the safe operations as
    /// `contract.safe_transfer(token_addr, ...)`, etc.
    #[allow(clippy::pub_underscore_fields)]
    pub struct VestingWallet {
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

#[public]
impl VestingWallet {
    /// The contract should be able to receive Eth.
    #[payable]
    pub fn receive_ether(&self) {}

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
    pub fn released_token(&self, token: Address) -> U256 {
        self._erc20_released.get(token)
    }
}
