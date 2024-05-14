# Integration Tests
## Testing Examples Contracts
Deploying every contract from `./examples` and running integration tests.
### Against local nitro node
Set up first a local nitro node according to this [guide](https://github.com/OffchainLabs/nitro-testnode/blob/release/README.md) and run this command from the project root:
```terminal
    ./integration/test.sh
```

### Against stylus dev net
ALICE_PRIV_KEY and BOB_PRIV_KEY should be valid funded wallets.

Run this command from the project root:
```terminal
    ALICE_PRIV_KEY=0x... \
    BOB_PRIV_KEY=0x... \
    RPC_URL=https://stylus-testnet.arbitrum.io/rpc \
        ./integration/test.sh
```