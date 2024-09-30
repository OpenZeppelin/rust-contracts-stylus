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

use crate::{
    report::{ContractReport, FunctionReport},
    CacheOpt,
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

sol!("../examples/erc20/src/constructor.sol");

const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "TTK";
const CAP: U256 = uint!(1_000_000_U256);

pub async fn bench() -> eyre::Result<ContractReport> {
    let reports = run_with(CacheOpt::None).await?;
    let report = reports
        .into_iter()
        .try_fold(ContractReport::new("Erc20"), ContractReport::add)?;

    let cached_reports = run_with(CacheOpt::Bid(0)).await?;
    let report = cached_reports
        .into_iter()
        .try_fold(report, ContractReport::add_cached)?;

    Ok(report)
}

pub async fn run_with(
    cache_opt: CacheOpt,
) -> eyre::Result<Vec<FunctionReport>> {
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
        (burnCall::SIGNATURE, receipt!(contract.burn(uint!(1_U256)))?),
        (transferCall::SIGNATURE, receipt!(contract.transfer(bob_addr, uint!(1_U256)))?),
        (approveCall::SIGNATURE, receipt!(contract.approve(bob_addr, uint!(5_U256)))?),
        (burnFromCall::SIGNATURE, receipt!(contract_bob.burnFrom(alice_addr, uint!(1_U256)))?),
        (transferFromCall::SIGNATURE, receipt!(contract_bob.transferFrom(alice_addr, bob_addr, uint!(4_U256)))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(
    account: &Account,
    cache_opt: CacheOpt,
) -> eyre::Result<Address> {
    let args = Erc20Example::constructorCall {
        name_: TOKEN_NAME.to_owned(),
        symbol_: TOKEN_SYMBOL.to_owned(),
        cap_: CAP,
    };
    let args = alloy::hex::encode(args.abi_encode());
    crate::deploy(account, "erc20", Some(args), cache_opt).await
}
