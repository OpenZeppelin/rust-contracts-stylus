use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::SolCall,
};
use alloy_primitives::uint;
use e2e::{receipt, Account};

use crate::{
    report::{ContractReport, FunctionReport},
    Opt,
};

sol!(
    #[sol(rpc)]
   contract MathExample {
        #[derive(Debug)]
        function compute(uint256[2] memory inputs) external view returns (uint256 hash);
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("oz_crypto::MathExample", run).await
}

pub async fn run(cache_opt: Opt) -> eyre::Result<Vec<FunctionReport>> {
    let alice = Account::new().await?;
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = MathExample::new(contract_addr, &alice_wallet);

    #[rustfmt::skip]
    let receipts = vec![
        (MathExample::computeCall::SIGNATURE, receipt!(contract.compute([uint!(123456_U256), uint!(123456789_U256)]))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(account, "oz-crypto", None, cache_opt).await
}
