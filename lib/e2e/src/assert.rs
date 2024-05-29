use alloy::{hex::FromHex, sol_types::SolError, transports::RpcError};

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
        let selector = raw_value.get().trim_matches('"');
        let selector: [u8; 4] = FromHex::from_hex(selector)
            .expect("should extract the error selector");

        assert_eq!(selector, R::SELECTOR);
    }
}
