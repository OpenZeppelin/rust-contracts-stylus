use tiny_keccak::{Hasher as TinyHasher, Keccak};

use super::hash::{BuildHasher, Hasher};

#[derive(Default)]
pub struct KeccakBuilder;

impl BuildHasher for KeccakBuilder {
    type Hasher = Keccak256;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Keccak256(Keccak::v256())
    }
}

pub struct Keccak256(Keccak);

impl Hasher for Keccak256 {
    type Output = [u8; 32];

    fn update(&mut self, input: &[u8]) {
        self.0.update(input)
    }

    fn finalize(self) -> Self::Output {
        let mut buffer = [0u8; 32];
        self.0.finalize(&mut buffer);
        buffer
    }
}
