use ruint::Uint;

pub trait BitIterator {
    /// Returns an iterator over the bits of the integer, starting from the most
    /// significant bit.
    fn bit_be_iter(&self) -> impl Iterator<Item = bool>;

    /// Returns an iterator over the bits of the integer, starting from the most
    /// significant bit, and without leading zeroes.
    fn bit_be_trimmed_iter(&self) -> impl Iterator<Item = bool>;

    /// Returns an iterator over the bits of the integer, starting from the
    /// least significant bit.
    fn bit_le_iter(&self) -> impl Iterator<Item = bool>;

    /// Returns an iterator over the bits of the integer, starting from the
    /// least significant bit, and without trailing zeroes.
    fn bit_le_trimmed_iter(&self) -> impl Iterator<Item = bool>;
}

impl<const BITS: usize, const LIMBS: usize> BitIterator for Uint<BITS, LIMBS> {
    fn bit_be_iter(&self) -> impl Iterator<Item = bool> {
        let be_bytes = self.to_be_bytes_vec();
        bytes_to_bits(be_bytes)
    }

    fn bit_be_trimmed_iter(&self) -> impl Iterator<Item = bool> {
        let be_bytes = self.to_be_bytes_trimmed_vec();
        bytes_to_bits(be_bytes)
    }

    fn bit_le_iter(&self) -> impl Iterator<Item = bool> {
        let le_bytes = self.to_le_bytes_vec();
        bytes_to_bits(le_bytes)
    }

    fn bit_le_trimmed_iter(&self) -> impl Iterator<Item = bool> {
        let le_bytes = self.to_le_bytes_trimmed_vec();
        bytes_to_bits(le_bytes)
    }
}

/// Convert an array of bit to vector of bytes.
pub fn bits_to_bytes(bits: &[bool]) -> impl Iterator<Item = u8> + '_ {
    bits.chunks(8).map(|chunk| {
        chunk
            .iter()
            .enumerate()
            .fold(0, |acc, (i, &bit)| acc | ((bit as u8) << i))
    })
}

/// Convert an array of bytes to vector of bits.
pub fn bytes_to_bits(bytes: Vec<u8>) -> impl Iterator<Item = bool> {
    bytes.into_iter().flat_map(|byte| (0..8).map(move |i| byte & (1 << i) == 1))
}

// TODO#q: implement bit iterator for AsRef<[u64]>

// TODO#q: tests
