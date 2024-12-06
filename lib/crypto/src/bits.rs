//! Bit manipulation utilities.

/// Iterates over bits in big-endian order.
pub trait BitIteratorBE {
    /// Returns an iterator over the bits of the integer, starting from the most
    /// significant bit.
    fn bit_be_iter(&self) -> impl Iterator<Item = bool>;

    /// Returns an iterator over the bits of the integer, starting from the most
    /// significant bit, and without leading zeroes.
    fn bit_be_trimmed_iter(&self) -> impl Iterator<Item = bool> {
        self.bit_be_iter().skip_while(|&b| !b)
    }
}

macro_rules! impl_bit_iter_be {
    ($int:ty) => {
        impl BitIteratorBE for $int {
            fn bit_be_iter(&self) -> impl Iterator<Item = bool> {
                (0..<$int>::BITS).rev().map(move |i| self & (1 << i) != 0)
            }
        }
    };
}

impl_bit_iter_be!(u8);
impl_bit_iter_be!(u16);
impl_bit_iter_be!(u32);
impl_bit_iter_be!(u64);
impl_bit_iter_be!(u128);
impl_bit_iter_be!(usize);

#[cfg(all(test, feature = "std"))]
mod tests {

    use super::*;

    #[test]
    fn u64_bit_iterator_be() {
        let num: u64 = 0b1100;
        let bits: Vec<bool> = num.bit_be_trimmed_iter().collect();

        assert_eq!(bits.len(), 4);
        assert_eq!(bits, vec![true, true, false, false]);
    }
}
