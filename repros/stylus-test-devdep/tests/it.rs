// Ensure dev-dependencies are compiled and features unify.
// Use both openzeppelin-stylus and stylus-sdk in the test.

use stylus_test_devdep_repro::touch;

#[test]
fn compiles_and_links() {
    // Call a function from the lib so the crate builds/links.
    touch();

    // Touch stylus-sdk so dev-dependencies are used in this test unit.
    // This ensures stylus-sdk with `stylus-test` is part of the build graph.
    let _ = core::mem::size_of::<stylus_sdk::host::WasmVM>();
}
