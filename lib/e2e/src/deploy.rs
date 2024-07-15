use alloy::primitives::Address;
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
) -> eyre::Result<Address> {
    let pkg = Crate::new()?;
    let sol_path = pkg.manifest_dir.join("src/constructor.sol");
    let wasm_path = pkg.wasm;

    let config = Deploy {
        generate_config: koba::config::Generate {
            wasm: wasm_path.clone(),
            sol: Some(sol_path),
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
    };

    let address = koba::deploy(&config).await?;
    Ok(address)
}
