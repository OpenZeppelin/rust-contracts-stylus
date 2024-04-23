extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(IERC20Burnable)]
pub fn ierc20_burnable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl contracts::erc20::extensions::burnable::IERC20Burnable for #name {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IERC20)]
pub fn ierc20_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl contracts::erc20::IERC20 for #name {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IERC20Virtual)]
pub fn ierc20_virtual_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl contracts::erc20::IERC20Virtual for #name {
            fn _update(
                &mut self,
                from:  alloy_primitives::Address,
                to:  alloy_primitives::Address,
                value:  alloy_primitives::U256,
            ) -> Result<(), crate::erc20::Error> {
                // Call "wrapped" token
                self.erc20._update(from, to, value)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IERC20Pausable)]
pub fn ierc20_pausable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl contracts::erc20::IERC20Virtual for #name {
            fn _update(
                &mut self,
                from:  alloy_primitives::Address,
                to: alloy_primitives::Address,
                value: alloy_primitives::U256,
            ) -> Result<(), crate::erc20::Error> {
                // Require `Unpaused` State
                self.when_not_paused().map_err(|_e| {
                    contracts::erc20::Error::PausableError(
                        contracts::utils::pausable::EnforcedPause {},
                    )
                })?;
                // Call "wrapped" token
                self.erc20._update(from, to, value)
            }
        }

        impl contracts::utils::pausable::IPausable for #name {
            fn paused(&self) -> bool {
                self.pausable.paused()
            }

            fn pause(&mut self) -> Result<(), contracts::utils::pausable::Error> {
                self.pausable.pause()
            }

            fn unpause(&mut self) -> Result<(), contracts::utils::pausable::Error> {
                self.pausable.unpause()
            }

            fn when_not_paused(&self) -> Result<(), contracts::utils::pausable::Error> {
                self.pausable.when_not_paused()
            }

            fn when_paused(&self) -> Result<(), contracts::utils::pausable::Error> {
                self.pausable.when_paused()
            }

        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IERC20Storage)]
pub fn ierc20_storage_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl contracts::erc20::IERC20Storage for #name {
            fn _get_total_supply(&self) -> alloy_primitives::U256 {
                self.erc20._get_total_supply()
            }

            fn _set_total_supply(&mut self, total_supply: alloy_primitives::U256) {
                self.erc20._set_total_supply(total_supply)
            }

            fn _get_balance(&self, account: alloy_primitives::Address) -> alloy_primitives::U256 {
                self.erc20._get_balance(account)
            }

            fn _set_balance(&mut self, account: alloy_primitives::Address, balance: alloy_primitives::U256) {
                self.erc20._set_balance(account, balance);
            }

            fn _get_allowance(&self, owner: alloy_primitives::Address, spender: alloy_primitives::Address) -> alloy_primitives::U256 {
                self.erc20._get_allowance(owner, spender)
            }

            fn _set_allowance(
                &mut self,
                owner: alloy_primitives::Address,
                spender: alloy_primitives::Address,
                allowance: alloy_primitives::U256,
            ) {
                self._set_allowance(owner, spender, allowance);
            }
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(IPausable)]
pub fn ipausable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl contracts::utils::pausable::IPausable for #name {
            fn paused(&self) -> bool {
                self.erc20.paused()
            }

            fn pause(&mut self) -> Result<(), contracts::utils::pausable::Error> {
                self.erc20.pause()
            }

            fn unpause(&mut self) -> Result<(), contracts::utils::pausable::Error> {
                self.erc20.unpause()
            }

            fn when_not_paused(&self) -> Result<(), contracts::utils::pausable::Error> {
                self.erc20.when_not_paused()
            }

            fn when_paused(&self) -> Result<(), contracts::utils::pausable::Error> {
                self.erc20.when_paused()
            }

        }
    };

    TokenStream::from(expanded)
}
