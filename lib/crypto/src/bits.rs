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
    use num_traits::ConstOne;
    use proptest::prelude::*;

    use super::*;

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

    macro_rules! trimmed_is_subset_of_full {
        ($ty:ident) => {{
            proptest!(|(value: $ty)| {
                let full: Vec<bool> = value.bit_be_iter().collect();
                let trimmed: Vec<bool> = value.bit_be_trimmed_iter().collect();
                let start_idx = value.leading_zeros() as usize;
                prop_assert_eq!(&full[start_idx..], trimmed);
            });
        }};
    }

    #[test]
    fn trimmed_is_subset_of_full() {
        trimmed_is_subset_of_full!(u8);
        trimmed_is_subset_of_full!(u16);
        trimmed_is_subset_of_full!(u32);
        trimmed_is_subset_of_full!(u64);
        trimmed_is_subset_of_full!(u128);
        trimmed_is_subset_of_full!(usize);
    }

    macro_rules! edge_case {
        ($ty:ident) => {{
            assert_eq!($ty::MIN.bit_be_trimmed_iter().count(), 0);
            assert_eq!($ty::MIN.bit_be_iter().count(), $ty::BITS as usize);
            assert_eq!(
                $ty::MAX.bit_be_trimmed_iter().count(),
                $ty::BITS as usize
            );
            assert_eq!($ty::MAX.bit_be_iter().count(), $ty::BITS as usize);
            assert_eq!($ty::ONE.bit_be_trimmed_iter().count(), usize::ONE);
        }};
    }

    #[test]
    fn edge_cases() {
        edge_case!(u8);
        edge_case!(u16);
        edge_case!(u32);
        edge_case!(u64);
        edge_case!(u128);
        edge_case!(usize);
    }
}
