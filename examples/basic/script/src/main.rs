use alloy::{
    network::EthereumWallet, primitives::Address, providers::ProviderBuilder,
    signers::local::PrivateKeySigner, sol, sol_types::SolConstructor,
};
use koba::config::Deploy;

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
    let contract_address = deploy().await;

    // WARNING: Please use a more secure method for storing your privaket key
    // than a string at the top of this file. The following code is for testing
    // purposes only.
    let signer = PRIVATE_KEY
        .parse::<PrivateKeySigner>()
        .expect("should parse the private key");
    let wallet = EthereumWallet::from(signer);

    let rpc_url = RPC_URL.parse().expect("should parse rpc url");
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc_url);

    let contract = BasicToken::new(contract_address, &provider);

    let call_result = contract.name().call().await.unwrap();
    assert_eq!(call_result.name, TOKEN_NAME.to_owned());

    let call_result = contract.symbol().call().await.unwrap();
    assert_eq!(call_result.symbol, TOKEN_SYMBOL.to_owned());
}

/// Deploy a `BasicToken` contract to `RPC_URL` using `koba`.
async fn deploy() -> Address {
    let args = BasicToken::constructorCall {
        name_: TOKEN_NAME.to_owned(),
        symbol_: TOKEN_SYMBOL.to_owned(),
    };
    let args = alloy::hex::encode(args.abi_encode());

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
    let sol_path = manifest_dir
        .join("examples")
        .join("basic")
        .join("token")
        .join("src")
        .join("constructor.sol");

    let config = Deploy {
        generate_config: koba::config::Generate {
            wasm: wasm_path.clone(),
            sol: Some(sol_path),
            args: Some(args),
            legacy: false,
        },
        auth: koba::config::PrivateKey {
            private_key_path: None,
            private_key: Some(PRIVATE_KEY.to_owned()),
            keystore_path: None,
            keystore_password_path: None,
        },
        endpoint: RPC_URL.to_owned(),
        deploy_only: false,
        quiet: false,
    };

    koba::deploy(&config).await.expect("should deploy contract")
}
