//! Context functions used for mocking unit tests.

use stylus_sdk::{alloy_primitives::Address, msg};

/// Returns the address of the message sender.
#[cfg(test)]
pub fn msg_sender() -> Address {
    msg::sender()
}

/// Returns the address of the message sender.
#[cfg(not(test))]
pub fn msg_sender() -> Address {
    msg::sender()
}
