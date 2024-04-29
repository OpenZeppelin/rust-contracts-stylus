//! Generic hashing support.
//!
//! This module provides a generic way to compute the [hash] of a value. It is
//! intended to be used as a replacement for [`core::hash`], which is limited
//! by the signature of [`core::hash::Hasher::finish`] returning a `u64`.
//!
//! [hash]: https://en.wikipedia.org/wiki/Hash_function

/// A hashable type.
///
/// Types implementing `Hash` are able to be [`hash`]ed with an instance of
/// [`Hasher`].
pub trait Hash {
    /// Feeds this value into the given [`Hasher`].
    fn hash<H: Hasher>(&self, state: &mut H);
}

/// A trait for hashing an arbitrary stream of bytes.
///
/// Instances of `Hasher` usually represent state that is changed while hashing
/// data.
///
/// `Hasher` provides a fairly basic interface for retrieving the generated hash
/// (with [`finish`]), and writing integers as well as slices of bytes into an
/// instance (with [`write`] and [`write_u8`] etc.). Most of the time, `Hasher`
/// instances are used in conjunction with the [`Hash`] trait.
pub trait Hasher {
    /// The output type of this hasher.
    ///
    /// For [`core::hash`] types, it's `u64`. For [`tiny_keccak`] it's `[u8]`.
    /// For this crate, it's `[u8; 32]`.
    type Output;

    /// Absorb additional input. Can be called multiple times.
    fn update(&mut self, input: &[u8]);

    /// Pad and squeeze the state to the output.
    fn finalize(self) -> Self::Output;
}

/// A trait for creating instances of [`Hasher`].
///
/// A `BuildHasher` is typically used (e.g., by [`HashMap`]) to create
/// [`Hasher`]s for each key such that they are hashed independently of one
/// another, since [`Hasher`]s contain state.
///
/// For each instance of `BuildHasher`, the [`Hasher`]s created by
/// [`build_hasher`] should be identical. That is, if the same stream of bytes
/// is fed into each hasher, the same output will also be generated.
///
/// # Examples
///
/// ```
/// use core::hash::{BuildHasher, Hasher, RandomState};
///
/// let s = RandomState::new();
/// let mut hasher_1 = s.build_hasher();
/// let mut hasher_2 = s.build_hasher();
///
/// hasher_1.write_u32(8128);
/// hasher_2.write_u32(8128);
///
/// assert_eq!(hasher_1.finish(), hasher_2.finish());
/// ```
///
/// [`build_hasher`]: BuildHasher::build_hasher
/// [`HashMap`]: ../../std/collections/struct.HashMap.html
pub trait BuildHasher {
    /// Type of the hasher that will be created.
    type Hasher: Hasher;

    /// Creates a new hasher.
    ///
    /// Each call to `build_hasher` on the same instance should produce
    /// identical [`Hasher`]s.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::hash::{BuildHasher, RandomState};
    ///
    /// let s = RandomState::new();
    /// let new_s = s.build_hasher();
    /// ```
    fn build_hasher(&self) -> Self::Hasher;

    /// Calculates the hash of a single value.
    ///
    /// This is intended as a convenience for code which *consumes* hashes, such
    /// as the implementation of a hash table or in unit tests that check
    /// whether a custom [`Hash`] implementation behaves as expected.
    ///
    /// This must not be used in any code which *creates* hashes, such as in an
    /// implementation of [`Hash`].  The way to create a combined hash of
    /// multiple values is to call [`Hash::hash`] multiple times using the same
    /// [`Hasher`], not to call this method repeatedly and combine the results.
    fn hash_one<Hashable>(
        &self,
        h: Hashable,
    ) -> <Self::Hasher as Hasher>::Output
    where
        Hashable: Hash,
        Self: Sized,
        Self::Hasher: Hasher,
    {
        let mut hasher = self.build_hasher();
        h.hash(&mut hasher);
        hasher.finalize()
    }
}
