#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc721::{
    extensions::{
        burnable::{Erc721Burnable, Erc721BurnableOverride},
        metadata::Erc721Metadata,
        uri_storage::Erc721UriStorage,
    },
    Erc721, Erc721Override, Error, IErc721Virtual,
};
use openzeppelin_stylus_proc::r#override;
use stylus_sdk::prelude::{entrypoint, external, sol_storage, TopLevelStorage};

sol_storage! {
    #[entrypoint]
    struct Erc721MetadataExample {
        Erc721<Override> erc721;
        Erc721Metadata metadata;
        Erc721UriStorage<Override> uri_storage;
    }
}

#[external]
#[inherit(Erc721UriStorage<Override>)]
#[inherit(Erc721Metadata)]
#[inherit(Erc721Burnable<Override>)]
#[inherit(Erc721<Override>)]
impl Erc721MetadataExample {
    pub fn mint(
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        Erc721::<Override>::_mint(storage, to, token_id)
    }
}

#[r#override]
#[inherit(Erc721BurnableOverride)]
#[inherit(Erc721Override)]
impl IErc721Virtual for Erc721MetadataExampleOverride {}
