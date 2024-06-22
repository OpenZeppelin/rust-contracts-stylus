# End-to-end Testing for Stylus Contracts

This end-to-end testing crate provides affordances to test your contracts in a
blockchain environment.

This crate is currently coupled to [`nitro-testnode`] and [`koba`].

[`nitro-testnode`]: https://github.com/OffchainLabs/nitro-testnode
[`koba`]: https://github.com/OpenZeppelin/koba

## Usage

Refer to our end-to-end tests [GitHub workflow] for a working example of a full
tests suite using the `e2e` crate.

[GitHub workflow]: ../../.github/workflows/e2e-tests.yml

### Users

Decorate your tests with the `test` procedural macro: a thin wrapper over
`tokio::test` that sets up `Account`s for your test.

```rust,ignore
#[e2e::test]
async fn users_are_funded(alice: Account) -> eyre::Result<()> {
    let balance = alice.wallet.get_balance(alice.address()).await?;
    let expected = parse_ether("10")?;
    assert_eq!(expected, balance);
    Ok(())
}
```

A `Account` is a thin wrapper over a [`PrivateKeySigner`] and an `alloy` provider with a
[`WalletFiller`]. Both of them are connected to the RPC endpoint defined by the
`RPC_URL` environment variable. This means that a `Account` is the main proxy
between the RPC and the test code.

All users start with 10 ETH as balance. You can have multiple users as
parameters of your test function, or you can create new users separately:

```rust,ignore
#[e2e::test]
async fn foo(alice: Account, bob: Account) -> eyre::Result<()> {
    let charlie = Account::new().await?;
    // ...
}
```

[`LocalWallet`]: https://github.com/alloy-rs/alloy/blob/8aa54828c025a99bbe7e2d4fc9768605d172cc6d/crates/signer-local/src/lib.rs#L37
[`WalletFiller`]: https://github.com/alloy-rs/alloy/blob/8aa54828c025a99bbe7e2d4fc9768605d172cc6d/crates/provider/src/fillers/wallet.rs#L30

### Contracts

We use `koba` to deploy contracts to the blockchain. This is not required, a
separate mechanism for deployment can be used. `e2e` exposes a `deploy`
function that abstracts away the mechanism used in our workflow.

Given a Solidity contract with a constructor at path `src/constructor.sol` like
this:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Example {
    mapping(address account => uint256) private _balances;
    mapping(address account => mapping(address spender => uint256))
        private _allowances;
    uint256 private _totalSupply;
    string private _name;
    string private _symbol;

    constructor(string memory name_, string memory symbol_) {
        _name = name_;
        _symbol = symbol_;
    }
}
```

`e2e::deploy` will deploy the contract marked with the `#[entrypoint]` macro.
Note the `sol!` invocation with the path to the constructor -- this will
generate the abi-encodable `Example::constructorCall` struct.

```rust,ignore
sol!("src/constructor.sol");

async fn deploy(rpc_url: &str, private_key: &str) -> eyre::Result<Address> {
    let args = Example::constructorCall {
        name_: "Token".to_owned(),
        symbol_: "TKN".to_owned(),
    };
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(rpc_url, private_key, Some(args)).await?
}
```

You can then make calls to your contract:

```rust,ignore
#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc20::new(contract_addr, &alice.wallet);

    let Erc20::nameReturn { name } = contract.name().call().await?;
    let Erc20::symbolReturn { symbol } = contract.symbol().call().await?;

    assert_eq!(name, TOKEN_NAME.to_owned());
    assert_eq!(symbol, TOKEN_SYMBOL.to_owned());
    Ok(())
}
```

## Notice

We maintain this crate on a best-effort basis. We use it extensively on our own
tests, so we will continue to add more affordances as we need them.

That being said, please do open an issue to start a discussion, keeping in mind
our [code of conduct] and [contribution guidelines].

[code of conduct]: ../../CODE_OF_CONDUCT.md
[contribution guidelines]: ../../CONTRIBUTING.md

## Security

> [!WARNING]
> This project is still in a very early and experimental phase. It has never
> been audited nor thoroughly reviewed for security vulnerabilities. Do not use
> in production.

Refer to our [Security Policy](../../SECURITY.md) for more details.
