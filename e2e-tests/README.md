# Integration Tests

## Run tests

### Setup local nitro node

Run in detached mode:

```terminal
./nitro-testnode -d
```

Clean local nitro node to free system resources:

```terminal
./nitro-testnode -down
```

### Run end-to-end tests

Builds every contract with entrypoint and run tests against locally deployed nitro node:

```terminal
./run.sh
```

## Add test for the new contract

Add solidity abi:

```rust
ethers::contract::abigen!(
    Erc20Token,
    r#"[
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool)
        function mint(address account, uint256 amount) external
        
        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed)
    ]"#
);
```

Then add wrapper type for the contract and link to an example crate name:

```rust
pub type Erc20 = Erc20Token<HttpMiddleware>;
link_to_crate!(Erc20, "erc20-example");
```

Tests should instantiate new contract like this:

```rust
let erc20 = &alice.deploys::<Erc20>().await?;
```
