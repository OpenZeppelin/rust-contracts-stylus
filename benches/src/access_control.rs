use alloy::{
    hex,
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::SolConstructor,
};
use e2e::{receipt, Account};

use crate::{report::Report, ArbOtherFields};

sol!(
    #[sol(rpc)]
    contract AccessControl {
        constructor();

        function hasRole(bytes32 role, address account) public view virtual returns (bool hasRole);
        function getRoleAdmin(bytes32 role) public view virtual returns (bytes32 role);
        function grantRole(bytes32 role, address account) public virtual;
        function revokeRole(bytes32 role, address account) public virtual;
        function renounceRole(bytes32 role, address callerConfirmation) public virtual;
        function setRoleAdmin(bytes32 role, bytes32 adminRole) public virtual;
    }
);

const DEFAULT_ADMIN_ROLE: [u8; 32] =
    openzeppelin_stylus::access::control::AccessControl::DEFAULT_ADMIN_ROLE;
// There's no way to query constants of a Stylus contract, so this one is
// hard-coded :(
const ROLE: [u8; 32] =
    keccak_const::Keccak256::new().update(b"TRANSFER_ROLE").finalize();
const NEW_ADMIN_ROLE: [u8; 32] =
    hex!("879ce0d4bfd332649ca3552efe772a38d64a315eb70ab69689fd309c735946b5");

pub async fn bench() -> eyre::Result<Report> {
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

    let contract_addr = deploy(&alice).await;
    let contract = AccessControl::new(contract_addr, &alice_wallet);
    let contract_bob = AccessControl::new(contract_addr, &bob_wallet);

    // IMPORTANT: Order matters!
    #[rustfmt::skip]
    let receipts = vec![
        ("hasRole(DEFAULT_ADMIN_ROLE, alice)", receipt!(contract.hasRole(DEFAULT_ADMIN_ROLE.into(), alice_addr))?),
        ("getRoleAdmin(ROLE)", receipt!(contract.getRoleAdmin(ROLE.into()))?),
        ("revokeRole(ROLE, alice)", receipt!(contract.revokeRole(ROLE.into(), alice_addr))?),
        ("grantRole(ROLE, bob)", receipt!(contract.grantRole(ROLE.into(), bob_addr))?),
        ("renounceRole(ROLE, bob)", receipt!(contract_bob.renounceRole(ROLE.into(), bob_addr))?),
        ("setRoleAdmin(ROLE, NEW_ADMIN_ROLE)", receipt!(contract.setRoleAdmin(ROLE.into(), NEW_ADMIN_ROLE.into()))?),
    ];

    let mut report = Report::new("AccessControl");
    for (signature, receipt) in receipts {
        let l2_gas = receipt.gas_used;
        let arb_fields: ArbOtherFields = receipt.other.deserialize_into()?;
        let l1_gas = arb_fields.gas_used_for_l1.to::<u128>();
        let effective_gas = l2_gas - l1_gas;

        report.add(signature, effective_gas);
    }

    Ok(report)
}

async fn deploy(account: &Account) -> Address {
    let args = AccessControl::constructorCall {};
    let args = alloy::hex::encode(args.abi_encode());
    crate::deploy(account, "access-control", Some(args)).await
}
