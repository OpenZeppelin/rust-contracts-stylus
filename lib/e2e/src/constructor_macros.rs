/// Constructor data.
pub struct Constructor {
    /// Constructor signature.
    pub signature: String,
    /// Constructor arguments.
    pub args: Vec<String>,
}

/// Generates a function selector for the given method and its args.
#[macro_export]
macro_rules! constructor {
    () => {{
        $crate::constructor::Constructor {
            signature: "constructor()".to_string(),
            args: vec![],
        }
    }};

    ($first:ty $(, $ty:ty)* $(,)?) => {{
        $crate::constructor::Constructor {
            signature: format!("constructor({})", $(<$ty as stylus_sdk::abi::AbiType>::ABI.as_str(), )*),
            args: vec![$($ty.to_string(),)*],
        }
    }};
}
