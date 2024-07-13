use core::marker::PhantomData;

use alloy_primitives::Address;
use openzeppelin_stylus_proc::r#override;
use stylus_sdk::{alloy_primitives::U256, msg, prelude::*};

use crate::{
    token::erc721::{Error, IErc721Virtual, TopLevelStorage},
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

#[r#override]
impl IErc721Virtual for Erc721PausableOverride {
    fn _update(
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, Error> {
        let pausable = storage.inner_mut::<Pausable>();
        pausable.when_not_paused()?;
        Super::_update::<This>(storage, to, token_id, auth)
    }
}

#[cfg(all(test, feature = "std"))]
pub(crate) mod tests {
    use alloy_primitives::address;
    use stylus_sdk::storage::{StorageBool, StorageMap};

    use super::*;
    use crate::token::erc721::{
        tests::random_token_id, Erc721, Erc721Override, IErc721,
    };

    sol_storage! {
        pub struct Token {
            Erc721<Override> erc721;
            Erc721Pausable<Override> pausable;
        }
    }

    #[r#override]
    #[inherit(Erc721PausableOverride)]
    #[inherit(Erc721Override)]
    impl IErc721Virtual for TokenOverride {}

    unsafe impl TopLevelStorage for Token {}

    impl Default for Token {
        fn default() -> Self {
            let root = U256::ZERO;

            Token {
                erc721: Erc721 {
                    _owners: unsafe { StorageMap::new(root, 0) },
                    _balances: unsafe {
                        StorageMap::new(root + U256::from(32), 0)
                    },
                    _token_approvals: unsafe {
                        StorageMap::new(root + U256::from(64), 0)
                    },
                    _operator_approvals: unsafe {
                        StorageMap::new(root + U256::from(96), 0)
                    },
                    _phantom_data: PhantomData,
                },
                pausable: Erc721Pausable {
                    pausable: Pausable {
                        _paused: unsafe {
                            StorageBool::new(root + U256::from(128), 0)
                        },
                    },
                    _phantom_data: PhantomData,
                },
            }
        }
    }

    #[motsu::test]
    fn error_transfer_while_paused(contract: Token) {
        let alice = msg::sender();
        let bob = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("mint a token to Alice");

        let pausable: &mut Pausable = contract.inner_mut();
        pausable.pause();
        let paused = pausable.paused();
        assert!(paused);

        let err =
            Erc721::<Override>::transfer_from(contract, alice, bob, token_id)
                .expect_err("should not transfer from paused contract");

        assert!(matches!(err, Error::EnforcedPause(_)));
    }
}
