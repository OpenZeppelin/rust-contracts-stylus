use std::path::PathBuf;

use alloy::primitives::Address;
use eyre::Context;
use koba::config::Deploy;

pub fn deploy(
    rpc_url: &str,
    private_key: &str,
    args: Option<String>,
) -> eyre::Result<Address> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // Fine to unwrap here, otherwise a bug in `cargo`.
    let mut sol_path: PathBuf =
        manifest_dir.parse().wrap_err("failed to parse manifest dir path")?;
    sol_path.push("src/constructor.sol");

    let name = env!("CARGO_PKG_NAME");
    let target_dir = std::env::var("CARGO_TARGET_DIR")?;
    // Fine to unwrap here, otherwise a bug in `cargo`.
    let mut wasm_path: PathBuf =
        target_dir.parse().wrap_err("failed to parse target dir path")?;
    wasm_path.push(format!("wasm32-unknown-unknown/release/{name}.wasm"));

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

    let address = koba::deploy(&config)?;
    Ok(address)
}
