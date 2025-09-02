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
    contract Erc6909 {
        function balanceOf(address owner, uint256 id) external view returns (uint256 balance);
        function allowance(address owner, address spender, uint256 id) external view returns (uint256 allowance);
        function isOperator(address owner, address spender) external view returns (bool approved);
        function approve(address spender, uint256 id, uint256 amount) external returns (bool);
        function setOperator(address spender, bool approved) external returns (bool);
        function transfer(address receiver, uint256 id, uint256 amount) external returns (bool);
        function transferFrom(address sender, address receiver, uint256 id, uint256 amount) external returns (bool);
        function mint(address to, uint256 id, uint256 amount) external;
        function burn(address from, uint256 id, uint256 amount) external;
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("Erc6909", run).await
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

    let contract = Erc6909::new(contract_addr, &alice_wallet);
    let contract_bob = Erc6909::new(contract_addr, &bob_wallet);

    let token_id = uint!(1_U256);
    let amount = uint!(100_U256);
    let one = uint!(1_U256);

    // IMPORTANT: Order matters!
    use Erc6909::*;
    #[rustfmt::skip]
    let receipts = vec![
        (mintCall::SIGNATURE, receipt!(contract.mint(alice_addr, token_id, amount))?),
        (balanceOfCall::SIGNATURE, receipt!(contract.balanceOf(alice_addr, token_id))?),
        (allowanceCall::SIGNATURE, receipt!(contract.allowance(alice_addr, bob_addr, token_id))?),
        (isOperatorCall::SIGNATURE, receipt!(contract.isOperator(alice_addr, bob_addr))?),
        (setOperatorCall::SIGNATURE, receipt!(contract.setOperator(bob_addr, true))?),
        (transferCall::SIGNATURE, receipt!(contract.transfer(bob_addr, token_id, one))?),
        (approveCall::SIGNATURE, receipt!(contract.approve(bob_addr, token_id, one))?),
        (transferFromCall::SIGNATURE, receipt!(contract_bob.transferFrom(alice_addr, bob_addr, token_id, one))?),
        (burnCall::SIGNATURE, receipt!(contract.burn(alice_addr, token_id, one))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(account, "erc6909", None, cache_opt).await
}
