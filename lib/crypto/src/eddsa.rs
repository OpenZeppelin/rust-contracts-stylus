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
};

/// Ed25519 scalar.
pub type Scalar = Fp256<Curve25519FrParam>;

/// Ed25519 scalar used for reduction sha512 hash values to `256-bit`.
pub(crate) type WideScalar = Fp512<Curve25519Fr512Param>;

/// Scalar field parameters for curve ed25519 with `512-bit` inner integer size.
pub(crate) struct Curve25519Fr512Param;
impl FpParams<LIMBS_512> for Curve25519Fr512Param {
    const GENERATOR: Fp512<Self> = Fp512::from_fp(Curve25519FrParam::GENERATOR);
    const MODULUS: U512 = U512::from_uint(Curve25519FrParam::MODULUS);
}

/// Ed25519 projective point.
pub type ProjectivePoint = Projective<Curve25519Config>;

/// Ed25519 affine point.
pub type AffinePoint = Affine<Curve25519Config>;

/// Ed25519 secret key as defined in [RFC8032 § 5.1.5]:
///
/// The private key is 32 octets (256 bits, corresponding to b) of
/// cryptographically secure random data.
///
/// [RFC8032 § 5.1.5]: https://www.rfc-editor.org/rfc/rfc8032#section-5.1.5
pub type SecretKey = [u8; SECRET_KEY_LENGTH];

/// The length of an ed25519 [`SecretKey`] in bytes.
pub const SECRET_KEY_LENGTH: usize = 32;

/// Ed25519 public key as defined in [RFC8032 § 5.1.5].
///
/// [RFC8032 § 5.1.5]: https://www.rfc-editor.org/rfc/rfc8032#section-5.1.5
pub type PublicKey = [u8; PUBLIC_KEY_LENGTH];

/// The length of an ed25519 [`PublicKey`] in bytes.
pub const PUBLIC_KEY_LENGTH: usize = 32;

/// The length of an Ed25519 signature in bytes.
pub const SIGNATURE_LENGTH: usize = 64;

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
#[derive(Clone, PartialEq, Debug, ZeroizeOnDrop)]
pub(crate) struct ExpandedSecretKey {
    /// The secret scalar used for signing.
    pub(crate) scalar: Scalar,
    /// The domain separator used when hashing the message to generate the
    /// pseudorandom `r` value.
    pub(crate) hash_prefix: [u8; 32],
}

