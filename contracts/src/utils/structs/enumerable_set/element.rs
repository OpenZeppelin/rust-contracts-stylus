//! Element abstraction for enumerable sets.
//!
//! This module provides the foundational traits and implementations that enable
//! enumerable sets to work with different data types in Stylus smart contracts.

use alloy_primitives::{Address, B256, U128, U16, U256, U32, U64, U8};
use stylus_sdk::{
    prelude::{Erase, SimpleStorageType, StorageType},
    storage::{
        StorageAddress, StorageB256, StorageKey, StorageU128, StorageU16,
        StorageU256, StorageU32, StorageU64, StorageU8,
    },
};

/// Trait that associate set element with storage type.
pub trait Element: StorageKey + Copy {
    /// Set element type in storage.
    type StorageElement: for<'a> StorageType<Wraps<'a> = Self>
        + Accessor<Wraps = Self>
        + for<'a> SimpleStorageType<'a>
        + Erase;
}

/// Abstracts accessor inside the contract.
pub trait Accessor {
    /// Type of the number associated with the storage type.
    type Wraps;

    /// Gets underlying element [`Self::Wraps`] from persistent storage.
    fn get(&self) -> Self::Wraps;

    /// Sets underlying element [`Self::Wraps`] in persistent storage.
    fn set(&mut self, value: Self::Wraps);
}

/// Implements [`Element`] and [`Accessor`] traits for a given type.
macro_rules! impl_element_and_accessor {
    ($ty:ty, $storage_ty:ty) => {
        impl Element for $ty {
            type StorageElement = $storage_ty;
        }

        impl Accessor for $storage_ty {
            type Wraps = $ty;

            fn get(&self) -> Self::Wraps {
                self.get()
            }

            fn set(&mut self, value: Self::Wraps) {
                self.set(value);
            }
        }
    };
}

impl_element_and_accessor!(Address, StorageAddress);
impl_element_and_accessor!(B256, StorageB256);
impl_element_and_accessor!(U8, StorageU8);
impl_element_and_accessor!(U16, StorageU16);
impl_element_and_accessor!(U32, StorageU32);
impl_element_and_accessor!(U64, StorageU64);
impl_element_and_accessor!(U128, StorageU128);
impl_element_and_accessor!(U256, StorageU256);
