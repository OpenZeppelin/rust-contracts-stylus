/// Produces the value of TARGET as a string literal.
#[macro_export]
macro_rules! target {
    () => {
        "x86_64-unknown-linux-gnu"
    };
}

/// Produces the value of HOST as a string literal.
#[macro_export]
macro_rules! host {
    () => {
        "x86_64-unknown-linux-gnu"
    };
}
