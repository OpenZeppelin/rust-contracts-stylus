#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc721::{
    base::{Erc721, Erc721Override, IErc721Virtual},
    extensions::{
        burnable::{Erc721Burnable, Erc721BurnableOverride},
        pausable::{Erc721Pausable, Erc721PausableOverride},
    },
    Error,
};
use openzeppelin_stylus_proc::r#override;
use stylus_sdk::{alloy_sol_types::sol, evm, prelude::*};

sol! {
    /// Emitted when life is not doomed and there is a way.
    #[allow(missing_docs)]
    event ThereIsWay();

    /// The operation failed because there is no way. Like end of the world.
    #[derive(Debug)]
    error NoWay();
}

sol_storage! {
    #[entrypoint]
    struct NoWayNft {
        bool _is_there_a_way;

        Erc721<Override> erc721;
        Erc721Burnable<Override> burnable;
        Erc721Pausable<Override> pausable;
    }
}

#[external]
#[inherit(Erc721Burnable<Override>)]
#[inherit(Erc721Pausable<Override>)]
#[inherit(Erc721<Override>)]
impl NoWayNft {
    fn is_there_a_way(&self) -> bool {
        *self._is_there_a_way
    }

    fn no_way(&mut self) {
        self._is_there_a_way.set(false);
    }

    fn there_is_a_way(&mut self) {
        self._is_there_a_way.set(true);
    }

    fn mint(
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        Erc721::<Override>::_mint(storage, to, token_id)
    }
}

#[r#override]
#[inherit(Erc721BurnableOverride)]
#[inherit(Erc721PausableOverride)]
#[inherit(Erc721Override)]
impl IErc721Virtual for NoWayNftOverride {
    fn _update(
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, Error> {
        let storage = storage.inner_mut::<NoWayNft>();
        if storage.is_there_a_way() {
            evm::log(ThereIsWay {});
            Super::_update::<This>(storage, to, token_id, auth)
        } else {
            Err(Error::Custom(NoWay {}.into()))
        }
    }
}
