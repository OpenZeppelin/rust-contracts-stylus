use alloy::primitives::{Address, U256, U8};

/// Constructor data.
pub struct Constructor {
    /// Constructor signature.
    pub signature: String,
    /// Constructor arguments.
    pub args: Vec<String>,
}

/// Helper trait to convert values to string representation
pub trait AbiTypeToString {
    /// Stringify ABI type.
    fn abi_type_to_string(&self) -> String;
}

macro_rules! impl_to_arg_string {
    ($($abi_type:ident),* $(,)?) => {$(
        impl AbiTypeToString for $abi_type {
            fn abi_type_to_string(&self) -> String {
                self.to_string()
            }
        }
    )*};
}

impl_to_arg_string!(U256, u64, String, U8, Address);

// Special implementation for Bytes
impl AbiTypeToString for stylus_sdk::abi::Bytes {
    fn abi_type_to_string(&self) -> String {
        format!("0x{}", stylus_sdk::hex::encode(self))
    }
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
        fn get_abi_str<T: stylus_sdk::abi::AbiType>(_: T) -> &'static str {
            <T as stylus_sdk::abi::AbiType>::ABI.as_str()
        }

        let signature_params = {
            let mut params = vec![get_abi_str($first)];
            $(params.push(get_abi_str($rest));)*
            params.join(",")
        };

        let args = vec![$crate::AbiTypeToString::abi_type_to_string(&$first)$(, $crate::AbiTypeToString::abi_type_to_string(&$rest))*];

        $crate::Constructor {
            signature: format!("constructor({})", signature_params),
            args,
        }
    }};
}
