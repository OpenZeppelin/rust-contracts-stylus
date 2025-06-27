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
/// # Example
///
/// ```rust,ignore
/// impl IErc165 for Erc20 {
///     fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
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
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool;
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloc::{vec, vec::Vec};

    use alloy_primitives::{Address, FixedBytes};
    use motsu::prelude::*;
    use stylus_sdk::prelude::*;

    use super::*;

    #[test]
    fn test_supports_interface() {
        // The interface ID for IErc165 itself
        let interface_id =
            FixedBytes::<4>::from(Erc165::INTERFACE_ID.to_be_bytes());
        assert!(Erc165::supports_interface(interface_id));

        // Example interface ID that Erc165 should not support
        let unsupported_interface_id =
            FixedBytes::<4>::from([0xFF, 0xFF, 0xFF, 0xFF]);
        assert!(!Erc165::supports_interface(unsupported_interface_id));
    }

    // A wrapper contract for Erc165 that can be tested with motsu
    #[storage]
    struct Erc165Wrapper;

    unsafe impl TopLevelStorage for Erc165Wrapper {}

    #[external]
    impl Erc165Wrapper {
        fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
            Erc165::supports_interface(interface_id)
        }

        fn get_supported_interfaces(
            &self,
            interface_ids: Vec<FixedBytes<4>>,
        ) -> Vec<bool> {
            interface_ids
                .iter()
                .map(|&id| Erc165::supports_interface(id))
                .collect()
        }
    }

    #[motsu::test]
    fn return_bomb_resistance(
        contract: Contract<Erc165Wrapper>,
        alice: Address,
    ) {
        // The interface ID for IErc165 itself
        let erc165_interface_id =
            FixedBytes::<4>::from(Erc165::INTERFACE_ID.to_be_bytes());

        // Some other interface IDs for testing
        let dummy_id_2 = FixedBytes::<4>::from([0x34u8; 4]);
        let dummy_id_3 = FixedBytes::<4>::from([0x56u8; 4]);
        let dummy_unsupported_id =
            FixedBytes::<4>::from([0xFF, 0xFF, 0xFF, 0xFF]);
        let dummy_unsupported_id_2 = FixedBytes::<4>::from([0x9Au8; 4]);

        // Test that our implementation works correctly for the ERC165 interface
        // ID
        let result =
            contract.sender(alice).supports_interface(erc165_interface_id);
        assert!(result);

        // Test that our implementation correctly rejects unsupported interface
        // IDs
        let result_unsupported =
            contract.sender(alice).supports_interface(dummy_unsupported_id);
        assert!(!result_unsupported);

        // Test the get_supported_interfaces method with a small set of IDs
        let results = contract.sender(alice).get_supported_interfaces(vec![
            erc165_interface_id,
            dummy_id_2,
            dummy_id_3,
            dummy_unsupported_id,
            dummy_unsupported_id_2,
        ]);
        assert_eq!(results.len(), 5);
        assert!(results[0]); // ERC165 interface ID should be supported
        assert!(!results[1]); // dummy_id_2 should not be supported
        assert!(!results[2]); // dummy_id_3 should not be supported
        assert!(!results[3]); // Unsupported interface ID should not be supported
        assert!(!results[4]); // dummy_unsupported_id_2 should not be supported

        // Test with a large number of interface IDs to simulate a potential
        // attack Create a vector with 100+ interface IDs
        let mut large_interface_ids = Vec::new();
        for i in 0..120 {
            let bytes = [(i % 256) as u8, ((i >> 8) % 256) as u8, 0, 0];
            large_interface_ids.push(FixedBytes::<4>::from(bytes));
        }
        // Add the ERC165 interface ID at a known position
        large_interface_ids[50] = erc165_interface_id;

        // Call get_supported_interfaces with the large list
        let large_results = contract
            .sender(alice)
            .get_supported_interfaces(large_interface_ids);

        // Verify results
        assert_eq!(large_results.len(), 120);
        assert!(large_results[50]); // ERC165 interface ID should be supported

        // Verify that all other results are false (except for the ERC165
        // interface ID)
        for (i, &result) in large_results.iter().enumerate() {
            if i != 50 {
                assert!(
                    !result,
                    "Interface at index {} should not be supported",
                    i
                );
            }
        }
    }
}
