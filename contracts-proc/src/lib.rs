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

/// Computes the interface id as an associated constant `INTERFACE_ID` for the
/// trait that describes contract's abi.
///
/// Selector collision should be handled with
/// macro `#[selector(name = "actualSolidityMethodName")]` on top of the method.
///
/// # Examples
///
/// ```rust,ignore
/// #[interface_id]
/// pub trait IErc721 {
///     fn balance_of(&self, owner: Address) -> Result<U256, Vec<u8>>;
///
///     fn owner_of(&self, token_id: U256) -> Result<Address, Vec<u8>>;
///
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
/// impl IErc165 for Erc721 {
///     fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
///         <Self as IErc721>::interface_id() == interface_id
///             || Erc165::interface_id() == interface_id
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn interface_id(attr: TokenStream, input: TokenStream) -> TokenStream {
    interface_id::interface_id(&attr, input)
}
