use std::{
    path::{Path, PathBuf},
    process::Command,
};

use alloy::primitives::Address;
use eyre::bail;
use koba::config::Deploy;

use crate::package::Crate;

/// Deploy and activate the contract `contract_name`, which lives in `pkg_dir`,
/// using `rpc_url`, `private_key` and the ABI-encoded constructor `args`.
pub async fn deploy(
    rpc_url: &str,
    private_key: &str,
    args: Option<String>,
) -> eyre::Result<Address> {
    let pkg = Crate::new()?;
    let sol_path: PathBuf = pkg.manifest_dir.join("src/constructor.sol");
    let wasm_path = pkg.wasm;

    let config = Deploy {
        generate_config: koba::config::Generate {
            wasm: wasm_path.clone(),
            sol: sol_path,
            args,
            legacy: true,
        },
        auth: koba::config::PrivateKey {
            private_key_path: None,
            private_key: Some(private_key.to_owned()),
            keystore_path: None,
            keystore_password_path: None,
        },
        endpoint: rpc_url.to_owned(),
        deploy_only: true,
    };

    let address = koba::deploy(&config).await?;
    activate(&wasm_path, rpc_url, private_key, address)?;

    Ok(address)
}

/// Uses `cargo-stylus` to activate a Stylus contract.
fn activate(
    wasm_path: &Path,
    rpc_url: &str,
    private_key: &str,
    address: Address,
) -> eyre::Result<()> {
    let output = Command::new("cargo")
        .arg("stylus")
        .arg("deploy")
        .args(["-e", rpc_url])
        .args(["--wasm-file-path", &wasm_path.to_string_lossy()])
        .args(["--private-key", private_key])
        .args(["--activate-program-address", &address.to_string()])
        .args(["--mode", "activate-only"])
        .output()?;

    if !output.status.success() {
        // If the program is up-to-date, that means that it was activated
        // already, so we can just ignore this error and carry on.
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("ProgramUpToDate") {
            bail!("failed to activate the contract at address {address}");
        }
    }

    Ok(())
}
