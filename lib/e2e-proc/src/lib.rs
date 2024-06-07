use proc_macro::TokenStream;

mod test;

/// Defines an end-to-end Stylus contract test that sets up `e2e::User`s based
/// on the function's parameters.
///
/// # Examples
///
/// ```rust,ignore
/// #[e2e::test]
/// async fn foo(alice: User, bob: User) -> eyre::Result<()> {
///     let charlie = User::new().await?;
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, input: TokenStream) -> TokenStream {
    test::test(attr, input)
}
