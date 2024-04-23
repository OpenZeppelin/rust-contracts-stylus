//! ERC20 Pausable extension TODO

use crate::{
    erc20::{IERC20Virtual, IERC20},
    utils::capped::ICapped,
};

/// TODO docs
pub trait IERC20Capped: IERC20Virtual + IERC20 + ICapped {}

#[cfg(all(test, feature = "std"))]
mod tests {}
