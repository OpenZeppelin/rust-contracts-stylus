use core::ops::{Index, IndexMut};

use ark_serialize::{Read, Write};
use ruint::Uint;

/// A buffer to hold values of size 8 * N + 1 bytes. This is mostly
/// a hack that's necessary until `generic_const_exprs` is stable.
#[derive(Copy, Clone)]
#[repr(C, align(1))]
pub(super) struct SerBuffer<const LIMBS: usize> {
    pub(super) buffers: [[u8; 8]; LIMBS],
    pub(super) last: u8,
}

impl<const LIMBS: usize> SerBuffer<LIMBS> {
    pub(super) const fn zeroed() -> Self {
        Self { buffers: [[0u8; 8]; LIMBS], last: 0u8 }
    }

    #[inline(always)]
    pub(super) const fn get(&self, index: usize) -> &u8 {
        if index == 8 * LIMBS {
            &self.last
        } else {
            let part = index / 8;
            let in_buffer_index = index % 8;
            &self.buffers[part][in_buffer_index]
        }
    }

    #[inline(always)]
    pub(super) fn get_mut(&mut self, index: usize) -> &mut u8 {
        if index == 8 * LIMBS {
            &mut self.last
        } else {
            let part = index / 8;
            let in_buffer_index = index % 8;
            &mut self.buffers[part][in_buffer_index]
        }
    }

    #[allow(unsafe_code)]
    pub(super) fn as_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                8 * LIMBS + 1,
            )
        }
    }

    #[inline(always)]
    pub(super) fn last_n_plus_1_bytes_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut u8> {
        self.buffers[LIMBS - 1]
            .iter_mut()
            .chain(ark_std::iter::once(&mut self.last))
    }

    #[inline(always)]
    pub(super) fn copy_from_u8_slice(&mut self, other: &[u8]) {
        other.chunks(8).enumerate().for_each(|(i, chunk)| {
            if i < LIMBS {
                self.buffers[i][..chunk.len()].copy_from_slice(chunk);
            } else {
                self.last = chunk[0]
            }
        });
    }

    #[inline(always)]
    pub(super) fn copy_from_u64_slice(&mut self, other: &[u64; LIMBS]) {
        other
            .iter()
            .zip(&mut self.buffers)
            .for_each(|(other, this)| *this = other.to_le_bytes());
    }

    /// Convert the buffer to a `Uint`.
    /// Panics if `Uint` is too small.
    #[inline(always)]
    pub(super) fn to_bigint<const BITS: usize>(self) -> Uint<BITS, LIMBS> {
        let mut limbs = [0_u64; LIMBS];
        limbs
            .iter_mut()
            .zip(self.buffers)
            .for_each(|(other, this)| *other = u64::from_le_bytes(this));
        Uint::from_limbs(limbs)
    }

    #[inline(always)]
    /// Write up to `num_bytes` bytes from `self` to `other`.
    /// `num_bytes` is allowed to range from `8 * (N - 1) + 1` to `8 * N + 1`.
    pub(super) fn write_up_to(
        &self,
        mut other: impl Write,
        num_bytes: usize,
    ) -> ark_std::io::Result<()> {
        debug_assert!(num_bytes <= 8 * LIMBS + 1, "index too large");
        debug_assert!(num_bytes > 8 * (LIMBS - 1), "index too small");
        // unconditionally write first `N - 1` limbs.
        for i in 0..(LIMBS - 1) {
            other.write_all(&self.buffers[i])?;
        }
        // for the `N`-th limb, depending on `index`, we can write anywhere from
        // 1 to all bytes.
        let remaining_bytes = num_bytes - (8 * (LIMBS - 1));
        let write_last_byte = remaining_bytes > 8;
        let num_last_limb_bytes = ark_std::cmp::min(8, remaining_bytes);
        other.write_all(&self.buffers[LIMBS - 1][..num_last_limb_bytes])?;
        if write_last_byte {
            other.write_all(&[self.last])?;
        }
        Ok(())
    }

    #[inline(always)]
    /// Read up to `num_bytes` bytes from `other` to `self`.
    /// `num_bytes` is allowed to range from `8 * (N - 1)` to `8 * N + 1`.
    pub(super) fn read_exact_up_to(
        &mut self,
        mut other: impl Read,
        num_bytes: usize,
    ) -> ark_std::io::Result<()> {
        debug_assert!(num_bytes <= 8 * LIMBS + 1, "index too large");
        debug_assert!(num_bytes > 8 * (LIMBS - 1), "index too small");
        // unconditionally write first `N - 1` limbs.
        for i in 0..(LIMBS - 1) {
            other.read_exact(&mut self.buffers[i])?;
        }
        // for the `N`-th limb, depending on `index`, we can write anywhere from
        // 1 to all bytes.
        let remaining_bytes = num_bytes - (8 * (LIMBS - 1));
        let write_last_byte = remaining_bytes > 8;
        let num_last_limb_bytes = ark_std::cmp::min(8, remaining_bytes);
        other
            .read_exact(&mut self.buffers[LIMBS - 1][..num_last_limb_bytes])?;
        if write_last_byte {
            let mut last = [0u8; 1];
            other.read_exact(&mut last)?;
            self.last = last[0];
        }
        Ok(())
    }
}

impl<const N: usize> Index<usize> for SerBuffer<N> {
    type Output = u8;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl<const N: usize> IndexMut<usize> for SerBuffer<N> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
    }
}
