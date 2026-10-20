# End-to-end Testing for Stylus Contracts

This crate runs the example contracts of this repository against a local
Stylus devnet: the nitro dev node that the Stylus SDK ships for its own
integration tests (`stylus_tools::devnet::Node`, nitro-node v3.11.1,
`ArbOS` 60). It gives tests funded accounts, deploys compiled contracts the way
`cargo stylus deploy` does (through the `StylusDeployer` contract when there
is a constructor, as fragments when the contract is above the code size
limit), and offers assertions for reverts and events.

The crate is internal to this repository and is not published.

## Running the tests

Start the devnet once (it needs Docker):

```bash
./scripts/devnet.sh up
```

The daemon writes the RPC URL of the node to `target/devnet/rpc`. Tests and
benches read it from there, or from the `RPC_URL` environment variable when
it is set. Then build the contracts and run the suite:

```bash
./scripts/e2e-tests.sh            # every example
./scripts/e2e-tests.sh -p erc20-example   # one example
./scripts/devnet.sh down
```

`scripts/e2e-tests.sh` builds every example for `wasm32-unknown-unknown` in
release mode and runs `cargo test --features e2e --test "*"`.

## Accounts

Decorate your tests with the `test` procedural macro: a thin wrapper over
`tokio::test` that sets up `Account`s for your test.

```rust,ignore
#[e2e::test]
async fn accounts_are_funded(alice: Account) -> eyre::Result<()> {
    let balance = alice.wallet.get_balance(alice.address()).await?;
    assert_eq!(e2e::DEFAULT_FUNDING_WEI, balance);
    Ok(())
}
```

An `Account` is a thin wrapper over a [`PrivateKeySigner`] and an `alloy`
provider with a [`WalletFiller`], connected to the devnet. Every account
starts with 0.1 ETH (`DEFAULT_FUNDING_WEI`); call `account.fund(wei)` for
more. Accounts are funded from a per-process faucet account that is topped up
from the devnet owner, so test binaries can run in parallel.

```rust,ignore
#[e2e::test]
async fn foo(alice: Account, bob: Account) -> eyre::Result<()> {
    let charlie = Account::new().await?;
    // ...
}
```

[`PrivateKeySigner`]: https://docs.rs/alloy/latest/alloy/signers/local/type.PrivateKeySigner.html
[`WalletFiller`]: https://docs.rs/alloy/latest/alloy/providers/fillers/struct.WalletFiller.html

## Deployments

`account.as_deployer()` deploys the compiled contract of the crate the test
belongs to, from `target/wasm32-unknown-unknown/release`. Use
`with_constructor(constructor!(args...))` to run the contract's constructor
and `with_example_name("erc20")` to deploy another example.

```rust,ignore
#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let receipt = alice
        .as_deployer()
        .with_constructor(constructor!("Token".to_string(), "TKN".to_string()))
        .deploy()
        .await?;
    let contract = Erc20::new(receipt.contract_address, &alice.wallet);
    // ...
}
```

A reverting constructor surfaces as a `ContractInitializationError` in the
returned `eyre::Report`.

## Assertions

`send!`, `watch!` and `receipt!` shorten the `alloy` calls. `Revert`,
`Panic` and `RustPanic` check the error of a failed call; `EventExt::emits`
checks that a receipt contains an event.

```rust,ignore
let err = send!(contract.transfer(bob, value)).expect_err("should revert");
assert!(err.reverted_with(Erc20::ERC20InsufficientBalance { .. }));

let receipt = receipt!(contract.mint(alice_addr, value))?;
assert!(receipt.emits(Erc20::Transfer { from: Address::ZERO, to: alice_addr, value }));
```
