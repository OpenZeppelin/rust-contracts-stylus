#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{uint, Address, U128, U256};
use alloy_sol_types::SolError;
use openzeppelin_stylus::{
    token::erc721::{
        extensions::IErc721Burnable, ERC721InvalidReceiver, Erc721,
    },
    utils::structs::{
        bitmap::BitMap,
        checkpoints::{Trace160, U160, U96},
    },
};
use stylus_sdk::{alloy_sol_types::sol, evm, prelude::*};

sol_storage! {
    #[entrypoint]
    struct Erc721ConsecutiveExample {
        #[borrow]
        Erc721 erc721;
        Trace160 sequential_ownership;
        BitMap sequentian_burn;
    }
}

sol! {
    /// Emitted when the tokens from `fromTokenId` to `toTokenId` are transferred from `fromAddress` to `toAddress`.
    event ConsecutiveTransfer(
        uint256 indexed fromTokenId,
        uint256 toTokenId,
        address indexed fromAddress,
        address indexed toAddress
    );
}

sol! {
    /// Batch mint is restricted to the constructor.
    /// Any batch mint not emitting the {IERC721-Transfer} event outside of the constructor
    /// is non ERC-721 compliant.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC721ForbiddenBatchMint();

    /// Exceeds the max amount of mints per batch.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC721ExceededMaxBatchMint(uint256 batchSize, uint256 maxBatch);

    /// Individual minting is not allowed.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC721ForbiddenMint();

    /// Batch burn is not supported.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC721ForbiddenBatchBurn();
}

// Maximum size of a batch of consecutive tokens. This is designed to limit
// stress on off-chain indexing services that have to record one entry per
// token, and have protections against "unreasonably large" batches of tokens.
const MAX_BATCH_SIZE: U96 = uint!(5000_U96);

// Used to offset the first token id in {_nextConsecutiveId}
const FIRST_CONSECUTIVE_ID: U96 = uint!(0_U96);

#[external]
#[inherit(Erc721)]
impl Erc721ConsecutiveExample {
    // Burn token with `token_id` and record it at [`Self::sequentian_burn`]
    // storage.
    pub fn burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        self.erc721.burn(token_id)?;

        // record burn
        if token_id < self.next_consecutive_id().to() // the tokenId was minted in a batch
            && !self.sequentian_burn.get(token_id)
        // and the token was never marked as burnt
        {
            self.sequentian_burn.set(token_id)
        }
        Ok(())
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
        let batch_size = U96::from(batch_size);
        let next = self.next_consecutive_id();

        if batch_size > U96::ZERO {
            if to.is_zero() {
                return Err(
                    ERC721InvalidReceiver { receiver: Address::ZERO }.encode()
                );
            }

            if batch_size > MAX_BATCH_SIZE.to() {
                return Err(ERC721ExceededMaxBatchMint {
                    batchSize: U256::from(batch_size),
                    maxBatch: U256::from(MAX_BATCH_SIZE),
                }
                .encode());
            }

            let last = next + batch_size - uint!(1_U96);
            self.sequential_ownership
                .push(last, U160::from_be_bytes(to.into_array()))
                .map_err(Vec::<u8>::from)?;

            self.erc721._increase_balance(to, U128::from(batch_size));
            evm::log(ConsecutiveTransfer {
                fromTokenId: next.to::<U256>(),
                toTokenId: last.to::<U256>(),
                fromAddress: Address::ZERO,
                toAddress: to,
            });
        };
        Ok(next.to())
    }
}

impl Erc721ConsecutiveExample {
    /// Returns the next tokenId to mint using {_mintConsecutive}. It will
    /// return [`FIRST_CONSECUTIVE_ID`] if no consecutive tokenId has been
    /// minted before.
    fn next_consecutive_id(&self) -> U96 {
        match self.sequential_ownership.latest_checkpoint() {
            None => FIRST_CONSECUTIVE_ID,
            Some((latest_id, _)) => latest_id + uint!(1_U96),
        }
    }
}
