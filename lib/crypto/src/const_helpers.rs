#[macro_export]
macro_rules! const_for {
    (($i:ident in $start:tt..$end:tt)  $code:expr ) => {{
        let mut $i = $start;
        while $i < $end {
            $code
            $i += 1;
        }
    }};
}

#[macro_export]
macro_rules! sbb {
    ($a:expr, $b:expr, &mut $borrow:expr $(,)?) => {{
        let tmp =
            (1u128 << 64) + ($a as u128) - ($b as u128) - ($borrow as u128);
        $borrow = if tmp >> 64 == 0 { 1 } else { 0 };
        tmp as u64
    }};
}

#[macro_export]
macro_rules! adc {
    ($a:expr, $b:expr, &mut $carry:expr $(,)?) => {{
        let tmp = ($a as u128) + ($b as u128) + ($carry as u128);
        $carry = (tmp >> 64) as u64;
        tmp as u64
    }};
}

// TODO#q: implement as a function
#[macro_export]
macro_rules! const_modulo {
    ($a:expr, $divisor:expr) => {{
        // Stupid slow base-2 long division taken from
        // https://en.wikipedia.org/wiki/Division_algorithm
        assert!(!$divisor.const_is_zero());
        let mut remainder = Self::new([0u64; N]);
        let mut i = ($a.num_bits() - 1) as isize;
        let mut carry;
        while i >= 0 {
            (remainder, carry) = remainder.const_mul2_with_carry();
            remainder.0[0] |= $a.get_bit(i as usize) as u64;
            if remainder.const_geq($divisor) || carry {
                let (r, borrow) = remainder.const_sub_with_borrow($divisor);
                remainder = r;
                assert!(borrow == carry);
            }
            i -= 1;
        }
        remainder
    }};
}

pub(super) struct RBuffer<const N: usize>(pub [u64; N], pub u64);

impl<const N: usize> RBuffer<N> {
    /// Find the number of bits in the binary decomposition of `self`.
    pub(super) const fn num_bits(&self) -> u32 {
        (N * 64) as u32 + (64 - self.1.leading_zeros())
    }

    /// Returns the `i`-th bit where bit 0 is the least significant one.
    /// In other words, the bit with weight `2^i`.
    pub(super) const fn get_bit(&self, i: usize) -> bool {
        let d = i / 64;
        let b = i % 64;
        if d == N {
            (self.1 >> b) & 1 == 1
        } else {
            (self.0[d] >> b) & 1 == 1
        }
    }
}

pub(super) struct R2Buffer<const N: usize>(pub [u64; N], pub [u64; N], pub u64);

impl<const N: usize> R2Buffer<N> {
    /// Find the number of bits in the binary decomposition of `self`.
    pub(super) const fn num_bits(&self) -> u32 {
        ((2 * N) * 64) as u32 + (64 - self.2.leading_zeros())
    }

    /// Returns the `i`-th bit where bit 0 is the least significant one.
    /// In other words, the bit with weight `2^i`.
    pub(super) const fn get_bit(&self, i: usize) -> bool {
        let d = i / 64;
        let b = i % 64;
        if d == 2 * N {
            (self.2 >> b) & 1 == 1
        } else if d >= N {
            (self.1[d - N] >> b) & 1 == 1
        } else {
            (self.0[d] >> b) & 1 == 1
        }
    }
}
