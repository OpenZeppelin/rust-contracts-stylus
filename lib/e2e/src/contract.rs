use std::sync::Arc;

use ethers::prelude::*;

use crate::HttpMiddleware;

/// Abstraction for the deployed contract.
pub trait Contract {
    /// Crate name of the contract.
    ///
    /// e.g can be `erc721-example`.
    const CRATE_NAME: &'static str;

    /// Abstracts token creation function.
    ///
    /// e.g. `Self::new(address, client)`.
    fn new(address: Address, client: Arc<HttpMiddleware>) -> Self;
}

#[macro_export]
/// Link `abigen!` contract to the crate name.
///
/// # Example
/// ```
/// use e2e::prelude::*;
///
/// abigen!(
///     Erc20Token,
///     r#"[
///         function transferFrom(address sender, address recipient, uint256 amount) external returns (bool)
///         function mint(address account, uint256 amount) external
///
///         error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed)
///     ]"#
/// );
///
/// pub type Erc20 = Erc20Token<HttpMiddleware>;
/// link_to_crate!(Erc20, "erc20-example");
/// ```
macro_rules! link_to_crate {
    ($token_type:ty, $program_address:literal) => {
        impl $crate::prelude::Contract for $token_type {
            const CRATE_NAME: &'static str = $program_address;

            fn new(
                address: ethers::types::Address,
                client: std::sync::Arc<HttpMiddleware>,
            ) -> Self {
                Self::new(address, client)
            }
        }
    };
}
