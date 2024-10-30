/// Iterates over bits in big-endian order.
pub trait BitIteratorBE {
    /// Returns an iterator over the bits of the integer, starting from the most
    /// significant bit.
    fn bit_be_iter(&self) -> impl Iterator<Item = bool>;

    /// Returns an iterator over the bits of the integer, starting from the most
    /// significant bit, and without leading zeroes.
    fn bit_be_trimmed_iter(&self) -> impl Iterator<Item = bool>;
}

impl<T: AsRef<[u64]>> BitIteratorBE for T {
    fn bit_be_iter(&self) -> impl Iterator<Item = bool> {
        self.as_ref().iter().copied().flat_map(u64_to_bits)
    }

    fn bit_be_trimmed_iter(&self) -> impl Iterator<Item = bool> {
        self.bit_be_iter().skip_while(|&b| !b)
    }
}

/// Convert u64 to bits iterator.
#[allow(clippy::module_name_repetitions)]
pub fn u64_to_bits(num: u64) -> impl Iterator<Item = bool> {
    (0..64).map(move |i| num & (1 << i) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_iterator_be() {
        let num = [0, 0b11 << 60];
        let bits: Vec<bool> = num.bit_be_trimmed_iter().collect();

        assert_eq!(bits.len(), 4);
        assert_eq!(bits, vec![true, true, false, false]);
    }
}
