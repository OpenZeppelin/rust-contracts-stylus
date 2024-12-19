#![cfg(feature = "e2e")]

use abi::Erc4262;
use alloy::{
    primitives::{b256, keccak256, Address, B256, U256},
    sol,
    sol_types::SolType,
};
use alloy_primitives::uint;
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt, Revert};
use eyre::Result;

mod abi;
