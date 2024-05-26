use std::path::PathBuf;

use alloy::primitives::Address;
use eyre::Context;
use koba::config::Deploy;

pub fn deploy(
    rpc_url: &str,
    private_key: &str,
    args: Option<String>,
) -> eyre::Result<Address> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR")
        .parse::<PathBuf>()
        .wrap_err("failed to parse manifest dir path")?;
    // Fine to unwrap here, otherwise a bug in `cargo`.
    let sol_path: PathBuf = manifest_dir.join("src/constructor.sol");

    let name = env!("CARGO_PKG_NAME");
    // This is super flaky because it assumes we are in a workspace. This is
    // fine for now since we only use this function in our tests, but if we
    // publish this as a crate, we need to account for the other cases.
    let target_dir = manifest_dir.join("../target");
    // Fine to unwrap here, otherwise a bug in `cargo`.
    let wasm_path: PathBuf =
        target_dir.join(format!("wasm32-unknown-unknown/release/{name}.wasm"));

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
