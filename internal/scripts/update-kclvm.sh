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
    kclvm_install_dir="$topdir/_build/dist/$os/kclvm"
    mkdir -p "$kclvm_install_dir/bin"
    mkdir -p "$kclvm_install_dir/lib/site-packages"
    mkdir -p "$kclvm_install_dir/include"
}

prepare_dirs
kclvm_source_dir="$topdir/internal"

# Perform the build process.
set -x

# Copy KCLVM.
cp "$topdir/internal/scripts/cli/kcl" $kclvm_install_dir/bin/
cp "$topdir/internal/scripts/cli/kclvm" $kclvm_install_dir/bin/
cp "$topdir/internal/scripts/cli/kcl-plugin" $kclvm_install_dir/bin/
cp "$topdir/internal/scripts/cli/kcl-doc" $kclvm_install_dir/bin/
cp "$topdir/internal/scripts/cli/kcl-test" $kclvm_install_dir/bin/
cp "$topdir/internal/scripts/cli/kcl-lint" $kclvm_install_dir/bin/
cp "$topdir/internal/scripts/cli/kcl-fmt" $kclvm_install_dir/bin/
cp "$topdir/internal/scripts/cli/kcl-vet" $kclvm_install_dir/bin/
chmod +x $kclvm_install_dir/bin/kcl
chmod +x $kclvm_install_dir/bin/kclvm
chmod +x $kclvm_install_dir/bin/kcl-plugin
chmod +x $kclvm_install_dir/bin/kcl-doc
chmod +x $kclvm_install_dir/bin/kcl-test
chmod +x $kclvm_install_dir/bin/kcl-lint
chmod +x $kclvm_install_dir/bin/kcl-fmt
chmod +x $kclvm_install_dir/bin/kcl-vet

if [ -d $kclvm_install_dir/lib/site-packages/kclvm ]; then
   rm -rf $kclvm_install_dir/lib/site-packages/kclvm
fi

set +x

# build kclvm-cli

cd $topdir/kclvm
cargo build --release

touch $kclvm_install_dir/bin/kclvm_cli
rm $kclvm_install_dir/bin/kclvm_cli
cp ./target/release/kclvm_cli $kclvm_install_dir/bin/kclvm_cli

# Switch dll file extension according to os.
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

# libkclvm_cli

# Darwin dylib
if [ -e $topdir/kclvm/target/release/libkclvm_cli_cdylib.$dll_extension ]; then
    touch $kclvm_install_dir/bin/libkclvm_cli_cdylib.$dll_extension
    rm $kclvm_install_dir/bin/libkclvm_cli_cdylib.$dll_extension
    cp $topdir/kclvm/target/release/libkclvm_cli_cdylib.$dll_extension $kclvm_install_dir/bin/libkclvm_cli_cdylib.$dll_extension
fi

# build rust std lib

RUST_SYS_ROOT=`rustc --print sysroot`

# libstd-*.dylib or libstd-*.so
cd $RUST_SYS_ROOT/lib
RUST_LIBSTD=`find libstd-*.*`

mkdir -p $kclvm_install_dir/lib
cp "$RUST_SYS_ROOT/lib/$RUST_LIBSTD" $kclvm_install_dir/lib/$RUST_LIBSTD
echo "$RUST_LIBSTD" > $kclvm_install_dir/lib/rust-libstd-name.txt

# Build kclvm runtime

cd $topdir/kclvm/runtime
## Native
cargo build --release

# Darwin dylib
# Linux so
# Windows dll
if [ -e $topdir/kclvm/target/release/libkclvm.$dll_extension ]; then
    touch $kclvm_install_dir/lib/libkclvm.$dll_extension
    rm $kclvm_install_dir/lib/libkclvm.$dll_extension
    cp $topdir/kclvm/target/release/libkclvm.$dll_extension $kclvm_install_dir/lib/
    cp $topdir/kclvm/target/release/libkclvm.$dll_extension $kclvm_install_dir/lib/libkclvm.$dll_extension
fi

# Copy LLVM runtime and header
cd $topdir/kclvm/runtime
cp src/_kclvm.bc $kclvm_install_dir/include/_kclvm.bc
cp src/_kclvm.h  $kclvm_install_dir/include/_kclvm.h

cd $kclvm_install_dir/include

# build kclvm_plugin python module

cd $topdir/kclvm/plugin
python3 setup.py install_lib --install-dir=$kclvm_install_dir/lib/site-packages

# Print the summary.
echo "================ Summary ================"
echo "  KCLVM is updated into $kclvm_install_dir"
