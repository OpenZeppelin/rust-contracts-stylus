use alloy::{
    hex,
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
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
    contract AccessControl {
        function hasRole(bytes32 role, address account) public view virtual returns (bool hasRole);
        function getRoleAdmin(bytes32 role) public view virtual returns (bytes32 role);
        function grantRole(bytes32 role, address account) public virtual;
        function revokeRole(bytes32 role, address account) public virtual;
        function renounceRole(bytes32 role, address callerConfirmation) public virtual;
        function setRoleAdmin(bytes32 role, bytes32 adminRole) public virtual;
    }
);

const DEFAULT_ADMIN_ROLE: [u8; 32] = [0; 32];

// There's no way to query constants of a Stylus contract, so this one is
// hard-coded :(
const ROLE: [u8; 32] =
    keccak_const::Keccak256::new().update(b"TRANSFER_ROLE").finalize();
const NEW_ADMIN_ROLE: [u8; 32] =
    hex!("879ce0d4bfd332649ca3552efe772a38d64a315eb70ab69689fd309c735946b5");

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("AccessControl", run).await
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

    let contract = AccessControl::new(contract_addr, &alice_wallet);
    let contract_bob = AccessControl::new(contract_addr, &bob_wallet);

    // IMPORTANT: Order matters!
    use AccessControl::*;
    #[rustfmt::skip]
    let receipts = vec![
        (hasRoleCall::SIGNATURE, receipt!(contract.hasRole(DEFAULT_ADMIN_ROLE.into(), alice_addr))?),
        (getRoleAdminCall::SIGNATURE, receipt!(contract.getRoleAdmin(ROLE.into()))?),
        (revokeRoleCall::SIGNATURE, receipt!(contract.revokeRole(ROLE.into(), alice_addr))?),
        (grantRoleCall::SIGNATURE, receipt!(contract.grantRole(ROLE.into(), bob_addr))?),
        (renounceRoleCall::SIGNATURE, receipt!(contract_bob.renounceRole(ROLE.into(), bob_addr))?),
        (setRoleAdminCall::SIGNATURE, receipt!(contract.setRoleAdmin(ROLE.into(), NEW_ADMIN_ROLE.into()))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(
        account,
        "access-control",
        Some(constructor!(account.address())),
        cache_opt,
    )
    .await
}
