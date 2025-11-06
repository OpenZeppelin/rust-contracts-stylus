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
    contract Erc1155 {
        function balanceOf(address account, uint256 id) external view returns (uint256 balance);
        function balanceOfBatch(address[] accounts, uint256[] ids) external view returns (uint256[] memory balances);
        function isApprovedForAll(address account, address operator) external view returns (bool approved);
        function setApprovalForAll(address operator, bool approved) external;
        function safeTransferFrom(address from, address to, uint256 id, uint256 value, bytes memory data) external;
        function safeBatchTransferFrom(address from, address to, uint256[] memory ids, uint256[] memory values, bytes memory data) external;
        function mint(address to, uint256 id, uint256 amount, bytes memory data) external;
        function mintBatch(address to, uint256[] memory ids, uint256[] memory amounts, bytes memory data) external;
        function burn(address account, uint256 id, uint256 value) external;
        function burnBatch(address account, uint256[] memory ids, uint256[] memory values) external;
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("Erc1155", run).await
}

pub async fn run(cache_opt: Opt) -> eyre::Result<Vec<FunctionReport>> {
    let alice = Account::new().await?;
    let alice_addr = alice.address();
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

    let contract = Erc1155::new(contract_addr, &alice_wallet);
    let contract_bob = Erc1155::new(contract_addr, &bob_wallet);

    let token_1 = U256::ONE;
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
        (safeBatchTransferFromCall::SIGNATURE, receipt!(contract.safeBatchTransferFrom(alice_addr, bob_addr, ids.clone(), values.clone(), data))?),
        // We should burn Bob's tokens on behalf of Bob, not Alice.
        (burnCall::SIGNATURE, receipt!(contract_bob.burn(bob_addr, token_1, value_1))?),
        (burnBatchCall::SIGNATURE, receipt!(contract_bob.burnBatch(bob_addr, ids, values))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(account, "erc1155", None, cache_opt).await
}
