use alloy::{
    rpc::types::eth::TransactionReceipt,
    sol_types::{SolError, SolEvent},
    transports::RpcError,
};

pub trait Assert<E> {
    /// Asserts that current error result corresponds to the typed abi-encoded
    /// error `expected`.
    fn assert(&self, expected: E);
}

impl<R: SolError, E> Assert<R> for RpcError<E> {
    fn assert(&self, _: R) {
        let raw_value = self
            .as_error_resp()
            .and_then(|payload| payload.data.clone())
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

pub trait Emits<E> {
    /// Asserts that transaction emitted an `expected` event.
    fn emits(&self, expected: E);
}

impl<E> Emits<E> for TransactionReceipt
where
    E: SolEvent,
    E: PartialEq,
    E: std::fmt::Debug,
{
    fn emits(&self, expected: E) {
        // Extract all events the are the expected type;
        let emitted = self
            .inner
            .logs()
            .iter()
            .filter_map(|log| log.log_decode().ok())
            .map(|log| log.inner.data)
            .any(|event| expected == event);

        assert_eq!(emitted, true, "Event {:?} not emitted", expected);
    }
}
