//! Trait and implementation of the ERC-165 standard, as defined in the [ERC].
//!
//! [ERC]: https://eips.ethereum.org/EIPS/eip-165

use alloy_primitives::aliases::B32;
use openzeppelin_stylus_proc::interface_id;

/// Interface of the ERC-165 standard, as defined in the [ERC].
///
/// Implementers can declare support of contract interfaces, which others can
/// query.
///
/// # Example
///
/// ```rust,ignore
/// impl IErc165 for Erc20 {
///     fn supports_interface(&self, interface_id: B32) -> bool {
///         <Self as IErc20>::interface_id() == interface_id
///             || <Self as IErc165>::interface_id() == interface_id
///     }
/// }
/// ```
///
/// [ERC]: https://eips.ethereum.org/EIPS/eip-165
#[interface_id]
pub trait IErc165 {
    /// Returns true if this contract implements the interface defined by
    /// `interface_id`. See the corresponding [ERC] to learn more about how
    /// these ids are created.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `interface_id` - The interface identifier, as specified in the [ERC].
    ///
    /// [ERC]: https://eips.ethereum.org/EIPS/eip-165#how-interfaces-are-identified
    fn supports_interface(&self, interface_id: B32) -> bool;
}
