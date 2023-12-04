fn main() {
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-search=..\\kclvm\\target\\release");
    } else {
        println!("cargo:rustc-link-search=../kclvm/target/release");
    }
    println!("cargo:rustc-link-lib=dylib=kclvm_cli_cdylib");
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    }
}
