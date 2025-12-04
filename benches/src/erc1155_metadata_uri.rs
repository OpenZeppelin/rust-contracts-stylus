use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::{Address, U256},
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
    contract Erc1155MetadataUri {
        function uri(uint256 id) external view returns (string memory uri);
        function setTokenURI(uint256 tokenId, string memory tokenURI) external;
        function setBaseURI(string memory tokenURI) external;
    }
);

const URI: &str = "https://github.com/OpenZeppelin/rust-contracts-stylus";
const BASE_URI: &str = "https://github.com";
const TOKEN_URI: &str = "/some/token/uri";

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("Erc1155MetadataUri", run).await
}

pub async fn run(cache_opt: Opt) -> eyre::Result<Vec<FunctionReport>> {
    let alice = Account::new().await?;
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = Erc1155MetadataUri::new(contract_addr, &alice_wallet);

    let token_id = U256::ONE;

    // IMPORTANT: Order matters!
    use Erc1155MetadataUri::*;
    #[rustfmt::skip]
    let receipts = vec![
        (setTokenURICall::SIGNATURE, receipt!(contract.setTokenURI(token_id, TOKEN_URI.to_owned()))?),
        (setBaseURICall::SIGNATURE, receipt!(contract.setBaseURI(BASE_URI.to_owned()))?),
        (uriCall::SIGNATURE, receipt!(contract.uri(token_id))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(
        account,
        "erc1155-metadata-uri",
        Some(constructor!(URI.to_string())),
        cache_opt,
    )
    .await
}
