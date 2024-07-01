use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::SolConstructor,
    uint,
};
use alloy_primitives::U256;
use e2e::{receipt, Account};
use koba::config::Deploy;

use crate::ArbOtherFields;

const RPC_URL: &str = "http://localhost:8547";

sol!(
    #[sol(rpc)]
    contract Erc20 {
        function name() external view returns (string name);
        function symbol() external view returns (string symbol);
        function decimals() external view returns (uint8 decimals);
        function totalSupply() external view returns (uint256 totalSupply);
        function balanceOf(address account) external view returns (uint256 balance);
        function allowance(address owner, address spender) external view returns (uint256 allowance);

        function cap() public view virtual returns (uint256 cap);

        function mint(address account, uint256 amount) external;
        function burn(uint256 amount) external;
        function burnFrom(address account, uint256 amount) external;

        function transfer(address recipient, uint256 amount) external returns (bool);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);
    }
);

sol!("../examples/erc20/src/constructor.sol");

const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "TTK";
const CAP: U256 = uint!(1_000_000_U256);

pub async fn bench() -> eyre::Result<()> {
    let alice = Account::new().await?;
    let alice_addr = alice.address();
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let bob = Account::new().await?;
    let bob_addr = bob.address();
    let bob_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(bob.signer.clone()))
        .on_http(bob.url().parse()?);

    let contract_addr = deploy(&alice).await;
    let contract = Erc20::new(contract_addr, &alice_wallet);
    let contract_bob = Erc20::new(contract_addr, &bob_wallet);

    println!("Running benches...");
    // IMPORTANT: Order matters!
    let receipts = vec![
        ("name()", receipt!(contract.name())?),
        ("symbol()", receipt!(contract.symbol())?),
        ("decimals()", receipt!(contract.decimals())?),
        ("totalSupply()", receipt!(contract.totalSupply())?),
        ("balanceOf(alice)", receipt!(contract.balanceOf(alice_addr))?),
        (
            "allowance(alice, bob)",
            receipt!(contract.allowance(alice_addr, bob_addr))?,
        ),
        ("cap()", receipt!(contract.cap())?),
        (
            "mint(alice, 10)",
            receipt!(contract.mint(alice_addr, uint!(10_U256)))?,
        ),
        ("burn(1)", receipt!(contract.burn(uint!(1_U256)))?),
        (
            "transfer(bob, 1)",
            receipt!(contract.transfer(bob_addr, uint!(1_U256)))?,
        ),
        (
            "approve(bob, 5)",
            receipt!(contract.approve(bob_addr, uint!(5_U256)))?,
        ),
        (
            "burnFrom(alice, 1)",
            receipt!(contract_bob.burnFrom(alice_addr, uint!(1_U256)))?,
        ),
        (
            "transferFrom(alice, bob, 5)",
            receipt!(contract_bob.transferFrom(
                alice_addr,
                bob_addr,
                uint!(4_U256)
            ))?,
        ),
    ];

    // Calculate the width of the longest function name.
    let max_name_width = receipts
        .iter()
        .max_by_key(|x| x.0.len())
        .expect("should at least bench one function")
        .0
        .len();
    let name_width = max_name_width.max("Function".len());

    // Calculate the total width of the table.
    let total_width = name_width + 3 + 6 + 3 + 6 + 3 + 20 + 4; // 3 for padding, 4 for outer borders

    // Print the table header.
    println!("+{}+", "-".repeat(total_width - 2));
    println!(
        "| {:<width$} | L2 Gas | L1 Gas |        Effective Gas |",
        "Function",
        width = name_width
    );
    println!(
        "|{}+--------+--------+----------------------|",
        "-".repeat(name_width + 2)
    );

    // Print each row.
    for (func_name, receipt) in receipts {
        let l2_gas = receipt.gas_used;
        let arb_fields: ArbOtherFields = receipt.other.deserialize_into()?;
        let l1_gas = arb_fields.gas_used_for_l1.to::<u128>();
        let effective_gas = l2_gas - l1_gas;

        println!(
            "| {:<width$} | {:>6} | {:>6} | {:>20} |",
            func_name,
            l2_gas,
            l1_gas,
            effective_gas,
            width = name_width
        );
    }

    // Print the table footer.
    println!("+{}+", "-".repeat(total_width - 2));

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
