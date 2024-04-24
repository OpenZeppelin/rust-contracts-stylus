//! ERC20 Pausable extension TODO

use crate::{
    erc20::{IERC20Virtual, IERC20},
    utils::capped::ICapped,
};

/// This macro provides an implementation of the ERC-20 Capped extension.
///
/// It adds the `cap` and `set_cap` functions, and expects the token
/// to contain `erc20` as a field that implements IERC20Capped trait.
#[macro_export]
macro_rules! erc20_capped_impl {
    () => {
        pub(crate) fn cap(&self) -> alloy_primitives::U256 {
            self.erc20.cap()
        }

        pub(crate) fn set_cap(
            &mut self,
            cap: alloy_primitives::U256,
        ) -> Result<(), alloc::vec::Vec<u8>> {
            self.erc20.set_cap(cap).map_err(|e| e.into())
        }
    };
}

/// TODO docs
pub trait IERC20Capped: IERC20Virtual + IERC20 + ICapped {}

#[cfg(all(test, feature = "std"))]
mod tests {}
