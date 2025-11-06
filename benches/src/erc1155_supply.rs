use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::{Address, U256},
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
    contract Erc1155Supply {
        function mint(address to, uint256 id, uint256 amount, bytes memory data) external;
        function totalSupply(uint256 id) external view returns (uint256);
        function totalSupply() external view returns (uint256);
        function exists(uint256 id) external view returns (bool);
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("Erc1155Supply", run).await
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

    let contract = Erc1155Supply::new(contract_addr, &alice_wallet);

    let token = U256::ONE;
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

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(account, "erc1155-supply", None, cache_opt).await
}
