use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::{SolCall, SolConstructor},
};
use e2e::{receipt, Account};

use crate::{
    report::{ContractReport, FunctionReport},
    CacheOpt,
};

sol!(
    #[sol(rpc)]
    contract Ownable {
        function owner() external view returns (address owner);
        function renounceOwnership() external onlyOwner;
        function transferOwnership(address newOwner) external;
    }
);

sol!("../examples/ownable/src/constructor.sol");

pub async fn bench() -> eyre::Result<ContractReport> {
    let reports = run_with(CacheOpt::None).await?;
    let report = reports
        .into_iter()
        .try_fold(ContractReport::new("Ownable"), ContractReport::add)?;

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

    let contract = Ownable::new(contract_addr, &alice_wallet);
    let contract_bob = Ownable::new(contract_addr, &bob_wallet);

    // IMPORTANT: Order matters!
    use Ownable::*;
    #[rustfmt::skip]
    let receipts = vec![
        (ownerCall::SIGNATURE, receipt!(contract.owner())?),
        (transferOwnershipCall::SIGNATURE, receipt!(contract.transferOwnership(bob_addr))?),
        (renounceOwnershipCall::SIGNATURE, receipt!(contract_bob.renounceOwnership())?),
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
    let args =
        OwnableExample::constructorCall { initialOwner: account.address() };
    let args = alloy::hex::encode(args.abi_encode());
    crate::deploy(account, "ownable", Some(args), cache_opt).await
}
