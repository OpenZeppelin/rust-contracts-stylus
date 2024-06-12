use alloy_primitives::U256;
use stylus_sdk::stylus_proc::sol_storage;

sol_storage! {
    #[derive(motsu_proc::StylusDefault)]
    pub struct Erc20 {
        /// Maps users to balances.
        mapping(address => uint256) _balances;
        /// Maps users to a mapping of each spender's allowance.
        mapping(address => mapping(address => uint256)) _allowances;
        /// The total supply of the token.
        uint256 _total_supply;
    }
}

#[motsu::test]
fn instantiates() {
    let contract = Erc20::default();
    assert_eq!(contract._total_supply.get(), U256::from(0));
}
