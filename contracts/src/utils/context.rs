//! Context functions used for mocking unit tests.

use stylus_sdk::{alloy_primitives::Address, msg};

/// Returns the address of the message sender.
#[cfg(test)]
pub fn msg_sender() -> Address {
    motsu::prelude::Context::current()
        .get_msg_sender()
        .expect("msg_sender should be set")
}

/// Returns the address of the message sender.
#[cfg(not(test))]
pub fn msg_sender() -> Address {
    msg::sender()
}
