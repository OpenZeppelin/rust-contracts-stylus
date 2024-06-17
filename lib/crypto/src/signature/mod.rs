//! ECDSA signature.

pub mod bytes;
pub mod error;

use core::fmt;

use crate::elliptic_curve::curve::PrimeCurve;

/// ECDSA signature (fixed-size). Generic over elliptic curve types.
///
/// Serialized as fixed-sized big endian scalar values with no added framing:
///
/// - `r`: field element size for the given curve, big-endian.
/// - `s`: field element size for the given curve, big-endian.
///
/// Both `r` and `s` MUST be non-zero.
///
/// For example, in a curve with a 256-bit modulus like NIST P-256 or
/// secp256k1, `r` and `s` will both be 32-bytes and serialized as big endian,
/// resulting in a signature with a total of 64-bytes.
#[derive(Clone, Eq, PartialEq)]
pub struct Signature<C: PrimeCurve> {
    r: C::Uint,
    s: C::Uint,
}

impl<C: PrimeCurve> fmt::Debug for Signature<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ecdsa::Signature<{:?}>(", C::default())?;

        for byte in self.to_bytes() {
            write!(f, "{byte:02X}")?;
        }

        write!(f, ")")
    }
}

impl<C: PrimeCurve> fmt::Display for Signature<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:X}")
    }
}

impl<C: PrimeCurve> fmt::LowerHex for Signature<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.to_bytes() {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl<C: PrimeCurve> fmt::UpperHex for Signature<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.to_bytes() {
            write!(f, "{byte:02X}")?;
        }
        Ok(())
    }
}
