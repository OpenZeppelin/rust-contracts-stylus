use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::SolCall,
};
use e2e::{constructor, receipt, Account};

use crate::{
    report::{ContractReport, FunctionReport},
    Opt,
};

sol!(
    #[sol(rpc)]
    contract Ownable {
        function owner() external view returns (address owner);
        function renounceOwnership() external onlyOwner;
        function transferOwnership(address newOwner) external;
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("Ownable", run).await
}

pub async fn run(cache_opt: Opt) -> eyre::Result<Vec<FunctionReport>> {
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

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(
        account,
        "ownable",
        Some(constructor!(account.address())),
        cache_opt,
    )
    .await
}
