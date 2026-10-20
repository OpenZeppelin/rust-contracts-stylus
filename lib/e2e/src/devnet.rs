//! Discovery of the local Stylus devnet.
//!
//! The devnet is a nitro dev node started with `stylus_tools::devnet::Node`
//! (see the `devnet` binary of this crate and `scripts/devnet.sh`). The node
//! runs on a random host port, so the daemon writes its RPC URL to
//! `target/devnet/rpc`; tests read that file unless `RPC_URL` is set.
use std::{
    fs,
    path::{Path, PathBuf},
};

use alloy::primitives::Address;
use eyre::{Context, ContextCompat};
pub use stylus_tools::devnet::DEVNET_PRIVATE_KEY;

/// Environment variable that overrides the RPC endpoint of the devnet.
pub const RPC_URL_ENV_VAR: &str = "RPC_URL";

/// Address of the `StylusDeployer` contract on the devnet.
///
/// It is the same CREATE2 address as on Arbitrum One, Nova and Sepolia.
pub const STYLUS_DEPLOYER: Address =
    stylus_tools::devnet::addresses::STYLUS_DEPLOYER;

/// Root of this repository, resolved at compile time.
pub(crate) fn workspace_root() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    root.canonicalize().unwrap_or(root)
}

/// Cargo's target directory for this workspace.
pub(crate) fn target_dir() -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map_or_else(|| workspace_root().join("target"), PathBuf::from)
}

/// Directory where the devnet daemon keeps its RPC URL and pid.
#[must_use]
pub fn devnet_dir() -> PathBuf {
    target_dir().join("devnet")
}

/// File holding the RPC URL of the running devnet.
#[must_use]
pub fn rpc_file() -> PathBuf {
    devnet_dir().join("rpc")
}

/// File holding the pid of the devnet daemon.
#[must_use]
pub fn pid_file() -> PathBuf {
    devnet_dir().join("pid")
}

/// The RPC endpoint of the devnet.
///
/// Reads `RPC_URL` from the environment, then the file written by the devnet
/// daemon.
///
/// # Errors
///
/// If neither is available.
pub fn rpc_url() -> eyre::Result<String> {
    if let Ok(url) = std::env::var(RPC_URL_ENV_VAR) {
        return Ok(url);
    }
    let path = rpc_file();
    let url = fs::read_to_string(&path)
        .with_context(|| {
            format!(
                "no devnet found: set {RPC_URL_ENV_VAR} or start the devnet \
                 with `./scripts/devnet.sh up` ({} is missing)",
                path.display()
            )
        })?
        .trim()
        .to_owned();
    eyre::ensure!(!url.is_empty(), "{} is empty", path.display());
    Ok(url)
}

/// The pid of the devnet daemon, if it wrote one.
///
/// # Errors
///
/// If the pid file is missing or corrupt.
pub fn daemon_pid() -> eyre::Result<u32> {
    let path = pid_file();
    fs::read_to_string(&path)
        .with_context(|| format!("{} is missing", path.display()))?
        .trim()
        .parse()
        .ok()
        .context("the pid file is corrupt")
}
