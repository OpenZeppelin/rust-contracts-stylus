//! Key-handling related logic.

use super::{
    curve::PrimeCurve,
    point::{affine::AffinePoint, projective::ProjectivePoint},
};

/// Elliptic curve public keys.
///
/// This is a wrapper type for [`AffinePoint`] which ensures an inner
/// non-identity point and provides a common place to handle encoding/decoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicKey<C>
where
    C: PrimeCurve,
{
    point: AffinePoint<C>,
}

impl<C> PublicKey<C>
where
    C: PrimeCurve,
{
    /// Convert an [`AffinePoint`] into a [`PublicKey`].
    pub fn from_affine(point: AffinePoint<C>) -> Self {
        Self { point }
    }

    /// Compute a [`PublicKey`] from a secret scalar value (i.e. a secret key
    /// represented as a raw scalar value). Returns `None` if the value computed
    /// is the identity point.
    pub fn from_secret_scalar(scalar: &C::Uint) -> Option<Self> {
        (&ProjectivePoint::GENERATOR * scalar)
            .to_affine()
            .map(|point| Self { point })
    }

    /// Borrow the inner [`AffinePoint`] from this [`PublicKey`].
    ///
    /// In ECC, public keys are elliptic curve points.
    pub fn as_affine(&self) -> &AffinePoint<C> {
        &self.point
    }
}
