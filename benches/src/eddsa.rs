use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::SolCall,
};
use alloy_primitives::hex;
use e2e::{receipt, Account};
use openzeppelin_crypto::{
    curve::CurveGroup,
    eddsa::{Signature, SigningKey, VerifyingKey},
};

use crate::{
    report::{ContractReport, FunctionReport},
    Opt,
};

sol!(
    #[sol(rpc)]
   contract EddsaExample {
        #[derive(Debug)]
        function verify(uint[2] verifying_key, uint[3] signature, bytes calldata message) external view returns (bool is_valid);
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("Eddsa", run).await
}

pub async fn run(cache_opt: Opt) -> eyre::Result<Vec<FunctionReport>> {
    let alice = Account::new().await?;
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = EddsaExample::new(contract_addr, &alice_wallet);

    let secret_key = hex!(
        "4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb"
    );
    let signing_key = SigningKey::from_bytes(&secret_key);

    // Verify with signed message.
    let message = b"Sign me!";
    let signature = signing_key.sign(message);

    #[rustfmt::skip]
    let receipts = vec![
        (EddsaExample::verifyCall::SIGNATURE, receipt!(contract.verify(
            encode_verifying_key(signing_key.verifying_key()),
            encode_signature(signature),
            message.into(),
        ))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(account, "eddsa", None, cache_opt).await
}

/// Non-canonical encoding of [`Signature`].
fn encode_signature(signature: Signature) -> [alloy_primitives::U256; 3] {
    let affine_r = signature.R.into_affine();
    [
        affine_r.x.into_bigint().into(),
        affine_r.y.into_bigint().into(),
        signature.s.into_bigint().into(),
    ]
}

/// Non-canonical encoding of [`VerifyingKey`].
fn encode_verifying_key(
    verifying_key: VerifyingKey,
) -> [alloy_primitives::U256; 2] {
    let affine_verifying_key = verifying_key.point.into_affine();
    [
        affine_verifying_key.x.into_bigint().into(),
        affine_verifying_key.y.into_bigint().into(),
    ]
}
