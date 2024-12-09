use alloy_sol_types::SolEvent;

use crate::context::Context;

/// Asserts that the `expected` event was emitted in a test case.
pub fn emits<E>(expected: &E) -> bool
where
    E: SolEvent,
{
    let expected = expected.encode_data();
    let events = Context::current().events();
    events.into_iter().rev().any(|event| expected == event)
}

/// Removes all emitted events for a test case.
pub fn clear_events() {
    Context::current().clear_events();
}
