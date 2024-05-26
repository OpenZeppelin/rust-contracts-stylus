use alloy::{hex, sol_types::SolError};

pub trait Assert<E: SolError> {
    /// Asserts that current error result corresponds to the typed abi encoded
    /// error `expected`.
    fn assert(&self, expected: E);
}

impl<E: SolError> Assert<E> for alloy::contract::Error {
    fn assert(&self, expected: E) {
        let received = format!("{:#}", self);
        let expected = hex::encode(expected.abi_encode());
        assert!(received.contains(&expected));
    }
}
