//! Contains generic size utilities for checkpoint storage contract.

use core::ops::{Add, Div, Mul, Sub};

use alloy_sol_types::sol_data::{IntBitCount, SupportedInt};
use stylus_sdk::{alloy_primitives::Uint, prelude::*};

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

/// Defines size of checkpoint storage contract with specific key and value
/// bits.
///
/// # Arguments
///
/// * `$name` - Identifier of the typed size.
/// * `$key_bits` - Number of bits in checkpoint's key.
/// * `$value_bits` - Number of bits in checkpoint's value.
macro_rules! define_checkpoint_size {
    ($name:ident, $key_bits:expr, $value_bits:expr) => {
        #[doc = "Size of checkpoint storage contract with"]
        #[doc = stringify!($key_bits)]
        #[doc = "bit key and "]
        #[doc = stringify!($value_bits)]
        #[doc = "bit value."]
        pub struct $name;

        impl Size for $name {
            type Key = stylus_sdk::alloy_primitives::Uint<
                $key_bits,
                { usize::div_ceil($key_bits, 64) },
            >;
            type KeyStorage = stylus_sdk::storage::StorageUint<
                $key_bits,
                { usize::div_ceil($key_bits, 64) },
            >;
            type Value = stylus_sdk::alloy_primitives::Uint<
                $value_bits,
                { usize::div_ceil($value_bits, 64) },
            >;
            type ValueStorage = stylus_sdk::storage::StorageUint<
                $value_bits,
                { usize::div_ceil($value_bits, 64) },
            >;
        }
    };
}

define_checkpoint_size!(S160, 96, 160);
define_checkpoint_size!(S224, 32, 224);
define_checkpoint_size!(S208, 48, 208);

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
where
    IntBitCount<B>: SupportedInt,
{
    type Wraps = Uint<B, L>;

    fn get(&self) -> Self::Wraps {
        self.get()
    }

    fn set(&mut self, value: Self::Wraps) {
        self.set(value);
    }
}
