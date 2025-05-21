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
        $crate::Constructor {
            signature: "constructor()".to_string(),
            args: vec![],
        }
    }};

    ($first:expr $(, $rest:expr)* $(,)?) => {{
        fn get_abi_str<T: stylus_sdk::abi::AbiType>(_: &T) -> &'static str {
            <T as stylus_sdk::abi::AbiType>::ABI.as_str()
        }

        let signature_params = vec![get_abi_str($first)$(, get_abi_str($rest))*].join(",");
        let args = vec![$first.to_string()$(, $rest.to_string())*];

        $crate::Constructor {
            signature: format!("constructor({})", signature_params),
            args,
        }
    }};
}
