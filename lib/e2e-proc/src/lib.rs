use proc_macro::TokenStream;

mod test;

/// Defines an end-to-end stylus contract test that provides test user's
/// injection from arguments.
///
/// # Examples
///
/// ```rust,ignore
/// #[e2e::test]
/// async fn mint(alice: User) -> Result<()> {
///     let erc721 = &alice.deploys::<Erc721>().await?;
///     let token_id = random_token_id();
///     let _ =
///         alice.uses(erc721).mint(alice.address(), token_id).ctx_send().await?;
///     let owner = alice.uses(erc721).owner_of(token_id).ctx_call().await?;
///     assert_eq!(owner, alice.address());
///
///     let balance =
///         alice.uses(erc721).balance_of(alice.address()).ctx_call().await?;
///     assert!(balance >= U256::one());
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, input: TokenStream) -> TokenStream {
    test::test(attr, input)
}
