use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::SolCall,
    uint,
};
use alloy_primitives::U256;
use e2e::{constructor, receipt, Account};

use crate::{
    report::{ContractReport, FunctionReport},
    Opt,
};

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

const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "TTK";
const CAP: U256 = uint!(1_000_000_U256);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("Erc20", run).await
}

pub async fn run(cache_opt: Opt) -> eyre::Result<Vec<FunctionReport>> {
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

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = Erc20::new(contract_addr, &alice_wallet);
    let contract_bob = Erc20::new(contract_addr, &bob_wallet);

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
        (burnCall::SIGNATURE, receipt!(contract.burn(U256::ONE))?),
        (transferCall::SIGNATURE, receipt!(contract.transfer(bob_addr, U256::ONE))?),
        (approveCall::SIGNATURE, receipt!(contract.approve(bob_addr, uint!(5_U256)))?),
        (burnFromCall::SIGNATURE, receipt!(contract_bob.burnFrom(alice_addr, U256::ONE))?),
        (transferFromCall::SIGNATURE, receipt!(contract_bob.transferFrom(alice_addr, bob_addr, uint!(4_U256)))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(
        account,
        "erc20",
        Some(constructor!(
            TOKEN_NAME.to_string(),
            TOKEN_SYMBOL.to_string(),
            CAP
        )),
        cache_opt,
    )
    .await
}
