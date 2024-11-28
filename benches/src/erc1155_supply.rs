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
        function safeTransferFrom(address from, address to, uint256 id, uint256 value, bytes memory data) external;
        function safeBatchTransferFrom(address from, address to, uint256[] memory ids, uint256[] memory values, bytes memory data) external;
        function mint(address to, uint256 id, uint256 amount, bytes memory data) external;
        function mintBatch(address to, uint256[] memory ids, uint256[] memory amounts, bytes memory data) external;
        function burn(address account, uint256 id, uint256 value) external;
        function burnBatch(address account, uint256[] memory ids, uint256[] memory values) external;
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

    let bob = Account::new().await?;
    let bob_addr = bob.address();

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = Erc1155Supply::new(contract_addr, &alice_wallet);

    let token_1 = uint!(1_U256);
    let token_2 = uint!(2_U256);
    let token_3 = uint!(3_U256);
    let token_4 = uint!(4_U256);

    let value_1 = uint!(100_U256);
    let value_2 = uint!(200_U256);
    let value_3 = uint!(300_U256);
    let value_4 = uint!(400_U256);

    let ids = vec![token_1, token_2, token_3, token_4];
    let values = vec![value_1, value_2, value_3, value_4];

    let data: alloy_primitives::Bytes = vec![].into();

    // IMPORTANT: Order matters!
    use Erc1155Supply::*;
    #[rustfmt::skip]
    let receipts = vec![
        (mintCall::SIGNATURE, receipt!(contract.mint(alice_addr, token_1, value_1, data.clone()))?),
        (mintBatchCall::SIGNATURE, receipt!(contract.mintBatch(alice_addr, ids.clone(), values.clone(), data.clone()))?),
        (existsCall::SIGNATURE, receipt!(contract.exists(token_1))?),
        (totalSupply_0Call::SIGNATURE, receipt!(contract.totalSupply_0(token_1))?),
        (totalSupply_1Call::SIGNATURE, receipt!(contract.totalSupply_1())?),
        (safeTransferFromCall::SIGNATURE, receipt!(contract.safeTransferFrom(alice_addr, bob_addr, token_1, value_1, data.clone()))?),
        (safeBatchTransferFromCall::SIGNATURE, receipt!(contract.safeBatchTransferFrom(alice_addr, bob_addr, ids.clone(), values.clone(), data.clone()))?),
        (burnCall::SIGNATURE, receipt!(contract.burn(bob_addr, token_1, value_1))?),
        (burnBatchCall::SIGNATURE, receipt!(contract.burnBatch(bob_addr, ids, values))?),
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
