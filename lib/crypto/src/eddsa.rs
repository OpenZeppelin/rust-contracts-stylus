//! This module contains an ed25519 signature implementation ([EDDSA]), that
//! includes key derivation, signing, and signature verification.
//!
//! Api and implementation of this module resembles [curve25519-dalek] crate and
//! based on `openzeppelin-crypto` primitives.
//!
//! [EDDSA]: https://en.wikipedia.org/wiki/EdDSA
//! [curve25519-dalek]: https://github.com/dalek-cryptography/curve25519-dalek

#![allow(non_snake_case)]
use sha2::{digest::Digest, Sha512};
use zeroize::ZeroizeOnDrop;

use crate::{
    arithmetic::{
        uint::{U256, U512},
        BigInteger,
    },
    curve::{
        te::{
            instance::curve25519::{Curve25519Config, Curve25519FrParam},
            Affine, Projective,
        },
        CurveGroup, PrimeGroup,
    },
    field::{
        fp::{Fp256, Fp512, FpParams, LIMBS_512},
        prime::PrimeField,
    },
    fp_from_num, from_num,
};

/// Ed25519 scalar.
pub(crate) type Scalar = Fp256<Curve25519FrParam>;

/// Ed25519 scalar used for reduction sha512 hash values to `256-bit`.
pub(crate) type WideScalar = Fp512<Curve25519Fr512Param>;

/// Scalar field parameters for curve ed25519 with `512-bit` inner integer size.
pub(crate) struct Curve25519Fr512Param;
impl FpParams<LIMBS_512> for Curve25519Fr512Param {
    const GENERATOR: Fp512<Self> = fp_from_num!("2");
    const MODULUS: U512 = from_num!("7237005577332262213973186563042994240857116359379907606001950938285454250989");
}

/// Ed25519 projective point.
pub(crate) type ProjectivePoint = Projective<Curve25519Config>;

/// Ed25519 affine point.
pub(crate) type AffinePoint = Affine<Curve25519Config>;

/// Ed25519 secret key as defined in [RFC8032 § 5.1.5]:
///
/// The private key is 32 octets (256 bits, corresponding to b) of
/// cryptographically secure random data.
///
/// [RFC8032 § 5.1.5]: https://www.rfc-editor.org/rfc/rfc8032#section-5.1.5
pub type SecretKey = [u8; SECRET_KEY_LENGTH];

/// The length of an ed25519 `SecretKey` in bytes.
pub const SECRET_KEY_LENGTH: usize = 32;

/// Contains the secret scalar and domain separator used for generating
/// signatures.
///
/// This is used internally for signing.
///
/// In the usual Ed25519 signing algorithm, `scalar` and `hash_prefix` are
/// defined such that `scalar || hash_prefix = H(sk)` where `sk` is the signing
/// key and `H` is SHA-512.
///
/// Instances of this secret are automatically overwritten with zeroes when they
/// fall out of scope.
#[derive(Copy, Clone, PartialEq)]
pub(crate) struct ExpandedSecretKey {
    /// The secret scalar used for signing.
    pub(crate) scalar: Scalar,
    /// The domain separator used when hashing the message to generate the
    /// pseudorandom `r` value.
    pub(crate) hash_prefix: [u8; 32],
}

impl ZeroizeOnDrop for ExpandedSecretKey {}

impl ExpandedSecretKey {
    pub(crate) fn from_bytes(bytes: &[u8]) -> ExpandedSecretKey {
        let hash = Sha512::default().chain_update(bytes).finalize();
        let bytes = &*hash;
        let mut scalar_bytes = [0u8; 32];
        let mut hash_prefix = [0u8; 32];
        scalar_bytes.copy_from_slice(&bytes[00..32]);
        hash_prefix.copy_from_slice(&bytes[32..64]);

        let scalar = Scalar::from_bigint(U256::from_bytes_le(&clamp_integer(
            scalar_bytes,
        )));
        Self { scalar, hash_prefix }
    }
}

