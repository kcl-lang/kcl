#!/usr/bin/env bash

# Stop on error.
set -e

# Environment

getSystemInfo() {
    arch=$(uname -m)
    case $arch in
        armv7*) arch="arm";;
        aarch64) arch="arm64";;
        x86_64) arch="amd64";;
    esac

    os=$(echo `uname`|tr '[:upper:]' '[:lower:]')
}

getSystemInfo

prepare_dirs () {
    install_dir="_build/dist/$os/core"
    mkdir -p "$install_dir"
}

prepare_dirs

# 1. Build kcl native library
cargo build --release

## Switch dll file extension according to os.
dll_extension="so"
case $os in
    "Linux" | "linux" | "Default" | "default" | "centos" | "ubuntu" | "debian" | "Ubuntu" | "Debian" | "Static-Debian" | "Cood1-Debian" | "Cood1Shared-Debian")
        dll_extension="so"
        ;;
    "Darwin" | "darwin" | "ios" | "macos")
        dll_extension="dylib"
        ;;
    *) dll_extension="dll"
        ;;
esac

## Copy kcl lib to the build folder
if [ -e target/release/libkcl.$dll_extension ]; then
    touch $install_dir/libkcl.$dll_extension
    rm $install_dir/libkcl.$dll_extension
    cp target/release/libkcl.$dll_extension $install_dir/libkcl.$dll_extension
fi

## 2. Build KCL language server binary
cargo build --release --manifest-path crates/tools/src/LSP/Cargo.toml

touch $install_dir/kcl-language-server
rm $install_dir/kcl-language-server
cp target/release/kcl-language-server $install_dir/kcl-language-server

## 3. Build CLI
cargo build --release --manifest-path crates/cli/Cargo.toml

touch $install_dir/libkcl
rm $install_dir/libkcl
cp ./target/release/libkcl $install_dir/libkcl

# Print the summary.
echo "================ Summary ================"
echo "  KCL is updated into $install_dir"
