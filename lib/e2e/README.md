# End-to-end Testing for Stylus Contracts

This end-to-end testing crate provides affordances to test your contracts in a
blockchain environment.

This crate is currently coupled to [`nitro-testnode`].

[`nitro-testnode`]: https://github.com/OffchainLabs/nitro-testnode

## Usage

Refer to our end-to-end tests [GitHub workflow] for a working example of a full
tests suite using the `e2e` crate.

[GitHub workflow]: ../../.github/workflows/e2e-tests.yml

### Accounts

Decorate your tests with the `test` procedural macro: a thin wrapper over
`tokio::test` that sets up `Account`s for your test.

```rust,ignore
#[e2e::test]
async fn accounts_are_funded(alice: Account) -> eyre::Result<()> {
    let balance = alice.wallet.get_balance(alice.address()).await?;
    let expected = parse_ether("10")?;
    assert_eq!(expected, balance);
    Ok(())
}
```

An `Account` is a thin wrapper over a [`PrivateKeySigner`] and an `alloy` provider with a
[`WalletFiller`]. Both of them are connected to the RPC endpoint defined by the
`RPC_URL` environment variable. This means that a `Account` is the main proxy
between the RPC and the test code.

All accounts start with 10 ETH as balance. You can have multiple accounts as
parameters of your test function, or you can create new accounts separately:

```rust,ignore
#[e2e::test]
async fn foo(alice: Account, bob: Account) -> eyre::Result<()> {
    let charlie = Account::new().await?;
    // ...
}
```

[`WalletFiller`]: https://github.com/alloy-rs/alloy/blob/8aa54828c025a99bbe7e2d4fc9768605d172cc6d/crates/provider/src/fillers/wallet.rs#L30

### Contracts

We use [cargo-stylus] to deploy contracts to the blockchain. `Deployer` type
exposes `Deployer::deploy` method that abstracts away the mechanism used in our
workflow.

Account type exposes `Account::as_deployer` method that returns `Deployer` type.
It will facilitate deployment of the contract marked with the `#[entrypoint]` macro.
Then you can configure deployment with default constructor:

```rust,ignore
let contract_addr = alice.as_deployer().deploy().await?.contract_address;
```

Or with a custom constructor.
Note that you can use the `e2e::constructor!` macro to instantiate a valid constructor struct.

```rust,ignore
let ctr = e2e::constructor!(
    "Token".to_owned(),
    "TKN".to_owned()
);
let receipt = alice
    .as_deployer()
    .with_constructor(ctr)
    .deploy()
    .await?;
```

Then altogether, your first test case can look like this:

```rust,ignore
#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let ctr = e2e::constructor!(
        "Token".to_owned(),
        "TKN".to_owned()
    );
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr)
        .deploy()
        .await?
        .contract_address;
    let contract = Erc20::new(contract_addr, &alice.wallet);

    let name = contract.name().call().await?.name;
    let symbol = contract.symbol().call().await?.symbol;

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

Refer to our [Security Policy](../../SECURITY.md) for more details.
