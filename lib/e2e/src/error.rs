use alloy::{
    rpc::json_rpc::ErrorPayload,
    sol_types::SolError,
    transports::{RpcError, TransportErrorKind},
};
use stylus_sdk::call::MethodError;

/// JSON-RPC error code for execution reverted.
const EXECUTION_REVERTED_CODE: i64 = 3;

/// JSON-RPC error message for execution reverted.
const EXECUTION_REVERTED_MESSAGE: &str = "execution reverted";

/// Possible panic codes for a revert.
///
/// Taken from <https://github.com/NomicFoundation/hardhat/blob/main/packages/hardhat-chai-matchers/src/internal/reverted/panic.ts>
#[derive(Debug)]
#[allow(missing_docs)] // Pretty straightforward variant names.
pub enum PanicCode {
    AssertionError = 0x1,
    ArithmeticOverflow = 0x11,
    DivisionByZero = 0x12,
    EnumConversionOutOfBounds = 0x21,
    IncorrectlyEncodedStorageByteArray = 0x22,
    PopOnEmptyArray = 0x31,
    ArrayAccessOutOfBounds = 0x32,
    TooMuchMemoryAllocated = 0x41,
    ZeroInitializedVariable = 0x51,
}

impl core::fmt::Display for PanicCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            PanicCode::AssertionError =>
                "Assertion error",
            PanicCode::ArithmeticOverflow =>
                "Arithmetic operation overflowed outside of an unchecked block",
            PanicCode::DivisionByZero =>
                "Division or modulo division by zero",
            PanicCode::EnumConversionOutOfBounds =>
                "Tried to convert a value into an enum, but the value was too big or negative",
            PanicCode::IncorrectlyEncodedStorageByteArray =>
                "Incorrectly encoded storage byte array",
            PanicCode::PopOnEmptyArray =>
                ".pop() was called on an empty array",
            PanicCode::ArrayAccessOutOfBounds =>
                "Array accessed at an out-of-bounds or negative index",
            PanicCode::TooMuchMemoryAllocated =>
                "Too much memory was allocated, or an array was created that is too large",
            PanicCode::ZeroInitializedVariable =>
                "Called a zero-initialized variable of internal function type"
        };

        write!(f, "{msg}")
    }
}

/// An error representing a Solidity-style panic with a specific code.
pub trait Panic {
    /// Checks that `Self` corresponds to a Solidity panic with code `code`.
    fn panicked_with(&self, code: PanicCode) -> bool;
}

/// An error representing a revert with custom error data.
pub trait Revert<E> {
    /// Checks that `Self` corresponds to the typed abi-encoded error
    /// `expected`.
    fn reverted_with(&self, expected: E) -> bool;
}

/// An error representing a Rust panic that caused a revert.
///
/// When Rust code panics (via `unwrap()`, `expect()`, `panic!()`, assertions,
/// etc.) in a Stylus contract, it reverts the transaction without
/// Solidity-style error data. This results in `data: "0x"` in the RPC error
/// response.
pub trait RustPanic {
    /// Checks that `Self` corresponds to a Rust panic.
    ///
    /// Returns `true` if the transaction reverted due to a Rust panic
    /// without custom error handling.
    fn panicked(&self) -> bool;
}

impl Panic for alloy::contract::Error {
    fn panicked_with(&self, code: PanicCode) -> bool {
        extract_error_payload(self)
            .map_or(false, |payload| payload.code == code as i64)
    }
}

impl RustPanic for alloy::contract::Error {
    fn panicked(&self) -> bool {
        extract_error_payload(self).map_or(false, |payload| {
            payload.code == EXECUTION_REVERTED_CODE
                && payload.message == EXECUTION_REVERTED_MESSAGE
        })
    }
}

impl<E: MethodError> Revert<E> for alloy::contract::Error {
    fn reverted_with(&self, expected: E) -> bool {
        let raw_value = extract_error_payload(self)
            .and_then(|payload| payload.data.as_ref())
            .expect("should extract the error");

        let actual = &raw_value.get().trim_matches('"')[2..];
        let expected = alloy::hex::encode(expected.encode());
        expected == actual
    }
}

// Helper to extract the error response payload.
fn extract_error_payload(
    error: &alloy::contract::Error,
) -> Option<&ErrorPayload> {
    match error {
        alloy::contract::Error::TransportError(e) => e.as_error_resp(),
        _ => None,
    }
}

impl<E: SolError> Revert<E> for eyre::Report {
    fn reverted_with(&self, expected: E) -> bool {
        // Generic revert error
        let Some(received) = self
            .chain()
            .find_map(|err| err.downcast_ref::<RpcError<TransportErrorKind>>())
        else {
            return false;
        };
        let RpcError::ErrorResp(received) = received else {
            return false;
        };
        let Some(received) = &received.data else {
            return false;
        };
        let expected = alloy::hex::encode(expected.abi_encode());
        received.to_string().contains(&expected)
    }
}
