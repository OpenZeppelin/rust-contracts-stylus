use alloy::{
    primitives::Address, providers::WalletProvider, sol,
    sol_types::SolConstructor, uint,
};
use alloy_primitives::U256;
use e2e::Account;
use koba::config::Deploy;

const RPC_URL: &str = "http://localhost:8547";

sol!(
    #[sol(rpc)]
    contract Erc20 {
        function name() external view returns (string name);
        function symbol() external view returns (string symbol);
        function decimals() external view returns (uint8 decimals);
        function totalSupply() external view returns (uint256 totalSupply);
        function balanceOf(address account) external view returns (uint256 balance);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256 allowance);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        function cap() public view virtual returns (uint256 cap);

        function mint(address account, uint256 amount) external;
        function burn(uint256 amount) external;
        function burnFrom(address account, uint256 amount) external;

        error ERC20ExceededCap(uint256 increased_supply, uint256 cap);
        error ERC20InvalidCap(uint256 cap);

        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
        error ERC20InvalidSender(address sender);
        error ERC20InvalidReceiver(address receiver);
        error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
        error ERC20InvalidSpender(address spender);

        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 value);

        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }
);

sol!("../examples/erc20/src/constructor.sol");

const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "TTK";
const CAP: U256 = uint!(1_000_000_U256);

pub async fn bench() -> eyre::Result<()> {
    let alice = Account::new().await?;
    let bob = Account::new().await?;
    let contract_addr = deploy(&alice).await;
    let contract = Erc20::new(contract_addr, &alice.wallet);

    let gas = contract.name().estimate_gas().await?;
    println!("name(): {gas}");
    let gas = contract.symbol().estimate_gas().await?;
    println!("symbol(): {gas}");
    let gas = contract.cap().estimate_gas().await?;
    println!("cap(): {gas}");
    let gas = contract.decimals().estimate_gas().await?;
    println!("decimals(): {gas}");
    let gas = contract.totalSupply().estimate_gas().await?;
    println!("totalSupply(): {gas}");
    let gas = contract.balanceOf(alice.address()).estimate_gas().await?;
    println!("balanceOf(account): {gas}");

    let gas =
        contract.mint(alice.address(), uint!(100_U256)).estimate_gas().await?;
    println!("mint(account, amount): {gas}");

    let _ = contract
        .mint(alice.address(), uint!(100_U256))
        .send()
        .await?
        .watch()
        .await?;
    let gas = contract
        .burn(uint!(100_U256))
        .from(alice.address())
        .estimate_gas()
        .await?;
    println!("burn(amount): {gas}");
    let gas = contract
        .transfer(bob.address(), uint!(1_U256))
        .from(alice.address())
        .estimate_gas()
        .await?;
    println!("transfer(account, amount): {gas}");

    Ok(())
}

async fn deploy(account: &Account) -> Address {
    let args = Erc20Example::constructorCall {
        name_: TOKEN_NAME.to_owned(),
        symbol_: TOKEN_SYMBOL.to_owned(),
        cap_: CAP,
    };
    let args = alloy::hex::encode(args.abi_encode());

    let manifest_dir =
        std::env::current_dir().expect("should get current dir from env");

    let wasm_path = manifest_dir
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("erc20_example.wasm");
    let sol_path = manifest_dir
        .join("examples")
        .join("erc20")
        .join("src")
        .join("constructor.sol");

    let pk = account.pk();
    let config = Deploy {
        generate_config: koba::config::Generate {
            wasm: wasm_path.clone(),
            sol: sol_path,
            args: Some(args),
            legacy: false,
        },
        auth: koba::config::PrivateKey {
            private_key_path: None,
            private_key: Some(pk),
            keystore_path: None,
            keystore_password_path: None,
        },
        endpoint: RPC_URL.to_owned(),
        deploy_only: false,
    };

    koba::deploy(&config).await.expect("should deploy contract")
}
