//! Signature error type.

use core::fmt::{self, Debug, Display};

// TODO: Once we update `alloy` to a crates.io version, use the `signature`
// crate directly for better composability with the ecosystem.
/// Result type.
///
/// A result with the `signature` crate's [`Error`] type.
pub type Result<T> = core::result::Result<T, Error>;

/// Signature errors.
///
/// This type is deliberately opaque as to avoid sidechannel leakage which
/// could potentially be used recover signing private keys or forge signatures
/// (e.g. [BB'06]).
///
/// [BB'06]: https://en.wikipedia.org/wiki/Daniel_Bleichenbacher
#[derive(Default)]
#[non_exhaustive]
pub struct Error {}

impl Error {
    /// Create a new error with no associated source.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("signature::Error {}")
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("signature error")?;

        Ok(())
    }
}
