#[public]
#[inherit(Erc20)]
impl Erc20Example {
    fn burn(&mut self, value: U256) -> Result<(), erc20::Error> {
        // ...
        self.erc20.burn(value)
    }

    fn burn_from(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), erc20::Error> {
        // ...
        self.erc20.burn_from(account, value)
    }
}
