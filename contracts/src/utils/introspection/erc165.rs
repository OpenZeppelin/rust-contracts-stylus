//! Trait and implementation of the ERC-165 standard, as defined in the [ERC].
//!
//! [ERC]: https://eips.ethereum.org/EIPS/eip-165

use alloy_primitives::FixedBytes;
use openzeppelin_stylus_proc::interface_id;

/// Interface of the ERC-165 standard, as defined in the [ERC].
///
/// Implementers can declare support of contract interfaces, which others can
/// query.
///
/// For an implementation, see [`Erc165`].
///
/// [ERC]: https://eips.ethereum.org/EIPS/eip-165
#[interface_id]
pub trait IErc165 {
    /// Returns true if this contract implements the interface defined by
    /// `interface_id`. See the corresponding [ERC] to learn more about how
    /// these ids are created.
    ///
    /// NOTE: Method [`IErc165::supports_interface`] should be reexported with
    /// `#[public]` macro manually, see the Example section.
    ///
    /// # Arguments
    ///
    /// * `interface_id` - The interface identifier, as specified in the [ERC].
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[public]
    /// impl Erc20Example {
    ///     fn supports_interface(interface_id: FixedBytes<4>) -> bool {
    ///         Erc20::supports_interface(interface_id)
    ///             || Erc20Metadata::supports_interface(interface_id)
    ///     }
    /// }
    /// ```
    ///
    /// [ERC]: https://eips.ethereum.org/EIPS/eip-165#how-interfaces-are-identified
    fn supports_interface(interface_id: FixedBytes<4>) -> bool;
}

/// Implementation of the [`IErc165`] trait.
///
/// Contracts that want to support ERC-165 should implement the [`IErc165`]
/// trait for the additional interface id that will be supported and call
/// [`Erc165::supports_interface`] like:
///
/// ```rust,ignore
/// impl IErc165 for Erc20 {
///     fn supports_interface(interface_id: FixedBytes<4>) -> bool {
///         crate::token::erc20::INTERFACE_ID == u32::from_be_bytes(*interface_id)
///             || Erc165::supports_interface(interface_id)
///     }
/// }
/// ```
pub struct Erc165;

impl IErc165 for Erc165 {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        Self::INTERFACE_ID == u32::from_be_bytes(*interface_id)
    }
}
