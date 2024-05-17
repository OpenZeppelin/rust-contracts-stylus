# Integration Tests

## Run tests

Deploying every contract from `./examples` directory and running integration tests.

### Against local nitro node

Set up first a local nitro node according to
this [guide](https://github.com/OffchainLabs/nitro-testnode/blob/release/README.md) and run this command from the
project root:

```terminal
    ./integration/test.sh
```

### Against stylus dev net

`ALICE_PRIV_KEY` and `BOB_PRIV_KEY` should be valid funded wallets.
`RPC_URL` should contain url of the stylus testnet.
Run this command from the project root:

```terminal
    ALICE_PRIV_KEY=0x... \
    BOB_PRIV_KEY=0x... \
    RPC_URL=https://stylus-testnet.arbitrum.io/rpc \
        ./integration/test.sh
```

## Add test for the new contract

Assuming that contract associated crate exists at `./examples` directory
with the crate name `erc20-example`.
Add ethereum contracts to `./integration/src/infrastructure` directory like:

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

Then add wrapper type for the contract:

```rust
pub type Erc20 = Erc20Token<HttpMiddleware>;
link_to_crate!(Erc20, "erc20-example");
```

Tests should create new infrastructure instance like this:

```rust
let infra = Infrastructure::<Erc20>::new().await?;
```