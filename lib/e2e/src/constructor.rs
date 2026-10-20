/// Constructor data: the ABI-encoded constructor arguments.
///
/// Build it with the [`constructor!`](crate::constructor) macro.
#[derive(Clone, Debug, Default)]
pub struct Constructor {
    /// ABI-encoded constructor arguments, without a selector.
    pub args: Vec<u8>,
}

/// Builds a [`Constructor`] from the given arguments.
///
/// Every argument must implement `alloy::sol_types::SolValue`, which holds
/// for the `alloy` primitives (`Address`, `U256`, `U8`, `Bytes`, ...),
/// `String`, `bool` and the Rust integer types.
///
/// # Examples
///
/// ```rust,ignore
/// let ctr = constructor!("Token".to_string(), "TKN".to_string(), U256::from(1000));
/// let receipt = alice.as_deployer().with_constructor(ctr).deploy().await?;
/// ```
#[macro_export]
macro_rules! constructor {
    () => {
        $crate::Constructor { args: ::std::vec::Vec::new() }
    };
    ($($arg:expr),+ $(,)?) => {
        $crate::Constructor {
            args: $crate::__private::SolValue::abi_encode_params(&($($arg,)+)),
        }
    };
}
