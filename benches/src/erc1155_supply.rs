use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::SolCall,
    uint,
};
use e2e::{receipt, Account};

use crate::{
    report::{ContractReport, FunctionReport},
    CacheOpt,
};

sol!(
    #[sol(rpc)]
    contract Erc1155Supply {
        function mint(address to, uint256 id, uint256 amount, bytes memory data) external;
        function totalSupply(uint256 id) external view returns (uint256);
        function totalSupply() external view returns (uint256);
        function exists(uint256 id) external view returns (bool);
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    let reports = run_with(CacheOpt::None).await?;
    let report = reports
        .into_iter()
        .try_fold(ContractReport::new("Erc1155Supply"), ContractReport::add)?;

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

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = Erc1155Supply::new(contract_addr, &alice_wallet);

    let token = uint!(1_U256);
    let value = uint!(100_U256);

    // IMPORTANT: Order matters!
    use Erc1155Supply::*;
    #[rustfmt::skip]
    let receipts = vec![
        (mintCall::SIGNATURE, receipt!(contract.mint(alice_addr, token, value, vec![].into()))?),
        (existsCall::SIGNATURE, receipt!(contract.exists(token))?),
        (totalSupply_0Call::SIGNATURE, receipt!(contract.totalSupply_0(token))?),
        (totalSupply_1Call::SIGNATURE, receipt!(contract.totalSupply_1())?),
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
    crate::deploy(account, "erc1155-supply", None, cache_opt).await
}
