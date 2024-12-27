use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::SolCall,
};
use alloy_primitives::bytes;
use e2e::{receipt, Account};

use crate::{
    report::{ContractReport, FunctionReport},
    Opt,
};

sol!(
    #[sol(rpc)]
   contract PoseidonExample {
        #[derive(Debug)]
        function hash(bytes calldata data) external view returns (bytes32 hash);
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("renegades::Poseidon", run).await
}

pub async fn run(cache_opt: Opt) -> eyre::Result<Vec<FunctionReport>> {
    let alice = Account::new().await?;
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = PoseidonExample::new(contract_addr, &alice_wallet);

    #[rustfmt::skip]
    let receipts = vec![
        (PoseidonExample::hashCall::SIGNATURE, receipt!(contract.hash(bytes!("deadbeef")))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(account, "poseidon-renegades", None, cache_opt).await
}
