use eyre::{bail, Report};

use crate::prelude::abi::AbiEncode;

pub trait Assert<E: AbiEncode> {
    /// Asserts that current error result corresponds to the typed abi encoded
    /// error `expected_err`.
    fn assert(&self, expected_err: E) -> eyre::Result<()>;
}

impl<E: AbiEncode> Assert<E> for Report {
    fn assert(&self, expected_err: E) -> eyre::Result<()> {
        let received_err = format!("{:#}", self);
        let expected_err = expected_err.encode_hex();
        if received_err.contains(&expected_err) {
            Ok(())
        } else {
            bail!("Different error expected: Expected error is {expected_err}: Received error is {received_err}")
        }
    }
}
