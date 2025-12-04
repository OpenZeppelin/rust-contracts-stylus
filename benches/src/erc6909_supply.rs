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
    Opt,
};

sol!(
    #[sol(rpc)]
    contract Erc6909TokenSupply {
        function totalSupply(uint256 id) external view returns (uint256 totalSupply);
        function mint(address to, uint256 id, uint256 amount) external;
        function burn(address from, uint256 id, uint256 amount) external;
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("Erc6909TokenSupply", run).await
}

pub async fn run(cache_opt: Opt) -> eyre::Result<Vec<FunctionReport>> {
    let alice = Account::new().await?;
    let alice_addr = alice.address();
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = Erc6909TokenSupply::new(contract_addr, &alice_wallet);

    let token_id = uint!(1_U256);
    let amount = uint!(100_U256);

    // IMPORTANT: Order matters!
    use Erc6909TokenSupply::*;
    #[rustfmt::skip]
    let receipts = vec![
        (mintCall::SIGNATURE, receipt!(contract.mint(alice_addr, token_id, amount))?),
        (totalSupplyCall::SIGNATURE, receipt!(contract.totalSupply(token_id))?),
        (burnCall::SIGNATURE, receipt!(contract.burn(alice_addr, token_id, amount))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(account, "erc6909-supply", None, cache_opt).await
}
