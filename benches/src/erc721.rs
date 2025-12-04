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
    contract Erc721 {
        function balanceOf(address owner) external view returns (uint256 balance);
        function approve(address to, uint256 tokenId) external;
        function getApproved(uint256 tokenId) external view returns (address approved);
        function isApprovedForAll(address owner, address operator) external view returns (bool approved);
        function ownerOf(uint256 tokenId) external view returns (address ownerOf);
        function safeTransferFrom(address from, address to, uint256 tokenId) external;
        function setApprovalForAll(address operator, bool approved) external;
        function totalSupply() external view returns (uint256 totalSupply);
        function transferFrom(address from, address to, uint256 tokenId) external;
        function mint(address to, uint256 tokenId) external;
        function burn(uint256 tokenId) external;
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("Erc721", run).await
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

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = Erc721::new(contract_addr, &alice_wallet);

    let token_1 = U256::ONE;
    let token_2 = uint!(2_U256);
    let token_3 = uint!(3_U256);
    let token_4 = uint!(4_U256);

    receipt!(contract.mint(alice_addr, token_2))?;
    receipt!(contract.mint(alice_addr, token_3))?;
    receipt!(contract.mint(alice_addr, token_4))?;

    // IMPORTANT: Order matters!
    use Erc721::*;
    #[rustfmt::skip]
    let receipts = vec![
        (balanceOfCall::SIGNATURE, receipt!(contract.balanceOf(alice_addr))?),
        (approveCall::SIGNATURE, receipt!(contract.approve(bob_addr, token_2))?),
        (getApprovedCall::SIGNATURE, receipt!(contract.getApproved(token_2))?),
        (isApprovedForAllCall::SIGNATURE, receipt!(contract.isApprovedForAll(alice_addr, bob_addr))?),
        (ownerOfCall::SIGNATURE, receipt!(contract.ownerOf(token_2))?),
        (safeTransferFromCall::SIGNATURE, receipt!(contract.safeTransferFrom(alice_addr, bob_addr, token_3))?),
        (setApprovalForAllCall::SIGNATURE, receipt!(contract.setApprovalForAll(bob_addr, true))?),
        (totalSupplyCall::SIGNATURE, receipt!(contract.totalSupply())?),
        (transferFromCall::SIGNATURE, receipt!(contract.transferFrom(alice_addr, bob_addr, token_4))?),
        (mintCall::SIGNATURE, receipt!(contract.mint(alice_addr, token_1))?),
        (burnCall::SIGNATURE, receipt!(contract.burn(token_1))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(account, "erc721", None, cache_opt).await
}
