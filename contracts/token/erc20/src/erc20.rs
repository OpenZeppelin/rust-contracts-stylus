use alloc::{borrow::ToOwned, string::String, vec};
use core::marker::PhantomData;
use stylus_proc::SolidityError;

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use stylus_sdk::{
    evm, msg,
    stylus_proc::{external, sol_storage},
};

sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
}

sol! {
    error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
    error ERC20InvalidSender(address sender);
    error ERC20InvalidReceiver(address receiver);
    error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
    error ERC20InvalidApprover(address approver);
    error ERC20InvalidSpender(address spender);
}

#[derive(SolidityError)]
pub enum Erc20Error {
    InsufficientBalance(ERC20InsufficientBalance),
    InvalidSender(ERC20InvalidSender),
    InvalidReceiver(ERC20InvalidReceiver),
    InsufficientAllowance(ERC20InsufficientAllowance),
    InvalidApprover(ERC20InvalidApprover),
    InvalidSpender(ERC20InvalidSpender),
}

sol_storage! {
    pub struct Erc20<Metadata> {
        mapping(address => uint256) _balances;
        mapping(address => mapping(address => uint256)) _allowances;
        uint256 _total_supply;

        PhantomData<Metadata> metadata;
    }
}

pub trait IErc20Metadata {
    const NAME: &'static str;
    const SYMBOL: &'static str;
    const DECIMALS: u8;
}

#[external]
impl<Metadata> Erc20<Metadata>
where
    Metadata: IErc20Metadata,
{
    pub fn name() -> String {
        Metadata::NAME.to_owned()
    }

    pub fn symbol() -> String {
        Metadata::SYMBOL.to_owned()
    }

    pub fn decimals() -> u8 {
        Metadata::DECIMALS
    }

    pub fn total_supply(&self) -> U256 {
        self._total_supply.get()
    }

    pub fn balance_of(&self, account: Address) -> U256 {
        self._balances.get(account)
    }

    pub fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Erc20Error> {
        let from = msg::sender();
        if from == Address::ZERO {
            return Err(Erc20Error::InvalidSender(ERC20InvalidSender {
                sender: Address::ZERO,
            }));
        }
        if to == Address::ZERO {
            return Err(Erc20Error::InvalidReceiver(ERC20InvalidReceiver {
                receiver: Address::ZERO,
            }));
        }

        self._transfer(from, to, value)?;
        Ok(true)
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self._allowances.get(owner).get(spender)
    }

    pub fn approve(&mut self, spender: Address, value: U256) -> Result<bool, Erc20Error> {
        let owner = msg::sender();
        if owner == Address::ZERO {
            return Err(Erc20Error::InvalidApprover(ERC20InvalidApprover {
                approver: Address::ZERO,
            }));
        }
        if spender == Address::ZERO {
            return Err(Erc20Error::InvalidSpender(ERC20InvalidSpender {
                spender: Address::ZERO,
            }));
        }

        self._allowances.setter(owner).insert(spender, value);
        evm::log(Approval {
            owner,
            spender,
            value,
        });
        Ok(true)
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Erc20Error> {
        if from == Address::ZERO {
            return Err(Erc20Error::InvalidSender(ERC20InvalidSender {
                sender: Address::ZERO,
            }));
        }
        if to == Address::ZERO {
            return Err(Erc20Error::InvalidReceiver(ERC20InvalidReceiver {
                receiver: Address::ZERO,
            }));
        }

        let spender = msg::sender();
        self._use_allowance(from, spender, value)?;
        self._transfer(from, to, value)?;

        Ok(true)
    }
}

impl<Metadata> Erc20<Metadata>
where
    Metadata: IErc20Metadata,
{
    fn _transfer(&mut self, from: Address, to: Address, value: U256) -> Result<(), Erc20Error> {
        let from_balance = self._balances.get(from);
        if from_balance < value {
            return Err(Erc20Error::InsufficientBalance(ERC20InsufficientBalance {
                sender: from,
                balance: from_balance,
                needed: value,
            }));
        }

        let from_balance = from_balance - value;
        self._balances.insert(from, from_balance);
        let to_balance = self._balances.get(to);
        self._balances.insert(to, to_balance + value);
        Ok(())
    }

    pub fn _use_allowance(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Erc20Error> {
        let allowed = self._allowances.get(owner).get(spender);
        if allowed != U256::MAX {
            if allowed < value {
                return Err(Erc20Error::InsufficientAllowance(
                    ERC20InsufficientAllowance {
                        spender,
                        allowance: allowed,
                        needed: value,
                    },
                ));
            }

            self._allowances
                .setter(owner)
                .insert(spender, allowed - value);
        }

        Ok(())
    }
}
