use std::{
    env,
    ffi::OsStr,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use eyre::bail;
use toml::Table;

/// Returns the path to the compiled wasm binary with name `name`.
///
/// Note that this function works for both workspaces and standalone crates.
///
/// # Errors
///
/// May error if:
///
/// - Unable to read the current executable's path.
/// - The output directory is not `target`.
pub(crate) fn get_wasm_path() -> eyre::Result<PathBuf> {
    let name = read_pkg_name()?;

    let name = name.replace('-', "_");
    // Looks like
    // "rust-contracts-stylus/target/debug/deps/erc721-15764c2c9a33bee7".
    let mut target_dir = env::current_exe()?;

    // Recursively find a `target` directory.
    loop {
        let Some(parent) = target_dir.parent() else {
            // We've found `/`.
            bail!("output directory is not 'target'");
        };

        target_dir = parent.to_path_buf();
        let Some(leaf) = target_dir.file_name() else {
            // We've found the root because we are traversing a canonicalized
            // path, which means there are no `..` segments, and we started at
            // the executable.
            bail!("output directory is not 'target'");
        };

        if leaf == OsStr::new("target") {
            break;
        }
    }

    let wasm = target_dir
        .join("wasm32-unknown-unknown")
        .join("release")
        .join(format!("{name}.wasm"));

    Ok(wasm)
}

/// Reads and parses the package name from a manifest in `path`.
///
/// # Errors
///
/// May error if:
///
/// - Unable to parse the `Cargo.toml` at `path`.
/// - Unable to read the package name from the parsed toml file.
fn read_pkg_name() -> eyre::Result<String> {
    let cargo_toml = env::current_dir()?.join("Cargo.toml");

    let mut reader = BufReader::new(File::open(cargo_toml)?);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    let table = buffer.parse::<Table>()?;
    let name = table["package"]["name"].as_str();

    match name {
        Some(x) => Ok(x.to_owned()),
        None => Err(eyre::eyre!("unable to find package name in toml")),
    }
}
