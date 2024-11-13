use alloy_sol_macro::sol;
use stylus_sdk::{prelude::*, call::MethodError};


pub mod borrower;
pub mod lender;



sol! {
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC3156UnsupportedToken(address token);

     #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC3156ExceededMaxLoan(uint256 maxLoan);

    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC3156InvalidReceiver(address receiver);
}



#[derive(SolidityError, Debug)]
pub enum Error {
    UnsupportedToken(ERC3156UnsupportedToken),
    ExceededMaxLoan(ERC3156ExceededMaxLoan),
    InvalidReceiver(ERC3156InvalidReceiver),
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}
