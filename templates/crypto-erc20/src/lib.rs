//! Template project for `openzeppelin-stylus` & `openzeppelin-crypto`.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{aliases::B32, Address, U256, U8};
use openzeppelin_crypto::{
    field::{instance::FpBN256, prime::PrimeField},
    poseidon2::{instance::bn256::BN256Params, Poseidon2},
};
use openzeppelin_stylus::{
    token::erc20::{
        self,
        extensions::{Erc20Metadata, IErc20Burnable, IErc20Metadata},
        Erc20, IErc20,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct CryptoErc20 {
    erc20: Erc20,
    metadata: Erc20Metadata,
}

#[public]
#[implements(IErc20<Error = erc20::Error>, IErc20Burnable<Error = erc20::Error>, IErc20Metadata, IErc165)]
impl CryptoErc20 {
    #[constructor]
    pub fn constructor(
        &mut self,
        name: String,
        symbol: String,
    ) -> Result<(), Vec<u8>> {
        self.metadata.constructor(name, symbol);
        Ok(())
    }

    pub fn mint(&mut self, to: Address, value: U256) -> Result<(), Vec<u8>> {
        self.erc20._mint(to, value)?;
        Ok(())
    }

    pub fn poseidon_hash(&mut self, inputs: [U256; 2]) -> U256 {
        let mut hasher = Poseidon2::<BN256Params, FpBN256>::new();

        for input in inputs.iter() {
            let fp = FpBN256::from_bigint(
                openzeppelin_crypto::arithmetic::uint::U256::from(*input),
            );
            hasher.absorb(&fp);
        }

        let hash = hasher.squeeze();
        hash.into_bigint().into()
    }
}

#[public]
impl IErc20 for CryptoErc20 {
    type Error = erc20::Error;

    fn total_supply(&self) -> U256 {
        self.erc20.total_supply()
    }

    fn balance_of(&self, account: Address) -> U256 {
        self.erc20.balance_of(account)
    }

    fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.transfer(to, value)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.approve(spender, value)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.transfer_from(from, to, value)
    }
}

#[public]
impl IErc20Burnable for CryptoErc20 {
    type Error = erc20::Error;

    fn burn(&mut self, value: U256) -> Result<(), Self::Error> {
        self.erc20.burn(value)
    }

    fn burn_from(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        self.erc20.burn_from(account, value)
    }
}

#[public]
impl IErc20Metadata for CryptoErc20 {
    fn name(&self) -> String {
        self.metadata.name()
    }

    fn symbol(&self) -> String {
        self.metadata.symbol()
    }

    fn decimals(&self) -> U8 {
        self.metadata.decimals()
    }
}

#[public]
impl IErc165 for CryptoErc20 {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.erc20.supports_interface(interface_id)
            || self.metadata.supports_interface(interface_id)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{uint, Address, U256};
    use motsu::prelude::*;

    use super::*;

    #[motsu::test]
    fn constructs(contract: Contract<CryptoErc20>, alice: Address) {
        let name: String = "Stylus ERC-20 Workshop".to_string();
        let symbol: String = "MTK".to_string();

        contract
            .sender(alice)
            .constructor(name.clone(), symbol.clone())
            .motsu_expect("should construct");

        assert_eq!(name, contract.sender(alice).name());
        assert_eq!(symbol, contract.sender(alice).symbol());
        assert_eq!(uint!(18_U8), contract.sender(alice).decimals());
    }

    #[motsu::test]
    fn transfer_reverts_when_insufficient_balance(
        contract: Contract<CryptoErc20>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            .constructor(
                "Stylus ERC-20 Workshop".to_string(),
                "MTK".to_string(),
            )
            .motsu_expect("should construct");

        let one = U256::ONE;

        // Initialize state for the test case:
        // Alice's & Bob's balance as `one`.
        contract
            .sender(alice)
            .mint(alice, one)
            .motsu_expect("should mint tokens");

        // Store initial balance & supply.
        let initial_alice_balance = contract.sender(alice).balance_of(alice);
        let initial_bob_balance = contract.sender(alice).balance_of(bob);

        // Transfer action should NOT work - `InsufficientBalance`.
        let err =
            contract.sender(alice).transfer(bob, one + one).motsu_unwrap_err();

        assert!(matches!(
            err,
            erc20::Error::InsufficientBalance(erc20::ERC20InsufficientBalance {
                sender,
                balance,
                needed,
            }) if sender == alice && balance == initial_alice_balance && needed == one + one,
        ));

        // Check proper state (before revert).
        assert_eq!(
            initial_alice_balance,
            contract.sender(alice).balance_of(alice)
        );
        assert_eq!(initial_bob_balance, contract.sender(alice).balance_of(bob));
    }

    #[motsu::test]
    #[should_panic = "should not exceed `U256::MAX` for `total_supply`"]
    fn mint_reverts_when_arithmetic_overflow(
        contract: Contract<CryptoErc20>,
        alice: Address,
    ) {
        contract
            .sender(alice)
            .constructor(
                "Stylus ERC-20 Workshop".to_string(),
                "MTK".to_string(),
            )
            .motsu_expect("should construct");

        let one = U256::ONE;
        assert_eq!(U256::ZERO, contract.sender(alice).balance_of(alice));
        assert_eq!(U256::ZERO, contract.sender(alice).total_supply());

        // Initialize state for the test case:
        // Alice's balance as `U256::MAX`.
        contract
            .sender(alice)
            .mint(alice, U256::MAX)
            .motsu_expect("should mint tokens");
        // Mint action should NOT work:
        // overflow on `total_supply`.
        let _result = contract.sender(alice).mint(alice, one);
    }

    #[motsu::test]
    fn transfers(
        contract: Contract<CryptoErc20>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            .constructor(
                "Stylus ERC-20 Workshop".to_string(),
                "MTK".to_string(),
            )
            .motsu_expect("should construct");

        let one = U256::ONE;

        // Initialize state for the test case:
        //  Alice's & Bob's balance as `one`.
        contract
            .sender(alice)
            .mint(alice, one)
            .motsu_expect("should mint tokens");

        // Store initial balance & supply.
        let initial_alice_balance = contract.sender(alice).balance_of(alice);
        let initial_bob_balance = contract.sender(alice).balance_of(bob);

        // Transfer action should work.
        let result = contract.sender(alice).transfer(bob, one);
        assert!(result.is_ok());

        // Check updated balance & supply.
        assert_eq!(
            initial_alice_balance - one,
            contract.sender(alice).balance_of(alice)
        );
        assert_eq!(
            initial_bob_balance + one,
            contract.sender(alice).balance_of(bob)
        );

        contract.assert_emitted(&erc20::Transfer {
            from: alice,
            to: bob,
            value: one,
        });
    }

    #[motsu::test]
    fn hashes(contract: Contract<CryptoErc20>, alice: Address) {
        contract
            .sender(alice)
            .constructor(
                "Stylus ERC-20 Workshop".to_string(),
                "MTK".to_string(),
            )
            .motsu_expect("should construct");

        let hash = contract
            .sender(alice)
            .poseidon_hash([uint!(123_U256), uint!(123456_U256)]);

        let expected = U256::from_be_slice(&alloy_primitives::hex!(
            "16f70722695a5829a59319fbf746df957a513fdf72b070a67bb72db08070e5de"
        ));

        assert_eq!(hash, expected);
    }
}
