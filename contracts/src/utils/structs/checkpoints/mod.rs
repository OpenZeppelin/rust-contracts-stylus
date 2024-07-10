//! Contract for checkpointing values as they change at different points in
//! time, to looking up past values by block number later.
//!
//! To create a history of checkpoints, define a variable type [`trace::Trace`]
//! in your contract.
//! Types [`S160`], [`S160`] and [`S160`] can be used to
//! define sizes for key and value.
//! Then store a new checkpoint for the current
//! transaction block using the [`trace::Trace::push`] function.
pub mod trace;

use core::ops::{Add, Div, Mul, Sub};

use alloy_primitives::Uint;
use stylus_sdk::prelude::*;

/// Trait that associates types of specific size for checkpoints key and value.
pub trait Size {
    /// Type of the key in abi.
    type Key: Num;

    /// Type of the key in storage.
    type KeyStorage: for<'a> StorageType<Wraps<'a> = Self::Key>
        + Accessor<Wrap = Self::Key>;

    /// Type of the value in abi.
    type Value: Num;

    /// Type of the value in storage.
    type ValueStorage: for<'a> StorageType<Wraps<'a> = Self::Value>
        + Accessor<Wrap = Self::Value>;
}

/// Size of checkpoint storage contract corresponding to the size of 96 bits of
/// the key and size 160 bits of the value.
pub type S160 = SpecificSize<96, 2, 160, 3>;

/// Size of checkpoint storage contract corresponding to the size of 32 bits of
/// the key and size 224 bits of the value.
pub type S224 = SpecificSize<32, 1, 224, 4>;

/// Size of checkpoint storage contract corresponding to the size of 48 bits of
/// the key and size 208 bits of the value.
pub type S208 = SpecificSize<48, 1, 208, 4>;

/// Contains the size of checkpoint's key and value in bits.
pub struct SpecificSize<
    const KEY_BITS: usize,
    const KEY_LIMBS: usize,
    const VALUE_BITS: usize,
    const VALUE_LIMBS: usize,
>;

impl<const KB: usize, const KL: usize, const VB: usize, const VL: usize> Size
    for SpecificSize<KB, KL, VB, VL>
{
    type Key = Uint<KB, KL>;
    type KeyStorage = stylus_sdk::storage::StorageUint<KB, KL>;
    type Value = Uint<VB, VL>;
    type ValueStorage = stylus_sdk::storage::StorageUint<VB, VL>;
}

/// Abstracts number inside the checkpoint contract.
pub trait Num: Add + Sub + Mul + Div + Ord + Sized + Copy {
    /// Zero value of the number.
    const ZERO: Self;
}

impl<const B: usize, const L: usize> Num for Uint<B, L> {
    const ZERO: Self = Self::ZERO;
}

/// Abstracts accessor inside the checkpoint contract
pub trait Accessor {
    /// Type of the number associated with the storage type.
    type Wrap: Num;

    /// Gets underlying element [`Self::Wrap`] from persistent storage.
    fn get(&self) -> Self::Wrap;

    /// Sets underlying element [`Self::Wrap`] in persistent storage.
    fn set(&mut self, value: Self::Wrap);
}

impl<const B: usize, const L: usize> Accessor
    for stylus_sdk::storage::StorageUint<B, L>
{
    type Wrap = Uint<B, L>;

    fn get(&self) -> Self::Wrap {
        self.get()
    }

    fn set(&mut self, value: Self::Wrap) {
        self.set(value);
    }
}
