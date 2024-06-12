//! Procedural macro definitions used in `motsu`.
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

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

mod default_storage_layout;
mod test;

/// Defines a unit test that provides access to Stylus' execution context.
///
/// Internally, this is a thin wrapper over `#[test]` that gives access to
/// affordances like contract storage and `msg::sender`. If you don't need
/// them, you can pass no arguments to the test function or simply use
/// `#[test]` instead of `#[motsu::test]`.
///
/// # Examples
///
/// ```rust,ignore
/// #[cfg(test)]
/// mod tests {
///     #[motsu::test]
///     fn reads_balance(contract: Erc20) {
///        let balance = contract.balance_of(Address::ZERO);
///        assert_eq!(U256::ZERO, balance);
///
///        let owner = msg::sender();
///        let one = U256::from(1);
///        contract._balances.setter(owner).set(one);
///        let balance = contract.balance_of(owner);
///        assert_eq!(one, balance);
///     }
/// }
/// ```
///
/// ```rust,ignore
/// #[cfg(test)]
/// mod tests {
///     #[motsu::test]
///     fn t() { // If no params, it expands to a `#[test]`.
///         ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, input: TokenStream) -> TokenStream {
    test::test(attr, input)
}

/// Automatically implements the `Default` trait for a struct that uses
/// `sol_storage!`.
///
/// This macro initializes the struct fields based on how they are laid out in
/// the EVM state trie. It is intended to be a helper for tests to avoid having
/// to implement `Default` for each contract.
///
/// # Usage
///
/// To use this macro, simply add `#[derive(motsu::DefaultStorageLayout)]` to your
/// `sol_storage!` struct. Make sure all the fields in your struct are
/// compatible with Stylus' storage, that means they implement the `StorageType`
/// trait.
///
/// # Examples
///
/// ```rust,ignore
/// sol_storage! {
///    #[derive(motsu::DefaultStorageLayout)]
///    pub struct Erc20 {
///        /// Maps users to balances.
///        mapping(address => uint256) _balances;
///        /// Maps users to a mapping of each spender's allowance.
///        mapping(address => mapping(address => uint256)) _allowances;
///        /// The total supply of the token.
///        uint256 _total_supply;
///    }
/// }
/// ```
///
/// ## Notice
///
/// For now this macro only works with structs that use the
/// `sol_storage!` macro, which allows you to use the solidity syntax.
/// If you want to write your contracts using `#[solidity_storage]`
/// and the Rust syntax, you will have to implement `Default` yourself.
///
/// ## See Also
///
/// - [Layout of State Variables in Storage](https://docs.soliditylang.org/en/latest/internals/layout_in_storage.html)

#[proc_macro_derive(DefaultStorageLayout)]
pub fn derive_stylus_default(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    default_storage_layout::impl_stylus_default(&input)
}
