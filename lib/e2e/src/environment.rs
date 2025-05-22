use std::{path::PathBuf, process::Command};

use eyre::Context;

/// Gets expected path to the nitro test node.
pub(crate) fn get_node_path() -> eyre::Result<PathBuf> {
    let manifest_dir = get_workspace_root()?;
    Ok(manifest_dir.join("nitro-testnode"))
}

/// Runs the following command to get the worskpace root:
///
/// ```bash
/// git rev-parse --show-toplevel
/// ```
pub(crate) fn get_workspace_root() -> eyre::Result<PathBuf> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .wrap_err("should run `git rev-parse --show-toplevel`")?;

    let path = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string()
        .parse::<PathBuf>()
        .wrap_err("failed to parse manifest dir path")?;
    Ok(path)
}
