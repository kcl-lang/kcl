fn main() {
    setup_target();
}

/// Set rustc TARGET to KCLVM_DEFAULT_TARGET
fn setup_target() {
    println!(
        "cargo:rustc-env=KCLVM_DEFAULT_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
}
