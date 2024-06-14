//! Projective curve points.
use core::marker::PhantomData;

use bn::U256;

use super::curve::PrimeCurveParams;

/// Point on a Weierstrass curve in projective (homogeneous) coordinates.
#[derive(Clone, Copy, Debug)]
pub struct ProjectivePoint<C: PrimeCurveParams> {
    /// X-coordinate.
    pub x: U256,
    /// Y-coordinate.
    pub y: U256,
    /// Z-coordinate.
    pub z: U256,

    marker: PhantomData<C>,
}

impl<C: PrimeCurveParams> ProjectivePoint<C> {
    pub const IDENTITY: Self = Self {
        x: U256::ZERO,
        y: U256::ONE,
        z: U256::ZERO,
        marker: PhantomData,
    };

    pub const GENERATOR: Self = Self {
        x: C::GENERATOR.0,
        y: C::GENERATOR.1,
        z: U256::ONE,
        marker: PhantomData,
    };

    pub fn new(x: U256, y: U256, z: U256) -> Self {
        Self { x, y, z, marker: PhantomData }
    }
}
