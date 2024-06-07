use alloy::sol_types::SolError;

pub trait ErrorExt<E> {
    /// Asserts that current error result corresponds to the typed abi-encoded
    /// error `expected`.
    fn is(&self, expected: E) -> bool;
}

impl<E: SolError> ErrorExt<E> for alloy::contract::Error {
    fn is(&self, expected: E) -> bool {
        let Self::TransportError(e) = self else {
            return false;
        };

        let raw_value = e
            .as_error_resp()
            .and_then(|payload| payload.data.clone())
            .expect("should extract the error");
        let actual = &raw_value.get().trim_matches('"')[2..];
        let expected = alloy::hex::encode(expected.abi_encode());
        return expected == actual;
    }
}
