#[macro_export]
macro_rules! const_for {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
        }
    }};
}

#[macro_export]
macro_rules! unroll2_for {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
        }
    }};
}

#[macro_export]
macro_rules! unroll4_for {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
        }
    }};
}

#[macro_export]
macro_rules! unroll6_for {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
        }
    }};
}

#[macro_export]
macro_rules! unroll8_for {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
        }
    }};
}

#[macro_export]
macro_rules! cycle {
    ($i:ident, $end:tt, $code:expr) => {{
        if $i < $end {
            $code
        } else {
            break;
        }
        $i += 1;
    }};
}

// TODO#q: implement const_modulo as a function
#[macro_export]
macro_rules! const_modulo {
    ($a:expr, $divisor:expr) => {{
        // Stupid slow base-2 long division taken from
        // https://en.wikipedia.org/wiki/Division_algorithm
        assert!(!$divisor.ct_is_zero());
        let mut remainder = Self::new([0u64; N]);
        let mut i = ($a.num_bits() - 1) as isize;
        let mut carry;
        while i >= 0 {
            (remainder, carry) = remainder.ct_mul2_with_carry();
            remainder.limbs[0] |= $a.get_bit(i as usize) as u64;
            if remainder.ct_geq($divisor) || carry {
                let (r, borrow) = remainder.ct_sub_with_borrow($divisor);
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
