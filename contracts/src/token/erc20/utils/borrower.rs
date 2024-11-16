#![allow(missing_docs)]
//! Module with an interface required for smart contract
//! in order to borrow ERC-3156 flashlaon.

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    /// Interface that must be implemented by smart contracts
    /// in order to borrow ERC-3156 flashloan .
    interface IERC3156FlashBorrower {
        /// Handles the receipt of a flash loan.
        /// This function is called after the loan amount has been transferred to the borrower.
        ///
        /// To indicate successful handling of the flash loan, this function should return
        /// the `keccak256` hash of "ERC3156FlashBorrower.onFlashLoan".
        ///
        /// * `initiator` - The address which initiated the flash loan.
        /// * `token` - The address of the token being loaned (loan currency).
        /// * `amount` - The amount of tokens lent in the flash loan.
        /// * `fee` - The additional fee to repay with the flash loan amount.
        /// * `data` - Arbitrary data structure, intended to contain user-defined parameters.
        #[allow(missing_docs)]
        function onFlashLoan(
            address initiator,
            address token,
            uint256 amount,
            uint256 fee,
            bytes calldata data
        ) external returns (bytes32);
    }
}