impl ExpandedSecretKey {
    /// Construct secret key [`Self`] from a byte string of any length.
    ///
    /// Secret key will be derived from hashed `bytes`.
    pub(crate) fn from_bytes(bytes: &[u8]) -> ExpandedSecretKey {
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
///
/// Invariant: `verifying_key` is always the public key of
/// `secret_key`.
/// This prevents the signing function [oracle attack].
///
/// [oracle attack]: https://github.com/MystenLabs/ed25519-unsafe-libs
#[derive(Clone, PartialEq)]
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

    /// Construct a [`SigningKey`] from a [`SecretKey`].
    #[inline]
    #[must_use]
    pub fn from_bytes(secret_key: &SecretKey) -> Self {
        let signing_key: ExpandedSecretKey = secret_key.into();
        let verifying_key: VerifyingKey = signing_key.clone().into();
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
    ///
    /// ```rust
    ///  use openzeppelin_crypto::eddsa::{SecretKey, SigningKey, SECRET_KEY_LENGTH};
    ///
    ///  let secret_key: SecretKey = [1u8; SECRET_KEY_LENGTH];
    ///  let signing_key = SigningKey::from_bytes(&secret_key);
    ///  let message = b"Sign me!";
    ///
    ///  let signature = signing_key.sign(message);
    ///  assert!(signing_key.is_valid_signature(message, &signature));
    /// ```
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

impl From<CompressedPointY> for [u8; 32] {
    fn from(value: CompressedPointY) -> Self {
        value.0
    }
}

impl From<AffinePoint> for CompressedPointY {
    fn from(point: AffinePoint) -> Self {
        let mut s: [u8; 32] = point
            .y
            .into_bigint()
            .into_bytes_le()
            .try_into()
            .expect("Y coordinate should be 32 bytes");

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
    /// * the nonce half of the `ExpandedSecretKey`, and
    /// * the message to be signed.
    ///
    /// This digest is then interpreted as a `Scalar` and reduced into an
    /// element in ℤ/lℤ.  The scalar is then multiplied by the distinguished
    /// basepoint to produce `R`, and `EdwardsPoint`.
    pub R: ProjectivePoint,

    /// `s` is a `Scalar`, formed by using a hash function with `512-bits`
    /// output to produce the digest of:
    ///
    /// * the `r` portion of this `Signature`,
    /// * the `PublicKey` which should be used to verify this `Signature`, and
    /// * the message to be signed.
    ///
    /// This digest is then interpreted as a `Scalar` and reduced into an
    /// element in ℤ/lℤ.
    pub s: Scalar,
}

impl Signature {
    /// Converts the signature to a 64-byte array.
    ///
    /// The first 32 bytes contain the compressed encoding of the `R` value.
    /// The last 32 bytes contain the canonical encoding of the `s` scalar.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; SIGNATURE_LENGTH] {
        let mut bytes = [0u8; 64];

        // Get the compressed representation of R
        let r_compressed = CompressedPointY::from(self.R.into_affine());
        bytes[..32].copy_from_slice(r_compressed.as_ref());

        // Get the little-endian byte representation of s
        let s_bytes = self.s.into_bigint().into_bytes_le();
        bytes[32..].copy_from_slice(&s_bytes[..32]);

        bytes
    }

    /// Construct a signature from affine point `R` and scalar `s`.
    #[must_use]
    pub fn from_affine_R_s(R: AffinePoint, s: Scalar) -> Self {
        Signature { R: R.into(), s }
    }
}

/// Ed25519 key for signature verification (public key).
#[derive(Copy, Clone, Default, PartialEq)]
pub struct VerifyingKey {
    /// Edwards point used for curve arithmetic operations.
    pub point: ProjectivePoint,
}

impl VerifyingKey {
    /// Construct verifying key from affine point.
    #[must_use]
    pub fn from_affine(point: AffinePoint) -> Self {
        VerifyingKey { point: point.into() }
    }

    /// Verify a signature on a message with this keypair's public key.
    #[must_use]
    pub fn is_valid(&self, message: &[u8], signature: &Signature) -> bool {
        let expected_r = self.compute_R(signature, message);
        expected_r == signature.R
    }

    /// Helper for verification. Computes the expected `R` component of the
    /// signature. The caller compares this to the real `R` component.
    /// This computes `H(R || A || M)` where `H` is the 512-bit hash function
    /// given by `CtxDigest` (this is SHA-512 in spec-compliant Ed25519).
    fn compute_R(
        &self,
        signature: &Signature,
        message: &[u8],
    ) -> ProjectivePoint {
        let R = signature.R;
        let A = self.point;

        let mut h = Sha512::new();
        h.update(CompressedPointY::from(R.into_affine()));
        h.update(CompressedPointY::from(A.into_affine()));
        h.update(message);

        let k = WideScalar::from_bigint(U512::from_bytes_le(
            h.finalize().as_slice(),
        ));
        let k = Scalar::from_fp(k);

        // Compute R: `-[k]A + [s]B = R`.
        A * (-k) + ProjectivePoint::generator() * signature.s
    }

    /// Convert the [`VerifyingKey`] to a compressed byte representation.
    #[inline]
    #[must_use]
    pub fn to_bytes(&self) -> PublicKey {
        CompressedPointY::from(self.point.into_affine()).into()
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

    use hex_literal::hex;
    use num_traits::Zero;
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
        proptest!(|(secret_key: SecretKey, message: String, wrong_message: String)| {
            let signing_key = SigningKey::from_bytes(&secret_key);

            let signature = signing_key.sign(message.as_bytes());
            assert!(signing_key.is_valid_signature(message.as_bytes(), &signature));

            // Verify with a different message
            if message != wrong_message{
                assert!(!signing_key.is_valid_signature(wrong_message.as_bytes(), &signature));
            }
        })
    }

    /// Rfc 8032 test case.
    struct Rfc8032TestCase {
        secret_key: &'static [u8],
        expected_public_key: &'static [u8],
        message: &'static [u8],
        expected_signature: &'static [u8],
    }

    /// Macro for creating [`Rfc8032TestCase`] test cases.
    macro_rules! test_case {
        (
            $secret_key:expr, $public_key:expr, $message:expr, $signature:expr
        ) => {
            Rfc8032TestCase {
                secret_key: &hex!($secret_key),
                expected_public_key: &hex!($public_key),
                message: &hex!($message),
                expected_signature: &hex!($signature),
            }
        };
    }

    #[test]
    fn rfc8032_known_signatures() {
        // Test vectors from RFC 8032 (https://tools.ietf.org/html/rfc8032#section-7.1)
        // and curve25519_dalek crate (https://github.com/dalek-cryptography/curve25519-dalek/blob/main/ed25519-dalek/TESTVECTORS)

        let test_cases = [
            test_case!(
                "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60",
                "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
                "",
                "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b"
            ),
            test_case!(
                "4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb",
                "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
                "72",
                "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00"
            ),
            test_case!(
                "c5aa8df43f9f837bedb7442f31dcb7b166d38535076f094b85ce3a2e0b4458f7",
                "fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025",
                "af82",
                "6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290ae67f760984dc6594a7c15e9716ed28dc027beceea1ec40a"
            ),
            test_case!(
                "0d4a05b07352a5436e180356da0ae6efa0345ff7fb1572575772e8005ed978e9",
                "e61a185bcef2613a6c7cb79763ce945d3b245d76114dd440bcf5f2dc1aa57057",
                "cbc77b",
                "d9868d52c2bebce5f3fa5a79891970f309cb6591e3e1702a70276fa97c24b3a8e58606c38c9758529da50ee31b8219cba45271c689afa60b0ea26c99db19b00c"
            ),
            test_case!(
                "6df9340c138cc188b5fe4464ebaa3f7fc206a2d55c3434707e74c9fc04e20ebb",
                "c0dac102c4533186e25dc43128472353eaabdb878b152aeb8e001f92d90233a7",
                "5f4c8989",
                "124f6fc6b0d100842769e71bd530664d888df8507df6c56dedfdb509aeb93416e26b918d38aa06305df3095697c18b2aa832eaa52edc0ae49fbae5a85e150c07"
            ),
            test_case!(
                "b780381a65edf8b78f6945e8dbec7941ac049fd4c61040cf0c324357975a293c",
                "e253af0766804b869bb1595be9765b534886bbaab8305bf50dbc7f899bfb5f01",
                "18b6bec097",
                "b2fc46ad47af464478c199e1f8be169f1be6327c7f9a0a6689371ca94caf04064a01b22aff1520abd58951341603faed768cf78ce97ae7b038abfe456aa17c09"
            ),
            test_case!(
                "78ae9effe6f245e924a7be63041146ebc670dbd3060cba67fbc6216febc44546",
                "fbcfbfa40505d7f2be444a33d185cc54e16d615260e1640b2b5087b83ee3643d",
                "89010d855972",
                "6ed629fc1d9ce9e1468755ff636d5a3f40a5d9c91afd93b79d241830f7e5fa29854b8f20cc6eecbb248dbd8d16d14e99752194e4904d09c74d639518839d2300"
            ),
            test_case!(
                "691865bfc82a1e4b574eecde4c7519093faf0cf867380234e3664645c61c5f79",
                "98a5e3a36e67aaba89888bf093de1ad963e774013b3902bfab356d8b90178a63",
                "b4a8f381e70e7a",
                "6e0af2fe55ae377a6b7a7278edfb419bd321e06d0df5e27037db8812e7e3529810fa5552f6c0020985ca17a0e02e036d7b222a24f99b77b75fdd16cb05568107"
            ),
            test_case!(
                "3b26516fb3dc88eb181b9ed73f0bcd52bcd6b4c788e4bcaf46057fd078bee073",
                "f81fb54a825fced95eb033afcd64314075abfb0abd20a970892503436f34b863",
                "4284abc51bb67235",
                "d6addec5afb0528ac17bb178d3e7f2887f9adbb1ad16e110545ef3bc57f9de2314a5c8388f723b8907be0f3ac90c6259bbe885ecc17645df3db7d488f805fa08"
            ),
            test_case!(
                "605f90b53d8e4a3b48b97d745439f2a0807d83b8502e8e2979f03e8d376ac9fe",
                "aa3fae4cfa6f6bfd14ba0afa36dcb1a2656f36541ad6b3e67f1794b06360a62f",
                "3bcdcac292ac9519024aaecee2b3e999ff5d3445e9f1eb60940f06b91275b6c5db2722ed4d82fe89605226530f3e6b0737b308cde8956184944f388a80042f6cba274c0f7d1192a0a96b0da6e2d6a61b76518fbee555773a414590a928b4cd545fccf58172f35857120eb96e75c5c8ac9ae3add367d51d34ac403446360ec10f553ea9f14fb2b8b78cba18c3e506b2f04097063a43b2d36431cce02caf11c5a4db8c821752e52985d5af1bfbf4c61572e3fadae3ad424acd81662ea5837a1143b9669391d7b9cfe230cffb3a7bb03f6591c25a4f01c0d2d4aca3e74db1997d3739c851f0327db919ff6e77f6c8a20fdd3e1594e92d01901ab9aef194fc893e70d78c8ae0f480001a515d4f9923ae6278e8927237d05db23e984c92a683882f57b1f1882a74a193ab6912ff241b9ffa662a0d47f29205f084dbde845baaeb5dd36ae6439a437642fa763b57e8dbe84e55813f0151e97e5b9de768b234b8db15c496d4bfcfa1388788972bb50ce030bc6e0ccf4fa7d00d343782f6ba8de0",
                "dd0212e63288cbe14a4569b4d891da3c7f92727c5e7f9a801cf9d6827085e7095b669d7d45f882ca5f0745dccd24d87a57181320191e5b7a47c3f7f2dccbd707"
            ),
            test_case!(
                "9e2c3d189838f4dd52ef0832886874c5ca493983ddadc07cbc570af2ee9d6209",
                "f68d3b81e73557ee1f08bd2d3f46a4718256a0f3cd8d2e03eb8fe882aab65c69",
                "19485f5238ba82eadf5eff14ca75cd42e5d56fea69d5718cfb5b1d40d760899b450e66884558f3f25b7c3de9afc4738d7ac09da5dd4689bbfac07836f5e0be432b1ddcf1b1a075bc9815d0debc865d90bd5a0c5f5604d9b46ace816c57694ecc3d40d8f84df0ede2bc4d577775a027f725de0816f563fa88f88e077720ebb6ac02574604819824db7474d4d0b22cd1bc05768e0fb867ca1c1a7b90b34ab7a41afc66957266ac0c915934aaf31c0cf6927a4f03f23285e6f24afd5813849bb08c203ac2d0336dcbf80d77f6cf7120edfbcdf181db107ec8e00f32449c1d3f5c049a92694b4ea2c6ebe5e2b0f64b5ae50ad3374d246b3270057e724a27cf263b633ab65ecb7f5c266b8007618b10ac9ac83db0febc04fd863d9661ab6e58494766f71b9a867c5a7a4555f667c1af2e54588f162a41ce756407cc4161d607b6e0682980934caa1bef036f7330d9eef01ecc553583fee5994e533a46ca916f60f8b961ae01d20f7abf0df6141b604de733c636b42018cd5f1d1ef4f84cee40fc",
                "38a31b6b465084738262a26c065fe5d9e2886bf9dd35cde05df9bad0cc7db401c750aa19e66090bce25a3c721201e60502c8c10454346648af065eab0ee7d80f"
            ),
            test_case!(
                "575f8fb6c7465e92c250caeec1786224bc3eed729e463953a394c9849cba908f",
                "71bfa98f5bea790ff183d924e6655cea08d0aafb617f46d23a17a657f0a9b8b2",
                "2cc372e25e53a138793064610e7ef25d9d7422e18e249675a72e79167f43baf452cbacb50182faf80798cc38597a44b307a536360b0bc1030f8397b94cbf147353dd2d671cb8cab219a2d7b9eb828e9635d2eab6eb08182cb03557783fd282aaf7b471747c84acf72debe4514524f8447bafccccec0a840feca9755ff9adb60301c2f25d4e3ba621df5ad72100c45d7a4b91559c725ab56bb29830e35f5a6faf87db23001f11ffba9c0c15440302065827a7d7aaaeab7b446abce333c0d30c3eae9c9da63eb1c0391d4269b12c45b660290611ac29c91dbd80dc6ed302a4d191f2923922f032ab1ac10ca7323b5241c5751c3c004ac39eb1267aa10017ed2dac6c934a250dda8cb06d5be9f563b827bf3c8d95fd7d2a7e7cc3acbee92538bd7ddfba3ab2dc9f791fac76cdf9cd6a6923534cf3e067108f6aa03e320d954085c218038a70cc768b972e49952b9fe171ee1be2a52cd469b8d36b84ee902cd9410db2777192e90070d2e7c56cb6a45f0a839c78c219203b6f1b33cb4504c6a7996427741e6874cf45c5fa5a38765a1ebf1796ce16e63ee509612c40f088cbceffa3affbc13b75a1b9c02c61a180a7e83b17884fe0ec0f2fe57c47e73a22f753eaf50fca655ebb19896b827a3474911c67853c58b4a78fd085a23239b9737ef8a7baff11ddce5f2cae0543f8b45d144ae6918b9a75293ec78ea618cd2cd08c971301cdfa0a9275c1bf441d4c1f878a2e733ce0a33b6ecdacbbf0bdb5c3643fa45a013979cd01396962897421129a88757c0d88b5ac7e44fdbd938ba4bc37de4929d53751fbb43d4e09a80e735244acada8e6749f77787f33763c7472df52934591591fb226c503c8be61a920a7d37eb1686b62216957844c43c484e58745775553",
                "903b484cb24bc503cdced844614073256c6d5aa45f1f9f62c7f22e5649212bc1d6ef9eaa617b6b835a6de2beff2faac83d37a4a5fc5cc3b556f56edde2651f02"
            ),
        ];

        for Rfc8032TestCase {
            secret_key,
            expected_public_key: public_key,
            message,
            expected_signature,
        } in test_cases
        {
            let secret_key: SecretKey = secret_key
                .try_into()
                .expect("secret key should have proper size");

            // Verify public key encoding.
            let signing_key = SigningKey::from_bytes(&secret_key);
            assert_eq!(signing_key.verifying_key.to_bytes(), public_key);

            // Verify signature validation.
            let signature = signing_key.sign(message);
            assert!(signing_key.is_valid_signature(message, &signature));

            // Verify signature byte encoding.
            let serialized_sig = signature.to_bytes();
            assert_eq!(serialized_sig.as_ref(), expected_signature);

            // Verify signature fails to validate the wrong message.
            let wrong_msg = [message, b"invalid"].concat();
            assert!(!signing_key.is_valid_signature(&wrong_msg, &signature));
        }
    }

    #[test]
    fn zeroize_signing_key() {
        let ptr = {
            let secret = SigningKey::from_bytes(&[4u8; 32]);
            &secret.signing_key as *const ExpandedSecretKey
        };
        let secret_key = unsafe { ptr.as_ref().unwrap() };

        assert!(secret_key.scalar.is_zero());
        assert_eq!(secret_key.hash_prefix, [0u8; 32])
    }
}
