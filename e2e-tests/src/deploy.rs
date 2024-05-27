use std::{path::PathBuf, process::Command};

use alloy::primitives::Address;
use eyre::Context;
use koba::config::Deploy;

fn get_workspace_root() -> eyre::Result<String> {
    // dirname "$(cargo locate-project --workspace --message-format plain)"
    let output = Command::new("cargo")
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format")
        .arg("plain")
        .output()
        .wrap_err("should run `cargo locate-project`")?;
    let manifest_path = String::from_utf8_lossy(&output.stdout);
    let manifest_dir = Command::new("dirname")
        .arg(&*manifest_path)
        .output()
        .wrap_err("should run `dirname`")?;

    Ok(String::from_utf8_lossy(&manifest_dir.stdout).to_string())
}

pub async fn deploy(
    pkg_name: &str,
    pkg_dir: &str,
    rpc_url: &str,
    private_key: &str,
    args: Option<String>,
) -> eyre::Result<Address> {
    println!("pkg_name {:?}", pkg_name);
    println!("pkg_dir {:?}", pkg_dir);
    // Fine to unwrap here, otherwise a bug in `cargo`.
    let manifest_dir = get_workspace_root()?
        .parse::<PathBuf>()
        .wrap_err("failed to parse manifest dir path")?;
    println!("manifest_dir {:?}", manifest_dir);
    // Fine to unwrap here, otherwise a bug in `cargo`.
    let pkg_dir = pkg_dir
        .parse::<PathBuf>()
        .wrap_err("failed to parse manifest dir path")?;
    let sol_path: PathBuf = pkg_dir.join("src/constructor.sol");
    println!("sol_path {:?}", sol_path);

    // This is super flaky because it assumes we are in a workspace. This is
    // fine for now since we only use this function in our tests, but if we
    // publish this as a crate, we need to account for the other cases.
    let target_dir = manifest_dir.join("../target");
    // Fine to unwrap here, otherwise a bug in `cargo`.
    let wasm_path: PathBuf = target_dir
        .join(format!("wasm32-unknown-unknown/release/{pkg_name}.wasm"));

    let config = Deploy {
        generate_config: koba::config::Generate {
            wasm: wasm_path,
            sol: sol_path,
            args,
        },
        auth: koba::config::PrivateKey {
            private_key_path: None,
            private_key: Some(private_key.to_owned()),
            keystore_path: None,
            keystore_password_path: None,
        },
        endpoint: rpc_url.to_owned(),
    };

    println!("{:?}", config);
    let address = koba::deploy(&config).await?;
    Ok(address)
}
