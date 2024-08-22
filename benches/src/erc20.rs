use std::process::Command;

use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::{SolCall, SolConstructor},
    uint,
};
use alloy_primitives::U256;
use e2e::{receipt, Account};
use eyre::{bail, Context};

use crate::report::Report;

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

pub async fn bench() -> eyre::Result<Report> {
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

    // Add contract to cache manager. We leave the bid unspecified, since we
    // should have plenty of room to fill the cache (can hold ~4k contracts).
    let status = Command::new("cargo-stylus")
        .arg("cache")
        .args(&["-e", &std::env::var("RPC_URL")?])
        .args(&["--private-key", &format!("0x{}", alice.pk())])
        .args(&["--address", &format!("{}", contract_addr)])
        .status()
        .with_context(|| "`cargo stylus cache` failed:")?;
    if !status.success() {
        bail!("Failed to cache Erc20 contract");
    }

    // IMPORTANT: Order matters!
    use Erc20::*;
    #[rustfmt::skip]
    let receipts = vec![
        (nameCall::SIGNATURE, receipt!(contract.name())?),
        (symbolCall::SIGNATURE, receipt!(contract.symbol())?),
        (decimalsCall::SIGNATURE, receipt!(contract.decimals())?),
        (totalSupplyCall::SIGNATURE, receipt!(contract.totalSupply())?),
        (balanceOfCall::SIGNATURE, receipt!(contract.balanceOf(alice_addr))?),
        (allowanceCall::SIGNATURE, receipt!(contract.allowance(alice_addr, bob_addr))?),
        (capCall::SIGNATURE, receipt!(contract.cap())?),
        (mintCall::SIGNATURE, receipt!(contract.mint(alice_addr, uint!(10_U256)))?),
        (burnCall::SIGNATURE, receipt!(contract.burn(uint!(1_U256)))?),
        (transferCall::SIGNATURE, receipt!(contract.transfer(bob_addr, uint!(1_U256)))?),
        (approveCall::SIGNATURE, receipt!(contract.approve(bob_addr, uint!(5_U256)))?),
        (burnFromCall::SIGNATURE, receipt!(contract_bob.burnFrom(alice_addr, uint!(1_U256)))?),
        (transferFromCall::SIGNATURE, receipt!(contract_bob.transferFrom(alice_addr, bob_addr, uint!(4_U256)))?),
    ];

    let report =
        receipts.into_iter().try_fold(Report::new("Erc20"), Report::add)?;
    Ok(report)
}

async fn deploy(account: &Account) -> Address {
    let args = Erc20Example::constructorCall {
        name_: TOKEN_NAME.to_owned(),
        symbol_: TOKEN_SYMBOL.to_owned(),
        cap_: CAP,
    };
    let args = alloy::hex::encode(args.abi_encode());
    crate::deploy(account, "erc20", Some(args)).await
}
