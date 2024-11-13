use super::borrower::IERC3156FlashBorrower;
use alloy_primitives::{fixed_bytes, uint, Address, Bytes, FixedBytes, U128, U256};



pub trait  IERC3156FlashLender {
    
    fn max_flash_loan(token:Address) -> U256;

    fn flash_fee(token:Address, amount:U256)  -> U256;

    fn flash_loan(
        receiver: IERC3156FlashBorrower,
        token:Address,
        amount:U256,
        data:Bytes
    ) -> bool;
}