/// Clamps the given little-endian representation of a 32-byte integer.
///
/// Clamping the value puts it in the range:
/// **n ∈ 2^254 + 8\*{0, 1, 2, 3, ..., 2^251 − 1}**
///
/// # Explanation of clamping
///
/// For Curve25519, h = 8, and multiplying by 8 is the same as a binary
/// left-shift by 3 bits. If you take a secret scalar value between 2^251 and
/// 2^252 – 1 and left-shift by 3 bits, then you end up with a 255-bit number
/// with the most significant bit set to 1 and the least-significant three bits
/// set to 0.
///
/// The Curve25519 clamping operation takes **an arbitrary 256-bit random
/// value** and clears the most-significant bit (making it a 255-bit number),
/// sets the next bit, and then clears the 3 least-significant bits. In other
/// words, it directly creates a scalar value that is in the right form and
/// pre-multiplied by the cofactor.
///
/// See [clamping reference] for more details.
///
/// [clamping reference]: https://neilmadden.blog/2020/05/28/whats-the-curve25519-clamping-all-about/
#[must_use]
pub const fn clamp_integer(mut bytes: [u8; 32]) -> [u8; 32] {
    bytes[0] &= 0b1111_1000;
    bytes[31] &= 0b0111_1111;
    bytes[31] |= 0b0100_0000;
    bytes
}

impl From<&SecretKey> for ExpandedSecretKey {
    #[allow(clippy::unwrap_used)]
    fn from(secret_key: &SecretKey) -> ExpandedSecretKey {
        let hash = Sha512::default().chain_update(secret_key).finalize();
        ExpandedSecretKey::from_bytes(hash.as_ref())
    }
}

/// Ed25519 signing key which can be used to produce signatures.
/// Invariant: `verifying_key` is always the public key of
/// `secret_key`.
/// This prevents the signing function [oracle attack].
///
/// [oracle attack]: https://github.com/MystenLabs/ed25519-unsafe-libs
#[derive(Copy, Clone, PartialEq)]
pub struct SigningKey {
    /// The secret half of this signing key.
    pub(crate) signing_key: ExpandedSecretKey,
    /// The public half of this signing key.
    pub(crate) verifying_key: VerifyingKey,
}

impl SigningKey {
    /// Get the [`VerifyingKey`] for this [`SigningKey`].
    #[must_use]
    pub fn verifying_key(&self) -> VerifyingKey {
        self.verifying_key
    }

    /// Construct a [`SigningKey`] from a [`SecretKey`]
    #[inline]
    #[must_use]
    pub fn from_bytes(secret_key: &SecretKey) -> Self {
        let signing_key: ExpandedSecretKey = secret_key.into();
        let verifying_key: VerifyingKey = signing_key.into();
        Self { signing_key, verifying_key }
    }

    /// Verify a signature on a message with this signing key's public key.
    #[must_use]
    pub fn is_valid_signature(
        &self,
        message: &[u8],
        signature: &Signature,
    ) -> bool {
        self.verifying_key.is_valid(message, signature)
    }

    /// Sign a message with this signing key's secret key.
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> Signature {
        let mut h = Sha512::new();

        h.update(self.signing_key.hash_prefix);
        h.update(message);

        let r = WideScalar::from_bigint(U512::from_bytes_le(
            h.finalize().as_slice(),
        ));
        let r = Scalar::from_fp(r);

        let R = ProjectivePoint::generator() * r;

        h = Sha512::new();
        h.update(CompressedPointY::from(R.into_affine()));
        h.update(CompressedPointY::from(
            self.verifying_key.point.into_affine(),
        ));
        h.update(message);

        let k = WideScalar::from_bigint(U512::from_bytes_le(
            h.finalize().as_slice(),
        ));
        let k = Scalar::from_fp(k);
        let s: Scalar = (k * self.signing_key.scalar) + r;

        Signature { R, s }
    }
}

/// In "Ed25519" format, the curve point `(x,y)` is
/// determined by the y-coordinate and the sign of `x`.
///
/// The first 255 bits of a `CompressedEdwardsY` represent the `y`-coordinate.
/// The high bit of the 32nd byte gives the sign of `x`.
#[derive(Copy, Clone, Hash)]
pub struct CompressedPointY([u8; 32]);

