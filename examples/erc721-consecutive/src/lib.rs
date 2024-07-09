#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use alloy_sol_types::SolError;
use openzeppelin_stylus::token::erc721::extensions::{
    consecutive::Erc721Consecutive, IErc721Burnable,
};
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    struct Erc721ConsecutiveExample {
        #[borrow]
        Erc721Consecutive erc721_consecutive;
    }
}

#[external]
#[inherit(Erc721Consecutive)]
impl Erc721ConsecutiveExample {
    // Burn token with `token_id` and record it at [`Self::sequentian_burn`]
    // storage.
    pub fn burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        self.erc721_consecutive.burn(token_id)?;
    }

    // Mint a batch of tokens of length `batchSize` for `to`. Returns the token
    // id of the first token minted in the batch; if `batchSize` is 0,
    // returns the number of consecutive ids minted so far.
    //
    // Requirements:
    //
    // - `batchSize` must not be greater than [`MAX_BATCH_SIZE`].
    // - The function is called in the constructor of the contract (directly or
    //   indirectly).
    //
    // CAUTION: Does not emit a `Transfer` event. This is ERC-721 compliant as
    // long as it is done inside of the constructor, which is enforced by
    // this function.
    //
    // CAUTION: Does not invoke `onERC721Received` on the receiver.
    //
    // Emits a [`ConsecutiveTransfer`] event.
    pub fn mint_consecutive(
        &mut self,
        to: Address,
        batch_size: u128, // TODO: how to use U96 type
    ) -> Result<u128, Vec<u8>> {
        // TODO#q: polish initialization logic
        Ok(self.erc721_consecutive.mint_consecutive(to, batch_size)?)
    }
}
