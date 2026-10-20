use alloy::{primitives::Address, signers::local::PrivateKeySigner, sol};
use e2e::{constructor, Account};

sol!(
    #[sol(rpc)]
    contract BasicToken {
        constructor(string memory name_, string memory symbol_);

        function name() external view returns (string name);
        function symbol() external view returns (string symbol);
    }
);

const RPC_URL: &str = "https://sepolia-rollup.arbitrum.io/rpc";
const PRIVATE_KEY: &str = "your private key";
const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "TTK";

#[tokio::main]
async fn main() {
    // WARNING: Please use a more secure method for storing your privaket key
    // than a string at the top of this file. The following code is for testing
    // purposes only.
    let signer = PRIVATE_KEY
        .parse::<PrivateKeySigner>()
        .expect("should parse the private key");
    let account = Account::from_signer(signer, RPC_URL)
        .expect("should connect to the RPC endpoint");
    let provider = account.wallet.clone();

    let contract_address = deploy(&account).await;

    let contract = BasicToken::new(contract_address, &provider);

    let name = contract.name().call().await.unwrap();
    assert_eq!(name, TOKEN_NAME.to_owned());

    let symbol = contract.symbol().call().await.unwrap();
    assert_eq!(symbol, TOKEN_SYMBOL.to_owned());
}

/// Deploy a `BasicToken` contract to `RPC_URL` using `cargo-stylus`.
async fn deploy(account: &Account) -> Address {
    let manifest_dir =
        std::env::current_dir().expect("should get current dir from env");

    // NOTE: It's expected that you compiled your contract beforehand.
    //
    // You should run `cargo build --release --target wasm32-unknown-unknown` to
    // get a wasm binary at `target/wasm32-unknown-unknown/release/{name}.wasm`.
    let wasm_path = manifest_dir
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("basic_example.wasm");

    let constructor =
        constructor!(TOKEN_NAME.to_owned(), TOKEN_SYMBOL.to_owned());

    let deployer = account.as_deployer().with_constructor(constructor);
    let address = deployer
        .deploy_wasm(&wasm_path)
        .await
        .expect("contract should be deployed")
        .contract_address;

    address
}
