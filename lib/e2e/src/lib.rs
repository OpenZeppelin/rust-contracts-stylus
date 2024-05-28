use ethers::prelude::*;
mod assert;
pub mod context;
mod context_decorator;
mod contract;
pub mod prelude;
pub mod user;

pub use e2e_proc::test;

pub fn random_token_id() -> U256 {
    let num: u32 = rand::random();
    num.into()
}

pub type HttpMiddleware = SignerMiddleware<Provider<Http>, LocalWallet>;
