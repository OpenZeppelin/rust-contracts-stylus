# Integration Tests

## Run tests

Deploy every contract from `./examples` directory and running integration tests.

### Against local nitro node

Set up first a local nitro node according to
this [guide](https://github.com/OffchainLabs/nitro-testnode/blob/release/README.md)

```terminal
# setup nitro test node in detached mode
# docker images should be shutdown manually later
./test-node.bash --no-run --init --no-tokenbridge
./test-node.bash --detach

# fund Alice's wallet
./test-node.bash script send-l2 --to address_0x01fA6bf4Ee48B6C95900BCcf9BEA172EF5DBd478 --ethamount 10000
# fund Bob's wallet
./test-node.bash script send-l2 --to address_0xF4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526 --ethamount 10000
```

Run integration testing command:

```terminal
    ./e2e-tests/test.sh
```

### Against stylus dev net

`ALICE_PRIV_KEY` and `BOB_PRIV_KEY` should be valid funded wallets.
`RPC_URL` should contain url of the stylus testnet.
Run this command:

```terminal
    ALICE_PRIV_KEY=0x... \
    BOB_PRIV_KEY=0x... \
    RPC_URL=https://stylus-testnet.arbitrum.io/rpc \
        ./e2e-tests//test.sh
```

## Add test for the new contract

Assuming that contract associated crate exists at `./examples` directory
with the crate name `erc20-example`.
Add ethereum contracts to `./e2e-tests/src/context` directory like:

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
let context = E2EContext::<Erc20>::new().await?;
```