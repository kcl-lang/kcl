fn main() {
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-search=target\\release");
    } else {
        println!("cargo:rustc-link-search=target/release");
    }
    println!("cargo:rustc-link-lib=dylib=kcl");
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    }
}
