#![allow(missing_docs)]

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IERC3156FlashBorrower {
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
