use alloy::{
    network::EthereumWallet, primitives::Address, providers::ProviderBuilder,
    signers::local::PrivateKeySigner, sol,
};
use e2e::{Account, Constructor};

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
    let wallet = EthereumWallet::from(signer.clone());
    let rpc_url = RPC_URL.parse().expect("should parse rpc url");
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc_url);
    let account = Account { signer, wallet: provider.clone() };

    let contract_address = deploy(&account).await;

    let contract = BasicToken::new(contract_address, &provider);

    let call_result = contract.name().call().await.unwrap();
    assert_eq!(call_result.name, TOKEN_NAME.to_owned());

    let call_result = contract.symbol().call().await.unwrap();
    assert_eq!(call_result.symbol, TOKEN_SYMBOL.to_owned());
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

    let constructor = Constructor {
        signature: "constructor(string,string)".to_string(),
        args: vec![TOKEN_NAME.to_owned(), TOKEN_SYMBOL.to_owned()],
    };

    let deployer = account.as_deployer().with_constructor(constructor);
    let address = deployer
        .deploy_wasm(&wasm_path)
        .await
        .expect("contract should be deployed")
        .contract_address;

    address
}
