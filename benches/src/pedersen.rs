use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::SolCall,
};
use e2e::{receipt, Account};
use openzeppelin_crypto::arithmetic::uint::{from_str_hex, U256};

use crate::{
    report::{ContractReport, FunctionReport},
    Opt,
};

sol!(
    #[sol(rpc)]
   contract PedersenExample {
        #[derive(Debug)]
        function hash(uint256[2] memory inputs) external view returns (uint256 hash);
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("Pedersen", run).await
}

pub async fn run(cache_opt: Opt) -> eyre::Result<Vec<FunctionReport>> {
    let alice = Account::new().await?;
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = PedersenExample::new(contract_addr, &alice_wallet);

    let input_1: U256 = from_str_hex::<4>(
        "3d937c035c878245caf64531a5756109c53068da139362728feb561405371cb",
    );
    let input_2: U256 = from_str_hex::<4>(
        "208a0a10250e382e1e4bbe2880906c2791bf6275695e02fbbc6aeff9cd8b31a",
    );

    let input_1: alloy_primitives::U256 = input_1.into();
    let input_2: alloy_primitives::U256 = input_2.into();

    #[rustfmt::skip]
    let receipts = vec![
        (PedersenExample::hashCall::SIGNATURE, receipt!(contract.hash([input_1, input_2]))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(account, "pedersen", None, cache_opt).await
}
