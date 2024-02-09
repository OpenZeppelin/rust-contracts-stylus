use std::marker::PhantomData;

use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolError};
use stylus_sdk::{
    evm, msg,
    stylus_proc::{external, sol_storage},
};

// ERC20 Events.
sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
}

// ERC20 Errors.
sol! {
    error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
    error ERC20InvalidSender(address sender);
    error ERC20InvalidReceiver(address receiver);
    error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
    error ERC20InvalidApprover(address approver);
    error ERC20InvalidSpender(address spender);
}

pub enum Erc20Error {
    InsufficientBalance(ERC20InsufficientBalance),
    InvalidSender(ERC20InvalidSender),
    InvalidReceiver(ERC20InvalidReceiver),
    InsufficientAllowance(ERC20InsufficientAllowance),
    InvalidApprover(ERC20InvalidApprover),
    InvalidSpender(ERC20InvalidSpender),
}

impl From<Erc20Error> for Vec<u8> {
    fn from(err: Erc20Error) -> Vec<u8> {
        match err {
            Erc20Error::InsufficientBalance(e) => e.encode(),
            Erc20Error::InvalidSender(e) => e.encode(),
            Erc20Error::InvalidReceiver(e) => e.encode(),
            Erc20Error::InsufficientAllowance(e) => e.encode(),
            Erc20Error::InvalidApprover(e) => e.encode(),
            Erc20Error::InvalidSpender(e) => e.encode(),
        }
    }
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
    pub fn name() -> Result<String, Erc20Error> {
        Ok(Metadata::NAME.to_owned())
    }

    pub fn symbol() -> Result<String, Erc20Error> {
        Ok(Metadata::SYMBOL.to_owned())
    }

    pub fn decimals() -> Result<u8, Erc20Error> {
        Ok(Metadata::DECIMALS)
    }

    pub fn total_supply(&self) -> Result<U256, Erc20Error> {
        let total_supply = self._total_supply.get();
        Ok(total_supply)
    }

    pub fn balance_of(&self, account: Address) -> Result<U256, Erc20Error> {
        let balance = self._balances.get(account);
        Ok(balance)
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

    pub fn allowance(&self, owner: Address, spender: Address) -> Result<U256, Erc20Error> {
        let allowance = self._allowances.get(owner).get(spender);
        Ok(allowance)
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
