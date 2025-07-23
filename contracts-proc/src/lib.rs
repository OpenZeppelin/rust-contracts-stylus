#![doc = include_str!("../README.md")]

extern crate proc_macro;
use proc_macro::TokenStream;

/// Shorthand to print nice errors.
///
/// Note that it's defined before the module declarations.
macro_rules! error {
    ($tokens:expr, $($msg:expr),+ $(,)?) => {{
        let error = syn::Error::new(syn::spanned::Spanned::span(&$tokens), format!($($msg),+));
        return error.to_compile_error().into();
    }};
    (@ $tokens:expr, $($msg:expr),+ $(,)?) => {{
        return Err(syn::Error::new(syn::spanned::Spanned::span(&$tokens), format!($($msg),+)))
    }};
}

mod interface_id;

/// Automatically computes the ERC-165 interface ID for a trait.
///
/// Adds an `interface_id` associated function to your trait by XOR-ing all
/// method selectors together, following the ERC-165 standard.
///
/// ## Method naming
///
/// By default, Rust method names are converted to camelCase for Solidity.
/// Use `#[selector(name = "...")]` to override the Solidity function name.
///
/// ## Examples
///
/// ### Basic usage
///
/// ```rust,ignore
/// #[interface_id]
/// pub trait IErc721 {
///     fn balance_of(&self, owner: Address) -> Result<U256, Vec<u8>>;
///     fn owner_of(&self, token_id: U256) -> Result<Address, Vec<u8>>;
///
///     // Function overloading: different Rust names, same Solidity name
///     fn safe_transfer_from(
///         &mut self,
///         from: Address,
///         to: Address,
///         token_id: U256,
///     ) -> Result<(), Vec<u8>>;
///
///     #[selector(name = "safeTransferFrom")]
///     fn safe_transfer_from_with_data(
///         &mut self,
///         from: Address,
///         to: Address,
///         token_id: U256,
///         data: Bytes,
///     ) -> Result<(), Vec<u8>>;
/// }
///
/// // Now you can use the computed interface ID:
/// impl IErc165 for Erc721 {
///     fn supports_interface(&self, interface_id: B32) -> bool {
///         <Self as IErc721>::interface_id() == interface_id
///             || <Self as IErc165>::interface_id() == interface_id
///     }
/// }
/// ```
///
/// ### Selector collision error
///
/// The macro will catch duplicate Solidity signatures at compile time:
///
/// ```compile_fail
/// #[interface_id]
/// trait BadTrait {
///     fn transfer(&self, to: Address, amount: U256);          // transfer(address,uint256)
///
///     #[selector(name = "transfer")]
///     fn send_tokens(&self, recipient: Address, value: U256); // transfer(address,uint256) ❌ collision!
/// }
/// ```
#[proc_macro_attribute]
pub fn interface_id(attr: TokenStream, input: TokenStream) -> TokenStream {
    interface_id::interface_id(&attr, input)
}
