use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::{SolCall, SolConstructor},
    uint,
};
use e2e::{receipt, Account};

use crate::{
    report::{ContractReport, FunctionReport},
    CacheOpt,
};

sol!(
    #[sol(rpc)]
    contract Erc1155 {
        function balanceOf(address account, uint256 id) external view returns (uint256 balance);
        function balanceOfBatch(address[] accounts, uint256[] ids) external view returns (uint256[] memory balances);
        function isApprovedForAll(address account, address operator) external view returns (bool approved);
        function setApprovalForAll(address operator, bool approved) external;
        function safeTransferFrom(address from, address to, uint256 id, uint256 value, bytes memory data) external;
        function safeBatchTransferFrom(address from, address to, uint256[] memory ids, uint256[] memory values, bytes memory data) external;
        function mint(address to, uint256 id, uint256 amount, bytes memory data) external;
        function mintBatch(address to, uint256[] memory ids, uint256[] memory amounts, bytes memory data) external;
        function uri(uint256 id) external view returns (string memory uri);
        function setURI(string memory newUri) external;
    }
);

sol!("../examples/erc1155/src/constructor.sol");

const URI: &str = "https://github.com/OpenZeppelin/rust-contracts-stylus";
const NEW_URI: &str =
    "https://new.github.com/OpenZeppelin/rust-contracts-stylus";

pub async fn bench() -> eyre::Result<ContractReport> {
    let reports = run_with(CacheOpt::None).await?;
    let report = reports
        .into_iter()
        .try_fold(ContractReport::new("Erc1155"), ContractReport::add)?;

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

    let contract = Erc1155::new(contract_addr, &alice_wallet);

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
    use Erc1155::*;
    #[rustfmt::skip]
    let receipts = vec![
        (mintCall::SIGNATURE, receipt!(contract.mint(alice_addr, token_1, value_1, data.clone()))?),
        (mintBatchCall::SIGNATURE, receipt!(contract.mintBatch(alice_addr, ids.clone(), values.clone(), data.clone()))?),
        (balanceOfCall::SIGNATURE, receipt!(contract.balanceOf(alice_addr, token_1))?),
        (balanceOfBatchCall::SIGNATURE, receipt!(contract.balanceOfBatch(vec![alice_addr, bob_addr], vec![token_1, token_2]))?),
        (setApprovalForAllCall::SIGNATURE, receipt!(contract.setApprovalForAll(bob_addr, true))?),
        (isApprovedForAllCall::SIGNATURE, receipt!(contract.isApprovedForAll(alice_addr, bob_addr))?),
        (safeTransferFromCall::SIGNATURE, receipt!(contract.safeTransferFrom(alice_addr, bob_addr, token_1, value_1, data.clone()))?),
        (safeBatchTransferFromCall::SIGNATURE, receipt!(contract.safeBatchTransferFrom(alice_addr, bob_addr, ids, values, data.clone()))?),
        (uriCall::SIGNATURE, receipt!(contract.uri(token_1))?),
        (setURICall::SIGNATURE, receipt!(contract.setURI(NEW_URI.to_owned()))?)
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
    let args = Erc1155Example::constructorCall { uri_: URI.to_owned() };
    let args = alloy::hex::encode(args.abi_encode());
    crate::deploy(account, "erc1155", Some(args), cache_opt).await
}
