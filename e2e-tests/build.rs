use std::path::Path;

fn main() {
    set_env("TARGET_DIR", &get_target_dir());
    set_env("RPC_URL", "http://localhost:8547");
    set_env(
        "TEST_NITRO_NODE_PATH",
        Path::new(&load_env_var("CARGO_MANIFEST_DIR"))
            .join("nitro-testnode")
            .to_str()
            .expect("set env var TEST_NITRO_NODE_PATH"),
    );
}

fn set_env(var_name: &str, value: &str) {
    println!("cargo:rustc-env={}={}", var_name, value);
}

fn load_env_var(var_name: &str) -> String {
    std::env::var(var_name)
        .unwrap_or_else(|_| panic!("failed to load {} env var", var_name))
}

fn get_target_dir() -> String {
    // should be smth like
    // ./rust-contracts-stylus/target/debug/build/e2e-tests-b008947425bb8267/out
    let out_dir = load_env_var("OUT_DIR");
    let target_dir = Path::new(&out_dir).join("../../../../");
    target_dir.to_str().expect("target dir").to_string()
}
