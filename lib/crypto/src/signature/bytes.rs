//! Signature conversion from and to bytes.

use core::mem;

use bigint::Encoding;

use super::{
    error::{Error, Result},
    Signature,
};
use crate::elliptic_curve::curve::PrimeCurve;

impl<C: PrimeCurve> Signature<C> {
    /// Parse a signature from fixed-width bytes, i.e. 2 * the size of field
    /// elements.
    ///
    /// # Returns
    /// - `Ok(signature)` if the `r` and `s` components are both in the valid
    ///   range `1..n` when serialized as concatenated big endian integers.
    /// - `Err(err)` if the `r` and/or `s` component of the signature is
    ///   out-of-range when interpreted as a big endian integer.
    pub fn from_bytes(bytes: &[u8; 64]) -> Result<Self> {
        // SAFETY: `bytes` is 64 bytes long.
        let bytes: [[u8; 32]; 2] = unsafe { mem::transmute(*bytes) };

        let r = C::Uint::from_be_bytes(bytes[0]);
        let s = C::Uint::from_be_bytes(bytes[1]);

        if !C::is_member(r) || !C::is_member(s) {
            return Err(Error::new());
        }

        Ok(Signature { r, s })
    }

    /// Parse a signature from a byte slice.
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        <&[u8; 64]>::try_from(slice)
            .map_err(|_| Error::new())
            .and_then(Self::from_bytes)
    }

    /// Split the signature into its `r` and `s` components, represented as
    /// bytes.
    pub fn split_bytes(&self) -> ([u8; 32], [u8; 32]) {
        (self.r.to_be_bytes(), self.s.to_be_bytes())
    }

    /// Serialize this signature as bytes.
    pub fn to_bytes(&self) -> [u8; 64] {
        let mut bytes = [0; 64];
        bytes[..32].copy_from_slice(&self.r.to_be_bytes());
        bytes[32..].copy_from_slice(&self.s.to_be_bytes());
        bytes
    }
}
