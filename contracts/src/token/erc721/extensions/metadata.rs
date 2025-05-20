//! Optional Metadata of the ERC-721 standard.

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use alloy_primitives::U256;
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{prelude::*, storage::StorageString};

use crate::{
    token::erc721::{self, IErc721},
    utils::Metadata,
};

/// State of an [`Erc721Metadata`] contract.
#[storage]
pub struct Erc721Metadata {
    /// [`Metadata`] contract.
    pub(crate) metadata: Metadata,
    // TODO: Remove this field once function overriding is possible. For now we
    // keep this field `pub`, since this is used to simulate overriding.
    /// Base URI for tokens.
    pub base_uri: StorageString,
}

/// Interface for the optional metadata functions from the ERC-721 standard.
#[interface_id]
pub trait IErc721Metadata: IErc165 {
    /// The error type associated to this trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the token collection name.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn name(&self) -> String;

    /// Returns token collection symbol.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn symbol(&self) -> String;

    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
    ///
    /// NOTE: The implementation should use `#[selector(name = "tokenURI")]` to
    /// match Solidity's camelCase naming convention.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - ID of a token.
    ///
    /// # Errors
    ///
    /// * [`erc721::Error::NonexistentToken`] - If the token does not exist.
    #[selector(name = "tokenURI")]
    fn token_uri(
        &self,
        token_id: U256,
    ) -> Result<String, <Self as IErc721Metadata>::Error>;
}

impl Erc721Metadata {
    /// Constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `name` - Token name.
    /// * `symbol` - Token symbol.
    pub fn constructor(&mut self, name: String, symbol: String) {
        self.metadata.constructor(name, symbol);
    }

    /// Check [`IErc721Metadata::name()`] for more details.
    pub fn name(&self) -> String {
        self.metadata.name()
    }

    /// Check [`IErc721Metadata::symbol()`] for more details.
    pub fn symbol(&self) -> String {
        self.metadata.symbol()
    }

    /// Returns the base of Uniform Resource Identifier (URI) for tokens'
    /// collection.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn base_uri(&self) -> String {
        self.base_uri.get_string()
    }

    /// Check [`IErc721Metadata::token_uri()`] for more details.
    #[allow(clippy::missing_errors_doc)]
    pub fn token_uri(
        &self,
        token_id: U256,
        erc721: &impl IErc721<Error = erc721::Error>,
    ) -> Result<String, erc721::Error> {
        erc721.owner_of(token_id)?;

        let base_uri = self.base_uri();

        let token_uri = if base_uri.is_empty() {
            String::new()
        } else {
            base_uri + &token_id.to_string()
        };

        Ok(token_uri)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, FixedBytes};
    use motsu::prelude::Contract;

    use super::*;
    use crate::{
        token::erc721::{self, Erc721},
        utils::introspection::erc165::IErc165,
    };

    #[storage]
    struct Erc721MetadataExample {
        erc721: Erc721,
        metadata: Erc721Metadata,
    }

    #[public]
    #[implements(IErc721Metadata<Error=erc721::Error>, IErc165)]
    impl Erc721MetadataExample {
        #[constructor]
        fn constructor(&mut self, name: String, symbol: String) {
            self.metadata.constructor(name, symbol);
        }
    }

    #[public]
    impl IErc721Metadata for Erc721MetadataExample {
        type Error = erc721::Error;

        fn name(&self) -> String {
            self.metadata.name()
        }

        fn symbol(&self) -> String {
            self.metadata.symbol()
        }

        #[selector(name = "tokenURI")]
        fn token_uri(&self, token_id: U256) -> Result<String, erc721::Error> {
            self.metadata.token_uri(token_id, &self.erc721)
        }
    }

    #[public]
    impl IErc165 for Erc721MetadataExample {
        fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
            <Self as IErc721Metadata>::interface_id() == interface_id
                || <Self as IErc165>::interface_id() == interface_id
        }
    }

    unsafe impl TopLevelStorage for Erc721MetadataExample {}

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc721MetadataExample as IErc721Metadata>::interface_id();
        let expected: FixedBytes<4> = 0x5b5e139f.into();
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(
        contract: Contract<Erc721MetadataExample>,
        alice: Address,
    ) {
        assert!(contract.sender(alice).supports_interface(
            <Erc721MetadataExample as IErc721Metadata>::interface_id()
        ));
        assert!(contract.sender(alice).supports_interface(
            <Erc721MetadataExample as IErc165>::interface_id()
        ));

        let fake_interface_id: FixedBytes<4> = 0x12345678_u32.into();
        assert!(!contract.sender(alice).supports_interface(fake_interface_id));
    }

    #[motsu::test]
    fn metadata(contract: Contract<Erc721MetadataExample>, alice: Address) {
        let name: String = "Erc721MetadataExample".to_string();
        let symbol: String = "OZ".to_string();

        contract.init(alice, |contract| {
            contract.metadata.constructor(name.clone(), symbol.clone());
        });
        assert_eq!(contract.sender(alice).name(), name);
        assert_eq!(contract.sender(alice).symbol(), symbol);
    }

    #[motsu::test]
    fn constructor(contract: Contract<Erc721MetadataExample>, alice: Address) {
        let name: String = "Erc721MetadataExample".to_string();
        let symbol: String = "OZ".to_string();
        contract.init(alice, |contract| {
            contract.constructor(name.clone(), symbol.clone());
        });

        assert_eq!(contract.sender(alice).name(), name);
        assert_eq!(contract.sender(alice).symbol(), symbol);
    }
}
