//! Edwards Digital Signature Algorithm (EdDSA) over Curve25519.
//!
//! These functions can be used to implement ed25519 key generation,
//! signing, and verification.
use alloc::vec::Vec;

use ed25519_zebra::{
    ed25519::PublicKeyBytes, Signature, SigningKey, VerificationKey,
};

type Seed = [u8; 32];
type Public = PublicKeyBytes;

/// A key pair.
#[derive(Copy, Clone)]
pub struct Pair {
    public: VerificationKey,
    secret: SigningKey,
}

impl Pair {
    /// Get the seed for this key.
    pub fn seed(&self) -> Seed {
        self.secret.into()
    }

    /// Return a vec filled with raw data.
    fn to_raw_vec(&self) -> Vec<u8> {
        self.seed().to_vec()
    }

    fn sign(&self, message: &[u8]) -> Signature {
        self.secret.sign(message)
    }

    fn verify<M: AsRef<[u8]>>(
        sig: &Signature,
        message: M,
        public: &Public,
    ) -> bool {
        let Ok(public) = VerificationKey::try_from(public.to_bytes()) else {
            return false;
        };
        public.verify(&sig, message.as_ref()).is_ok()
    }
}
