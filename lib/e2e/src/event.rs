use alloy::{rpc::types::eth::TransactionReceipt, sol_types::SolEvent};

pub trait EventExt<E> {
    /// Asserts the contract emitted the `expected` event.
    fn emits(&self, expected: E);
}

impl<E> EventExt<E> for TransactionReceipt
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
