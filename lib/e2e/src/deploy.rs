use std::{
    path::{Path, PathBuf},
    process::Command,
};

use alloy::primitives::Address;
use eyre::{bail, Context};
use koba::config::Deploy;

use crate::system::get_workspace_root;

/// Deploy and activate the contract `contract_name`, which lives in `pkg_dir`,
/// using `rpc_url`, `private_key` and the ABI-encoded constructor `args`.
pub async fn deploy(
    contract_name: &str,
    pkg_dir: &str,
    rpc_url: &str,
    private_key: &str,
    args: Option<String>,
) -> eyre::Result<Address> {
    // Fine to unwrap here, otherwise a bug in `cargo`.
    let pkg_dir = pkg_dir
        .parse::<PathBuf>()
        .wrap_err("failed to parse manifest dir path")?;
    let sol_path: PathBuf = pkg_dir.join("src/constructor.sol");

    // This is super flaky because it assumes we are in a workspace. This is
    // fine for now since we only use this function in our tests, but if we
    // publish this as a crate, we need to account for the other cases.
    let manifest_dir = get_workspace_root()?;
    let wasm_path: PathBuf = manifest_dir.join(format!(
        "target/wasm32-unknown-unknown/release/{contract_name}.wasm"
    ));

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
        // Only activate once.
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("ProgramUpToDate") {
            bail!("failed to activate the contract at address {address}");
        }
    }

    Ok(())
}
