//! Optional Metadata of the ERC-20 standard.

use alloc::{string::String, vec, vec::Vec};

use alloy_primitives::{aliases::B32, uint, U8};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::prelude::*;

use crate::utils::{introspection::erc165::IErc165, Metadata};

/// Number of decimals used by default on implementors of [`Metadata`].
pub const DEFAULT_DECIMALS: U8 = uint!(18_U8);

/// State of an [`Erc20Metadata`] contract.
#[storage]
pub struct Erc20Metadata {
    /// [`Metadata`] contract.
    pub(crate) metadata: Metadata,
}

/// Interface for the optional metadata functions from the ERC-20 standard.
#[interface_id]
pub trait IErc20Metadata: IErc165 {
    /// Returns the name of the token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[must_use]
    fn name(&self) -> String;

    /// Returns the symbol of the token, usually a shorter version of the name.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[must_use]
    fn symbol(&self) -> String;

    /// Returns the number of decimals used to get a user-friendly
    /// representation of values of this token.
    ///
    /// For example, if `decimals` equals `2`, a balance of `505` tokens should
    /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
    ///
    /// Tokens usually opt for a value of `18`, imitating the relationship
    /// between Ether and Wei. This is the default value returned by this
    /// function ([`DEFAULT_DECIMALS`]), unless it's overridden.
    ///
    /// NOTE: This information is only used for *display* purposes: in
    /// no way it affects any of the arithmetic of the contract, including
    /// [`super::super::IErc20::balance_of`] and
    /// [`super::super::IErc20::transfer`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[must_use]
    fn decimals(&self) -> U8;
}

#[public]
#[implements(IErc20Metadata, IErc165)]
impl Erc20Metadata {
    /// Constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `name` - Token name.
    /// * `symbol` - Token symbol.
    #[constructor]
    pub fn constructor(&mut self, name: String, symbol: String) {
        self.metadata.constructor(name, symbol);
    }
}

#[public]
impl IErc20Metadata for Erc20Metadata {
    fn name(&self) -> String {
        self.metadata.name()
    }

    fn symbol(&self) -> String {
        self.metadata.symbol()
    }

    fn decimals(&self) -> U8 {
        DEFAULT_DECIMALS
    }
}

#[public]
impl IErc165 for Erc20Metadata {
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IErc20Metadata>::interface_id() == interface_id
            || <Self as IErc165>::interface_id() == interface_id
    }
}

#[cfg(test)]
mod tests {
    use motsu::prelude::Contract;
    use stylus_sdk::{alloy_primitives::Address, prelude::*};

    use super::*;

    unsafe impl TopLevelStorage for Erc20Metadata {}

    #[motsu::test]
    fn constructor(contract: Contract<Erc20Metadata>, alice: Address) {
        let name: String = "Erc20Metadata".to_string();
        let symbol: String = "OZ".to_string();
        contract.sender(alice).constructor(name.clone(), symbol.clone());

        assert_eq!(contract.sender(alice).name(), name);
        assert_eq!(contract.sender(alice).symbol(), symbol);
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc20Metadata as IErc20Metadata>::interface_id();
        let expected: B32 = 0xa219a025_u32.into();
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(contract: Contract<Erc20Metadata>, alice: Address) {
        assert!(contract.sender(alice).supports_interface(
            <Erc20Metadata as IErc20Metadata>::interface_id()
        ));
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc20Metadata as IErc165>::interface_id()));

        let fake_interface_id: B32 = 0x12345678_u32.into();
        assert!(!contract.sender(alice).supports_interface(fake_interface_id));
    }
}
