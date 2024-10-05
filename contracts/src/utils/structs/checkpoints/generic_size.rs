//! Contains generic size utilities for checkpoint storage contract.

use core::ops::{Add, Div, Mul, Sub};

use alloy_primitives::Uint;
use stylus_sdk::prelude::*;

/// Trait that associates types of specific size for checkpoints key and value.
pub trait Size {
    /// Type of the key in abi.
    type Key: Num;

    /// Type of the key in storage.
    type KeyStorage: for<'a> StorageType<Wraps<'a> = Self::Key>
        + Accessor<Wraps = Self::Key>;

    /// Type of the value in abi.
    type Value: Num;

    /// Type of the value in storage.
    type ValueStorage: for<'a> StorageType<Wraps<'a> = Self::Value>
        + Accessor<Wraps = Self::Value>;
}

/// Size of checkpoint storage contract with 96-bit key and 160-bit value.
pub struct S160;

impl Size for S160 {
    type Key = <Self::KeyStorage as Accessor>::Wraps;
    type KeyStorage = stylus_sdk::storage::StorageUint<96, 2>;
    type Value = <Self::ValueStorage as Accessor>::Wraps;
    type ValueStorage = stylus_sdk::storage::StorageUint<160, 3>;
}

/// Size of checkpoint storage contract with 32-bit key and 224-bit value.
pub struct S224;

impl Size for S224 {
    type Key = <Self::KeyStorage as Accessor>::Wraps;
    type KeyStorage = stylus_sdk::storage::StorageUint<32, 1>;
    type Value = <Self::ValueStorage as Accessor>::Wraps;
    type ValueStorage = stylus_sdk::storage::StorageUint<224, 4>;
}

/// Size of checkpoint storage contract with 48-bit key and 208-bit value.
pub struct S208;

impl Size for S208 {
    type Key = <Self::KeyStorage as Accessor>::Wraps;
    type KeyStorage = stylus_sdk::storage::StorageUint<48, 1>;
    type Value = <Self::ValueStorage as Accessor>::Wraps;
    type ValueStorage = stylus_sdk::storage::StorageUint<208, 4>;
}

/// Abstracts number inside the checkpoint contract.
pub trait Num: Add + Sub + Mul + Div + Ord + Sized + Copy {
    /// Zero value of the number.
    const ZERO: Self;
}

impl<const B: usize, const L: usize> Num for Uint<B, L> {
    const ZERO: Self = Self::ZERO;
}

/// Abstracts accessor inside the checkpoint contract.
pub trait Accessor {
    /// Type of the number associated with the storage type.
    type Wraps: Num;

    /// Gets underlying element [`Self::Wraps`] from persistent storage.
    fn get(&self) -> Self::Wraps;

    /// Sets underlying element [`Self::Wraps`] in persistent storage.
    fn set(&mut self, value: Self::Wraps);
}

impl<const B: usize, const L: usize> Accessor
    for stylus_sdk::storage::StorageUint<B, L>
{
    type Wraps = Uint<B, L>;

    fn get(&self) -> Self::Wraps {
        self.get()
    }

    fn set(&mut self, value: Self::Wraps) {
        self.set(value);
    }
}
