//! Implementation of the Content URI extension as defined in ERC-6909.
//! Provides per-contract and per-token URI storage.

use alloc::{string::String, vec, vec::Vec};

use alloy_primitives::{aliases::B32, Address, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    evm,
    prelude::*,
    storage::{StorageMap, StorageString},
};

use crate::{
    token::erc6909::{self, Erc6909, Error, IErc6909},
    utils::introspection::erc165::IErc165,
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when the contract URI is changed.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event ContractURIUpdated();

        /// Emitted when a token URI is changed.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event URI(string value, uint256 indexed id);
    }
}

pub use sol::*;

/// State of an [`Erc6909ContentUri`] contract.
#[storage]
pub struct Erc6909ContentUri {
    /// [`Erc6909`] contract.
    pub erc6909: Erc6909,
    /// Contract-level URI.
    pub(crate) contract_uri: StorageString,
    /// Mapping from token id to token URI.
    pub(crate) token_uris: StorageMap<U256, StorageString>,
}

/// Required interface of a [`Erc6909ContentUri`] contract.
#[interface_id]
pub trait IErc6909ContentUri: IErc165 {
    /// Returns the URI for the contract.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[selector(name = "contractURI")]
    fn contract_uri(&self) -> String;

    /// Returns the URI for the token of type `id`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `id` - Token id as a number.
    #[selector(name = "tokenURI")]
    fn token_uri(&self, id: U256) -> String;
}

#[public]
#[implements(IErc6909<Error = Error>, IErc6909ContentUri, IErc165)]
impl Erc6909ContentUri {}

#[public]
impl IErc6909ContentUri for Erc6909ContentUri {
    fn contract_uri(&self) -> String {
        self.contract_uri.get_string()
    }

    fn token_uri(&self, id: U256) -> String {
        self.token_uris.get(id).get_string()
    }
}

#[public]
impl IErc6909 for Erc6909ContentUri {
    type Error = erc6909::Error;

    fn balance_of(&self, owner: Address, id: U256) -> U256 {
        self.erc6909.balance_of(owner, id)
    }

    fn allowance(&self, owner: Address, spender: Address, id: U256) -> U256 {
        self.erc6909.allowance(owner, spender, id)
    }

    fn is_operator(&self, owner: Address, spender: Address) -> bool {
        self.erc6909.is_operator(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909.approve(spender, id, amount)
    }

    fn set_operator(
        &mut self,
        spender: Address,
        approved: bool,
    ) -> Result<bool, Self::Error> {
        self.erc6909.set_operator(spender, approved)
    }

    fn transfer(
        &mut self,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909.transfer(receiver, id, amount)
    }

    fn transfer_from(
        &mut self,
        sender: Address,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909.transfer_from(sender, receiver, id, amount)
    }
}

impl Erc6909ContentUri {
    /// Sets the contract URI.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_contract_uri` - New URI for the contract.
    ///
    /// # Events
    ///
    /// * [`ContractURIUpdated`].
    pub fn _set_contract_uri(&mut self, new_contract_uri: String) {
        self.contract_uri.set_str(new_contract_uri);
        evm::log(ContractURIUpdated {});
    }

    /// Sets the token URI for a given token of type `id`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `id` - Token id as a number.
    /// * `new_token_uri` - New URI for the token.
    ///
    /// # Events
    ///
    /// * [`URI`].
    pub fn _set_token_uri(&mut self, id: U256, new_token_uri: String) {
        self.token_uris.setter(id).set_str(&new_token_uri);
        evm::log(URI { value: new_token_uri, id });
    }

    /// Creates `amount` of token `id` and assigns them to `account`, by
    /// transferring it from [`Address::ZERO`]. Relies on the `_update`
    /// mechanism.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If the `to` address is [`Address::ZERO`].
    pub fn _mint(
        &mut self,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        if to.is_zero() {
            return Err(erc6909::Error::InvalidReceiver(
                erc6909::ERC6909InvalidReceiver { receiver: Address::ZERO },
            ));
        }

        self.erc6909._update(Address::ZERO, to, id, amount)
    }

    /// Destroys a `amount` of token `id` from `account`.
    /// Relies on the `_update` mechanism.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If the `from` address is [`Address::ZERO`].
    pub fn _burn(
        &mut self,
        from: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        if from.is_zero() {
            return Err(erc6909::Error::InvalidSender(
                erc6909::ERC6909InvalidSender { sender: Address::ZERO },
            ));
        }

        self.erc6909._update(from, Address::ZERO, id, amount)
    }
}

#[public]
impl IErc165 for Erc6909ContentUri {
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IErc6909ContentUri>::interface_id() == interface_id
            || self.erc6909.supports_interface(interface_id)
    }
}

#[cfg(test)]
mod tests {
    use motsu::prelude::*;
    use stylus_sdk::{
        alloy_primitives::{aliases::B32, fixed_bytes, uint, Address, U256},
        prelude::*,
    };

