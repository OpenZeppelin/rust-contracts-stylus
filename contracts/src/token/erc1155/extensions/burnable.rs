//! Extension of {ERC1155} that allows token holders to destroy both their
//! own tokens and those that they have been approved to use.
use crate::token::erc1155::Erc1155;

/// Extension of [`Erc1155`] that allows token holders to destroy both their
/// own tokens and those that they have been approved to use.
pub trait IErc1155Burnable {}
