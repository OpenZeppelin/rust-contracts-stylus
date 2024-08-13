use alloy::{
    primitives::Address, rpc::types::TransactionReceipt,
    sol_types::SolConstructor,
};
use koba::config::Deploy;

use crate::{project::Crate, Account};

/// Deploy and activate the contract implemented as `#[entrypoint]` in the
/// current crate using `account` and optional `constructor` parameter.
///
/// # Errors
///
/// May error if:
///
/// - Unable to collect information about the crate required for deployment.
/// - `koba::deploy` errors.
pub async fn deploy<C: SolConstructor>(
    account: &Account,
    constructor: Option<C>,
) -> eyre::Result<TransactionReceipt> {
    deploy_inner(
        account.url(),
        &account.pk(),
        constructor.map(|c| alloy::hex::encode(c.abi_encode())),
    )
    .await
}

/// Deploy and activate the contract implemented as `#[entrypoint]` in the
/// current crate using `rpc_url`, `private_key` and the ABI-encoded constructor
/// `args`.
///
/// # Errors
///
/// May error if:
///
/// - Unable to collect information about the crate required for deployment.
/// - `koba::deploy` errors.
async fn deploy_inner(
    rpc_url: &str,
    private_key: &str,
    args: Option<String>,
) -> eyre::Result<TransactionReceipt> {
    let pkg = Crate::new()?;
    let sol_path = pkg.manifest_dir.join("src/constructor.sol");
    let wasm_path = pkg.wasm;
    let has_constructor = args.is_some();

    let config = Deploy {
        generate_config: koba::config::Generate {
            wasm: wasm_path.clone(),
            sol: if has_constructor { Some(sol_path) } else { None },
            args,
            legacy: false,
        },
        auth: koba::config::PrivateKey {
            private_key_path: None,
            private_key: Some(private_key.to_owned()),
            keystore_path: None,
            keystore_password_path: None,
        },
        endpoint: rpc_url.to_owned(),
        deploy_only: false,
        quiet: false,
    };

    let receipt = koba::deploy(&config).await?;
    Ok(receipt)
}
