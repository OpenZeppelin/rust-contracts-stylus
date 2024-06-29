use core::marker::PhantomData;

use alloy_primitives::Address;
use stylus_sdk::{alloy_primitives::U256, msg, prelude::*};

use crate::{
    token::erc721::{traits::IErc721Virtual, Error, TopLevelStorage},
    utils::Pausable,
};

sol_storage! {
    #[cfg_attr(all(test, feature = "std"), derive(motsu::DefaultStorageLayout))]
    pub struct Erc721Pausable<V: IErc721Virtual> {
        Pausable pausable;
        PhantomData<V> _phantom_data;
    }
}

#[external]
#[inherit(Pausable)]
impl<V: IErc721Virtual> Erc721Pausable<V> {}

pub struct ERC721PausableOverride<B: IErc721Virtual>(B);

impl<B: IErc721Virtual> IErc721Virtual for ERC721PausableOverride<B> {
    type Base = B;

    fn update<V: IErc721Virtual>(
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, Error> {
        let pausable: &mut Pausable = storage.inner_mut();
        pausable.require_not_paused()?;
        Self::Base::update::<V>(storage, to, token_id, auth)
    }
}

#[cfg(all(test, feature = "std"))]
pub(crate) mod tests {
    use alloy_primitives::address;
    use openzeppelin_stylus_proc::inherit;

    use super::*;
    use crate::token::erc721::{
        base::{Erc721, Erc721Override},
        tests::{random_token_id, Override, Token},
        traits::IErc721,
    };

    #[motsu::test]
    fn error_transfer_while_paused(storage: Token) {
        let alice = msg::sender();
        let bob = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");
        let token_id = random_token_id();
        Erc721::<Override>::_mint(storage, alice, token_id)
            .expect("mint a token to Alice");

        let pausable: &mut Pausable = storage.inner_mut();
        pausable.pause();
        let paused = pausable.paused();
        assert!(paused);

        let err =
            Erc721::<Override>::transfer_from(storage, alice, bob, token_id)
                .expect_err("should not transfer from paused contract");

        assert!(matches!(err, Error::EnforcedPause(_)));
    }
}
