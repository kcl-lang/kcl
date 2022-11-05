//! Using `cc` crate to package LLVM `lld` static libraries into
//! the KCLVM CLI.
//!
//! Ref: https://github.com/hyperledger/solang/blob/main/build.rs

fn main() {
    stack_link_lld();
}

/// Using `cc` crate to package `lld` static libraries into the KCLVM CLI.
fn stack_link_lld() {
    use std::process::Command;

    let cxxflags = Command::new("llvm-config")
        .args(&["--cxxflags"])
        .output()
        .expect("could not execute llvm-config");

    let cxxflags = String::from_utf8(cxxflags.stdout).unwrap();

    let mut build = cc::Build::new();

    build.file("src/linker.cpp").cpp(true);

    if !cfg!(target_os = "windows") {
        build.flag("-Wno-unused-parameter");
    }

    for flag in cxxflags.split_whitespace() {
        build.flag(flag);
    }

    build.compile("liblinker.a");

    let libdir = Command::new("llvm-config")
        .args(&["--libdir"])
        .output()
        .unwrap();
    let libdir = String::from_utf8(libdir.stdout).unwrap();

    println!("cargo:libdir={}", libdir);
    for lib in &[
        "lldMachO",
        "lldELF",
        "lldMinGW",
        "lldCOFF",
        "lldDriver",
        "lldCore",
        "lldCommon",
        "lldWasm",
    ] {
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    // Add all the symbols were not using, needed by Windows and debug builds
    for lib in &["lldReaderWriter", "lldYAML"] {
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    let output = Command::new("git")
        .args(&["describe", "--tags"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // Make sure we have an 8MiB stack on Windows. Windows defaults to a 1MB
    // stack, which is not big enough for debug builds
    #[cfg(windows)]
    println!("cargo:rustc-link-arg=/STACK:8388608");
}
