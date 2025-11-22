fn main() {
    setup_target();
}

/// Set rustc TARGET to KCL_DEFAULT_TARGET
fn setup_target() {
    println!(
        "cargo:rustc-env=KCL_DEFAULT_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
}
