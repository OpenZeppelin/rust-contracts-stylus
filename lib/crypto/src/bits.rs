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
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn trimmed_iter_starts_with_one() {
        proptest!(|(value in 1u64..)| {
            let bits: Vec<bool> = value.bit_be_trimmed_iter().collect();
            prop_assert!(!bits.is_empty());
            prop_assert!(bits[0]);
        })
    }

    #[test]
    fn trimmed_is_subset_of_full() {
        proptest!(|(value: u64)| {
            let full: Vec<bool> = value.bit_be_iter().collect();
            let trimmed: Vec<bool> = value.bit_be_trimmed_iter().collect();
            let start_idx = value.leading_zeros() as usize;
            prop_assert_eq!(&full[start_idx..], trimmed);
        })
    }

    #[test]
    fn bit_be_iter_has_full_width_for_minimum_values() {
        assert_eq!(u8::MIN.bit_be_trimmed_iter().count(), 0);
        assert_eq!(u16::MIN.bit_be_trimmed_iter().count(), 0);
        assert_eq!(u32::MIN.bit_be_trimmed_iter().count(), 0);
        assert_eq!(u64::MIN.bit_be_trimmed_iter().count(), 0);
        assert_eq!(u128::MIN.bit_be_trimmed_iter().count(), 0);
        assert_eq!(usize::MIN.bit_be_trimmed_iter().count(), 0);
    }

    #[test]
    fn bit_be_iter_has_full_width_for_maximum_values() {
        assert_eq!(u8::MAX.bit_be_trimmed_iter().count(), 8);
        assert_eq!(u16::MAX.bit_be_trimmed_iter().count(), 16);
        assert_eq!(u32::MAX.bit_be_trimmed_iter().count(), 32);
        assert_eq!(u64::MAX.bit_be_trimmed_iter().count(), 64);
        assert_eq!(u128::MAX.bit_be_trimmed_iter().count(), 128);
        assert_eq!(
            usize::MAX.bit_be_trimmed_iter().count(),
            usize::BITS as usize
        );
    }

    #[test]
    fn bit_be_trimmed_iter_is_empty_for_minimum_values() {
        assert_eq!(u8::MIN.bit_be_trimmed_iter().count(), 0);
        assert_eq!(u16::MIN.bit_be_trimmed_iter().count(), 0);
        assert_eq!(u32::MIN.bit_be_trimmed_iter().count(), 0);
        assert_eq!(u64::MIN.bit_be_trimmed_iter().count(), 0);
        assert_eq!(u128::MIN.bit_be_trimmed_iter().count(), 0);
        assert_eq!(usize::MIN.bit_be_trimmed_iter().count(), 0);
    }

    #[test]
    fn bit_be_trimmed_iter_has_full_width_for_maximum_values() {
        assert_eq!(u8::MAX.bit_be_trimmed_iter().count(), 8);
        assert_eq!(u16::MAX.bit_be_trimmed_iter().count(), 16);
        assert_eq!(u32::MAX.bit_be_trimmed_iter().count(), 32);
        assert_eq!(u64::MAX.bit_be_trimmed_iter().count(), 64);
        assert_eq!(u128::MAX.bit_be_trimmed_iter().count(), 128);
        assert_eq!(
            usize::MAX.bit_be_trimmed_iter().count(),
            usize::BITS as usize
        );
    }

    #[test]
    fn zero_value() {
        let zero = 0u64;
        assert!(zero.bit_be_iter().all(|b| !b));
        assert_eq!(zero.bit_be_iter().count(), 64);
        assert_eq!(zero.bit_be_trimmed_iter().count(), 0);
    }

    #[test]
    fn one_value() {
        let one = 1u64;
        let full: Vec<_> = one.bit_be_iter().collect();
        assert_eq!(full.len(), 64);
        assert_eq!(full.iter().filter(|&&b| b).count(), 1);
        assert!(full.last().copied().unwrap());

        assert_eq!(one.bit_be_trimmed_iter().collect::<Vec<_>>(), vec![true]);
    }

    #[test]
    fn known_pattern() {
        let value = 0b1100u64;
        let full = value.bit_be_iter().collect::<Vec<_>>();
        let expected = [false; 60]
            .iter()
            .chain(&[true, true, false, false])
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(full, expected);

        let trimmed = value.bit_be_trimmed_iter().collect::<Vec<_>>();
        let expected = vec![true, true, false, false];
        assert_eq!(trimmed, expected);
    }

    #[test]
    fn max_value() {
        let max = u64::MAX;
        assert!(max.bit_be_iter().all(|b| b));
        assert!(max.bit_be_trimmed_iter().all(|b| b));
        assert_eq!(max.bit_be_iter().count(), 64);
        assert_eq!(max.bit_be_trimmed_iter().count(), 64);
    }
}
