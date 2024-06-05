//! Procedural macro definitions used in `motsu`.
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod stylus_default;
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

#[proc_macro_derive(StylusDefault)]
pub fn derive_stylus_default(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    stylus_default::impl_stylus_default(&input)
}