    use super::*;
    use crate::token::erc6909::{
        ERC6909InvalidReceiver, ERC6909InvalidSender, ERC6909InvalidSpender,
    };

    unsafe impl TopLevelStorage for Erc6909ContentUri {}

    #[motsu::test]
    fn contract_uri_initially_empty(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
    ) {
        assert!(contract.sender(alice).contract_uri().is_empty());
    }

    #[motsu::test]
    fn token_uri_initially_empty(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
    ) {
        let id = uint!(1_U256);
        assert!(contract.sender(alice).token_uri(id).is_empty());
    }

    #[motsu::test]
    fn set_contract_uri(contract: Contract<Erc6909ContentUri>, alice: Address) {
        let uri = "https://example.com/contract".to_string();
        contract.sender(alice)._set_contract_uri(uri.clone());
        assert_eq!(uri, contract.sender(alice).contract_uri());
    }

    #[motsu::test]
    fn set_token_uri(contract: Contract<Erc6909ContentUri>, alice: Address) {
        let id = uint!(1_U256);
        let uri = "https://example.com/token/1".to_string();
        contract.sender(alice)._set_token_uri(id, uri.clone());
        assert_eq!(uri, contract.sender(alice).token_uri(id));
    }

    #[motsu::test]
    fn set_token_uri_per_id(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
    ) {
        let id1 = uint!(1_U256);
        let id2 = uint!(2_U256);
        let uri1 = "https://example.com/token/1".to_string();
        let uri2 = "https://example.com/token/2".to_string();

        contract.sender(alice)._set_token_uri(id1, uri1.clone());
        contract.sender(alice)._set_token_uri(id2, uri2.clone());

        assert_eq!(uri1, contract.sender(alice).token_uri(id1));
        assert_eq!(uri2, contract.sender(alice).token_uri(id2));
    }

    #[motsu::test]
    fn set_contract_uri_overwrites(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
    ) {
        let uri1 = "https://example.com/v1".to_string();
        let uri2 = "https://example.com/v2".to_string();

        contract.sender(alice)._set_contract_uri(uri1);
        contract.sender(alice)._set_contract_uri(uri2.clone());

        assert_eq!(uri2, contract.sender(alice).contract_uri());
    }

    #[motsu::test]
    fn set_token_uri_overwrites(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
    ) {
        let id = uint!(1_U256);
        let uri1 = "https://example.com/v1".to_string();
        let uri2 = "https://example.com/v2".to_string();

        contract.sender(alice)._set_token_uri(id, uri1);
        contract.sender(alice)._set_token_uri(id, uri2.clone());

        assert_eq!(uri2, contract.sender(alice).token_uri(id));
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc6909ContentUri as IErc6909ContentUri>::interface_id();
        let expected: B32 = fixed_bytes!("0x20d88258");
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
    ) {
        assert!(contract.sender(alice).supports_interface(
            <Erc6909ContentUri as IErc6909ContentUri>::interface_id()
        ));
        assert!(
            contract.sender(alice).supports_interface(
                <Erc6909ContentUri as IErc165>::interface_id()
            )
        );
        assert!(contract.sender(alice).supports_interface(
            <Erc6909ContentUri as IErc6909>::interface_id()
        ));

        let fake_interface_id = 0x12345678u32;
        assert!(!contract
            .sender(alice)
            .supports_interface(fake_interface_id.into()));
    }

    #[motsu::test]
    fn mint(contract: Contract<Erc6909ContentUri>, alice: Address) {
        let id = uint!(1_U256);
        let ten = uint!(10_U256);

        contract
            .sender(alice)
            ._mint(alice, id, ten)
            .expect("should mint tokens for Alice");

        assert_eq!(ten, contract.sender(alice).balance_of(alice, id));
    }

    #[motsu::test]
    fn mint_errors_invalid_receiver(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
    ) {
        let id = uint!(1_U256);
        let ten = uint!(10_U256);

        let invalid_receiver = Address::ZERO;

        let err = contract
            .sender(alice)
            ._mint(invalid_receiver, id, ten)
            .motsu_unwrap_err();

        assert!(
            matches!(err, Error::InvalidReceiver(ERC6909InvalidReceiver { receiver }) if receiver == invalid_receiver)
        );
    }

    #[motsu::test]
    fn burn(contract: Contract<Erc6909ContentUri>, alice: Address) {
        let id = uint!(2_U256);
        let ten = uint!(10_U256);
        let one = uint!(1_U256);

        contract
            .sender(alice)
            ._mint(alice, id, ten)
            .expect("should mint tokens for Alice");

        contract
            .sender(alice)
            ._burn(alice, id, one)
            .expect("should burn tokens for Alice");

        assert_eq!(ten - one, contract.sender(alice).balance_of(alice, id));
    }

