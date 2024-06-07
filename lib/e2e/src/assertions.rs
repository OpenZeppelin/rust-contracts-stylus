use alloy::{
    rpc::types::eth::TransactionReceipt,
    sol_types::{SolError, SolEvent},
    transports::RpcError,
};

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

pub trait Emits<E> {
    /// Asserts the transaction emitted the `expected` event.
    fn emits(&self, expected: E);
}

impl<E> Emits<E> for TransactionReceipt
where
    E: SolEvent,
    E: PartialEq,
    E: std::fmt::Debug,
{
    fn emits(&self, expected: E) {
        // Extract all events that are the expected type.
        let emitted = self
            .inner
            .logs()
            .iter()
            .filter_map(|log| log.log_decode().ok())
            .map(|log| log.inner.data)
            .any(|event| expected == event);

        assert!(emitted, "Event {:?} not emitted", expected);
    }
}
