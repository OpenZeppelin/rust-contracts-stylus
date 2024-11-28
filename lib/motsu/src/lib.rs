//! # Motsu - Unit Testing for Stylus
//!
//! This crate enables unit-testing for Stylus contracts. It abstracts away the
//! machinery necessary for writing tests behind a
//! [`#[motsu::test]`][test_attribute] procedural macro.
//!
//! The name `motsu` is an analogy to the place where you put your fingers to
//! hold a stylus pen.
//!
//! ## Usage
//!
//! Annotate tests with [`#[motsu::test]`][test_attribute] instead of `#[test]`
//! to get access to VM affordances.
//!
//! Note that we require contracts to implement
//! `stylus_sdk::prelude::StorageType`. This trait is typically implemented by
//! default with `stylus_proc::sol_storage` macro.
//!
//! ```rust
//! #[cfg(test)]
//! mod tests {
//!     use contracts::token::erc20::Erc20;
//!
//!     #[motsu::test]
//!     fn reads_balance(contract: Erc20) {
//!         let balance = contract.balance_of(Address::ZERO); // Access storage.
//!         assert_eq!(balance, U256::ZERO);
//!     }
//! }
//! ```
//!
//! Annotating a test function that accepts no parameters will make
//! [`#[motsu::test]`][test_attribute] behave the same as `#[test]`.
//!
//! ```rust,ignore
//! #[cfg(test)]
//! mod tests {
//!     #[motsu::test] // Equivalent to #[test]
//!     fn test_fn() {
//!         ...
//!     }
//! }
//! ```
//!
//! [test_attribute]: crate::test
mod context;
pub mod prelude;
mod shims;

pub use motsu_proc::test;

#[cfg(all(test))]
mod tests {
    #![deny(rustdoc::broken_intra_doc_links)]
    extern crate alloc;

    use alloy_primitives::uint;
    use stylus_sdk::{
        alloy_primitives::{Address, U256},
        call::Call,
        prelude::{public, sol_storage, StorageType, TopLevelStorage},
    };

    use crate::context::{Account, Contract, TestRouter};

    /// Message sender that mocks `msg::sender()` in tests.
    pub fn msg_sender() -> Address {
        crate::prelude::Context::current()
            .get_msg_sender()
            .expect("msg_sender should be set")
    }

    sol_storage! {
        pub struct PingContract {
            uint256 _pings_count;
            address _pinged_from;
        }
    }

    #[public]
    impl PingContract {
        fn ping(&mut self, to: Address, value: U256) -> Result<U256, Vec<u8>> {
            let receiver = receiver::PongContract::new(to);
            let call = Call::new_in(self);
            let value =
                receiver.pong(call, value).expect("should pong successfully");

            let pings_count = self._pings_count.get();
            self._pings_count.set(pings_count + uint!(1_U256));
            self._pinged_from.set(msg_sender());
            Ok(value)
        }

        fn ping_count(&self) -> U256 {
            self._pings_count.get()
        }

        fn pinged_from(&self) -> Address {
            self._pinged_from.get()
        }
    }

    unsafe impl TopLevelStorage for PingContract {}

    mod receiver {
        use super::alloc;
        stylus_sdk::stylus_proc::sol_interface! {
            interface PongContract {
                #[allow(missing_docs)]
                function pong(uint256 value) external returns (uint256);
            }
        }
    }

    sol_storage! {
        pub struct PongContract {
            uint256 _pongs_count;
            address _ponged_from;
        }
    }

    #[public]
    impl PongContract {
        pub fn pong(&mut self, value: U256) -> Result<U256, Vec<u8>> {
            let pongs_count = self._pongs_count.get();
            self._pongs_count.set(pongs_count + uint!(1_U256));
            self._ponged_from.set(msg_sender());
            Ok(value + uint!(1_U256))
        }

        fn pong_count(&self) -> U256 {
            self._pongs_count.get()
        }

        fn ponged_from(&self) -> Address {
            self._ponged_from.get()
        }
    }

    unsafe impl TopLevelStorage for PongContract {}

    #[test]
    fn ping_pong_works() {
        let mut ping = Contract::<PingContract>::default();
        let ping = &mut ping;
        let mut pong = Contract::<PongContract>::default();
        let pong = &mut pong;

        let alice = Account::random();

        let value = uint!(10_U256);
        let ponged_value = ping
            .sender(alice)
            .ping(pong.address(), value)
            .expect("should ping successfully");

        assert_eq!(ponged_value, value + uint!(1_U256));
        assert_eq!(ping.sender(alice).ping_count(), uint!(1_U256));
        assert_eq!(pong.sender(alice).pong_count(), uint!(1_U256));

        assert_eq!(ping.sender(alice).pinged_from(), alice.address());
        assert_eq!(pong.sender(alice).ponged_from(), ping.address());
    }
}