    #[motsu::test]
    fn burn_errors_invalid_sender(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
    ) {
        let id = uint!(2_U256);
        let ten = uint!(10_U256);

        let invalid_sender = Address::ZERO;

        let err = contract
            .sender(alice)
            ._burn(invalid_sender, id, ten)
            .motsu_unwrap_err();
        assert!(
            matches!(err, Error::InvalidSender(ERC6909InvalidSender { sender }) if sender == invalid_sender)
        );
    }

    #[motsu::test]
    fn balance_of(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
        bob: Address,
    ) {
        let id = uint!(1_U256);
        let ten = uint!(10_U256);

        assert_eq!(U256::ZERO, contract.sender(alice).balance_of(alice, id));
        assert_eq!(U256::ZERO, contract.sender(alice).balance_of(bob, id));

        contract
            .sender(alice)
            ._mint(alice, id, ten)
            .expect("should mint tokens for Alice");

        assert_eq!(ten, contract.sender(alice).balance_of(alice, id));
        assert_eq!(U256::ZERO, contract.sender(alice).balance_of(bob, id));
    }

    #[motsu::test]
    fn transfer(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
        bob: Address,
    ) {
        let id = uint!(1_U256);
        let ten = uint!(10_U256);
        let three = uint!(3_U256);

        contract
            .sender(alice)
            ._mint(alice, id, ten)
            .expect("should mint tokens for Alice");

        let result = contract
            .sender(alice)
            .transfer(bob, id, three)
            .expect("should transfer tokens from Alice to Bob");

        assert!(result);
        assert_eq!(ten - three, contract.sender(alice).balance_of(alice, id));
        assert_eq!(three, contract.sender(alice).balance_of(bob, id));
    }

    #[motsu::test]
    fn approve(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
        bob: Address,
    ) {
        let id = uint!(1_U256);
        let ten = uint!(10_U256);

        assert_eq!(
            U256::ZERO,
            contract.sender(alice).allowance(alice, bob, id)
        );

        let result = contract
            .sender(alice)
            .approve(bob, id, ten)
            .expect("should approve Bob to spend Alice's tokens");

        assert!(result);
        assert_eq!(ten, contract.sender(alice).allowance(alice, bob, id));
    }

    #[motsu::test]
    fn approve_errors_invalid_spender(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
    ) {
        let id = uint!(1_U256);
        let ten = uint!(10_U256);

        let invalid_spender = Address::ZERO;

        let err = contract
            .sender(alice)
            .approve(invalid_spender, id, ten)
            .motsu_unwrap_err();

        assert!(
            matches!(err, Error::InvalidSpender(ERC6909InvalidSpender { spender }) if spender == invalid_spender)
        );
    }

    #[motsu::test]
    fn transfer_from_with_allowance(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        let id = uint!(1_U256);
        let ten = uint!(10_U256);
        let three = uint!(3_U256);

        contract
            .sender(alice)
            ._mint(alice, id, ten)
            .expect("should mint tokens for Alice");

        contract
            .sender(alice)
            .approve(bob, id, ten)
            .expect("should approve Bob to spend Alice's tokens");

        let result = contract
            .sender(bob)
            .transfer_from(alice, charlie, id, three)
            .expect("should transfer tokens from Alice to Charlie via Bob");

        assert!(result);
        assert_eq!(ten - three, contract.sender(alice).balance_of(alice, id));
        assert_eq!(three, contract.sender(alice).balance_of(charlie, id));
        assert_eq!(
            ten - three,
            contract.sender(alice).allowance(alice, bob, id)
        );
    }

    #[motsu::test]
    fn transfer_from_with_operator(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        let id = uint!(1_U256);
        let ten = uint!(10_U256);
        let three = uint!(3_U256);

        contract
            .sender(alice)
            ._mint(alice, id, ten)
            .expect("should mint tokens for Alice");

        contract
            .sender(alice)
            .set_operator(bob, true)
            .expect("should set Bob as operator for Alice");

        let result = contract
            .sender(bob)
            .transfer_from(alice, charlie, id, three)
            .expect("should transfer tokens from Alice to Charlie via Bob");

        assert!(result);
        assert_eq!(ten - three, contract.sender(alice).balance_of(alice, id));
        assert_eq!(three, contract.sender(alice).balance_of(charlie, id));
    }

    #[motsu::test]
    fn set_operator(
        contract: Contract<Erc6909ContentUri>,
        alice: Address,
        bob: Address,
    ) {
        assert!(!contract.sender(alice).is_operator(alice, bob));

        let result = contract
            .sender(alice)
            .set_operator(bob, true)
            .expect("should set Bob as operator for Alice");

        assert!(result);
        assert!(contract.sender(alice).is_operator(alice, bob));

        let result = contract
            .sender(alice)
            .set_operator(bob, false)
            .expect("should revoke Bob as operator for Alice");

        assert!(result);
        assert!(!contract.sender(alice).is_operator(alice, bob));
    }
}
