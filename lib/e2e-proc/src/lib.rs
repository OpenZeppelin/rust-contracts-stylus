use proc_macro::TokenStream;

mod test;

/// Defines an end-to-end Stylus contract test that sets up `e2e::User`s based
/// on the function's parameters.
///
/// # Examples
///
/// ```rust,ignore
/// #[e2e::test]
/// async fn foo(alice: User) -> eyre::Result<()> {
///     let contract_addr = deploy(alice.url(), &alice.pk()).await?;
///     let contract = Erc721::new(contract_addr, &alice.signer);
///
///     let alice_addr = alice.address();
///     let token_id = random_token_id();
///     let _ = send!(contract.mint(alice_addr, token_id));
///     // ...
/// }

/// #[e2e::test]
///     let charlie = User::new().await?;
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, input: TokenStream) -> TokenStream {
    test::test(attr, input)
}
