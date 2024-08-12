use alloy::{primitives::Address, rpc::types::TransactionReceipt};
use koba::config::Deploy;

use crate::project::Crate;

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
pub async fn deploy(
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
