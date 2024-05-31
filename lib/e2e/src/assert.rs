use alloy::{sol_types::SolError, transports::RpcError};

pub trait Assert<E> {
    /// Asserts that current error result corresponds to the typed abi-encoded
    /// error `expected`.
    fn assert(&self, expected: E);
}

impl<R: SolError, E> Assert<R> for RpcError<E> {
    fn assert(&self, _: R) {
        let raw_value = self
            .as_error_resp()
            .map(|payload| payload.data.clone())
            .flatten()
            .expect("should extract the error");
        let raw_error = raw_value.get().trim_matches('"');
        let selector = alloy::hex::encode(R::SELECTOR);

        assert!(raw_error.contains(&selector));
    }
}

impl<R: SolError> Assert<R> for alloy::contract::Error {
    fn assert(&self, expected: R) {
        if let Self::TransportError(e) = self {
            e.assert(expected);
        }
    }
}