impl AsRef<[u8]> for CompressedPointY {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<AffinePoint> for CompressedPointY {
    fn from(point: AffinePoint) -> Self {
        let mut s: [u8; 32] = point
            .y
            .into_bigint()
            .into_bytes_le()
            .try_into()
            .expect("Y coordinate should be of 32 bit");

        let is_odd = point.x.into_bigint().is_odd();
        s[31] ^= u8::from(is_odd) << 7;

        CompressedPointY(s)
    }
}

/// Ed25519 signature representation.
///
/// Signature verification libraries are expected to reject invalid
/// field elements at the time a signature is verified.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Signature {
    /// `R` is an `EdwardsPoint`, formed by using an hash function with
    /// `512-bits` output to produce the digest of:
    ///
    /// - the nonce half of the `ExpandedSecretKey`, and
    /// - the message to be signed.
    ///
    /// This digest is then interpreted as a `Scalar` and reduced into an
    /// element in ℤ/lℤ.  The scalar is then multiplied by the distinguished
    /// basepoint to produce `R`, and `EdwardsPoint`.
    pub(crate) R: ProjectivePoint,

    /// `s` is a `Scalar`, formed by using a hash function with `512-bits`
    /// output to produce the digest of:
    ///
    /// - the `r` portion of this `Signature`,
    /// - the `PublicKey` which should be used to verify this `Signature`, and
    /// - the message to be signed.
    ///
    /// This digest is then interpreted as a `Scalar` and reduced into an
    /// element in ℤ/lℤ.
    pub(crate) s: Scalar,
}

impl Signature {}

/// An ed25519 public key.
#[derive(Copy, Clone, Default, PartialEq)]
pub struct VerifyingKey {
    // Edwards point used for curve arithmetic operations.
    pub(crate) point: ProjectivePoint,
}

impl VerifyingKey {
    /// Verify a signature on a message with this keypair's public key.
    pub(crate) fn is_valid(
        &self,
        message: &[u8],
        signature: &Signature,
    ) -> bool {
        let expected_r = self.compute_R(signature, message);
        expected_r == signature.R
    }

    /// Helper for verification. Computes the expected R component of the
    /// signature. The caller compares this to the real R component.
    /// This computes `H(R || A || M)` where `H` is the 512-bit hash function
    /// given by `CtxDigest` (this is SHA-512 in spec-compliant Ed25519).
    fn compute_R(
        &self,
        signature: &Signature,
        message: &[u8],
    ) -> ProjectivePoint {
        let R = &signature.R;
        let A = &self.point;

        let mut h = Sha512::new();
        h.update(CompressedPointY::from(R.into_affine()));
        h.update(CompressedPointY::from(A.into_affine()));
        h.update(message);

        let k = WideScalar::from_bigint(U512::from_bytes_le(
            h.finalize().as_slice(),
        ));
        let k = Scalar::from_fp(k);

        // Compute R: `-[k]A + [s]B = R`.
        self.point * (-k) + ProjectivePoint::generator() * signature.s
    }
}

impl From<ProjectivePoint> for VerifyingKey {
    fn from(point: ProjectivePoint) -> Self {
        VerifyingKey { point }
    }
}

impl From<ExpandedSecretKey> for VerifyingKey {
    fn from(value: ExpandedSecretKey) -> Self {
        let point = ProjectivePoint::generator() * value.scalar;
        point.into()
    }
}

#[cfg(test)]
mod test {
    use alloc::string::String;

    use proptest::prelude::*;

    use super::*;

    #[test]
    fn sign_and_verify_known_message() {
        let secret_key: SecretKey = [1u8; SECRET_KEY_LENGTH];
        let signing_key = SigningKey::from_bytes(&secret_key);
        let message = b"Sign me!";

        let signature = signing_key.sign(message);
        assert!(signing_key.is_valid_signature(message, &signature));

        // Verify with a different message
        let invalid_message = b"I'm not signed!";
        assert!(!signing_key.is_valid_signature(invalid_message, &signature));
    }

    #[test]
    fn sign_and_verify() {
        proptest!(|(secret_key: SecretKey, message: String)| {
            let signing_key = SigningKey::from_bytes(&secret_key);

            let signature = signing_key.sign(message.as_bytes());
            assert!(signing_key.is_valid_signature(message.as_bytes(), &signature));
        })
    }
}
