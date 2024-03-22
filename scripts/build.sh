#!/usr/bin/env bash

# Stop on error.
set -e

# Environment

if [ -f "/etc/os-release" ]; then
    source /etc/os-release
    os=$ID
else
    os=$(uname)
fi

prepare_dirs () {
    install_dir="$topdir/_build/dist/$os/kclvm"
    mkdir -p "$install_dir/bin"
    mkdir -p "$install_dir/include"
}

prepare_dirs

# 1. Build kcl native library

cd $topdir/kclvm
export PATH=$PATH:/root/.cargo/bin:/usr/lib/llvm-12/bin
cargo build --release

## Switch dll file extension according to os.
dll_extension="so"
case $os in
    "Default" | "default" | "centos" | "ubuntu" | "debian" | "Ubuntu" |"Debian" | "Static-Debian" | "Cood1-Debian" | "Cood1Shared-Debian")
        dll_extension="so"
        ;;
    "Darwin" | "darwin" | "ios" | "macos")
        dll_extension="dylib"
        ;;
    *) dll_extension="dll"
        ;;
esac

## Copy C API header

cd $topdir/kclvm/runtime
cp src/_kclvm.h  $install_dir/include/kclvm.h

## Copy libkclvm_cli lib to the build folder

if [ -e $topdir/kclvm/target/release/libkclvm_cli_cdylib.$dll_extension ]; then
    touch $install_dir/bin/libkclvm_cli_cdylib.$dll_extension
    rm $install_dir/bin/libkclvm_cli_cdylib.$dll_extension
    cp $topdir/kclvm/target/release/libkclvm_cli_cdylib.$dll_extension $install_dir/bin/libkclvm_cli_cdylib.$dll_extension
fi

## 2. Build KCL language server binary

cd $topdir/kclvm/tools/src/LSP
cargo build --release

touch $install_dir/bin/kcl-language-server
rm $install_dir/bin/kcl-language-server
cp $topdir/kclvm/target/release/kcl-language-server $install_dir/bin/kcl-language-server

## 3. Build CLI

cd $topdir/cli
cargo build --release

touch $install_dir/bin/kclvm_cli
rm $install_dir/bin/kclvm_cli
cp ./target/release/kclvm_cli $install_dir/bin/kclvm_cli

cd $topdir

# Print the summary.
echo "================ Summary ================"
echo "  KCLVM is updated into $install_dir"
