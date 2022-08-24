#!/usr/bin/env bash

# Stop on error.
set -e

prepare_dirs () {
    cpython_build_dir="$topdir/_build/dist/$os/cpython"
    kclvm_packages_dir="$topdir/_build/packages"
    kclvm_install_dir="$topdir/_build/dist/$os/kclvm"
    mkdir -p "$kclvm_install_dir"
    mkdir -p "$kclvm_packages_dir"
}

prepare_dirs
kclvm_source_dir="$topdir/internal"

# Perform the build process.
set -x

# Copy KCLVM.
cp "$topdir/internal/kclvm_py/scripts/cli/kcl" $kclvm_install_dir/bin/
cp "$topdir/internal/kclvm_py/scripts/cli/kcl-plugin" $kclvm_install_dir/bin/
cp "$topdir/internal/kclvm_py/scripts/cli/kcl-doc" $kclvm_install_dir/bin/
cp "$topdir/internal/kclvm_py/scripts/cli/kcl-test" $kclvm_install_dir/bin/
cp "$topdir/internal/kclvm_py/scripts/cli/kcl-lint" $kclvm_install_dir/bin/
cp "$topdir/internal/kclvm_py/scripts/cli/kcl-fmt" $kclvm_install_dir/bin/
cp "$topdir/internal/kclvm_py/scripts/cli/kcl-vet" $kclvm_install_dir/bin/
chmod +x $kclvm_install_dir/bin/kcl
chmod +x $kclvm_install_dir/bin/kcl-plugin
chmod +x $kclvm_install_dir/bin/kcl-doc
chmod +x $kclvm_install_dir/bin/kcl-test
chmod +x $kclvm_install_dir/bin/kcl-lint
chmod +x $kclvm_install_dir/bin/kcl-fmt
chmod +x $kclvm_install_dir/bin/kcl-vet

kclvm_lib_dir=$kclvm_install_dir/lib/python3.7/
if [ -d $kclvm_install_dir/lib/python3.9/ ]; then
    kclvm_lib_dir=$kclvm_install_dir/lib/python3.9/
fi

if [ -d $kclvm_lib_dir/kclvm ]; then
   rm -rf $kclvm_lib_dir/kclvm
fi
cp -r $kclvm_source_dir/kclvm_py $kclvm_lib_dir/kclvm

set +x

# build kclvm-cli

cd $topdir/kclvm
cargo build --release

touch $kclvm_install_dir/bin/kclvm_cli
rm $kclvm_install_dir/bin/kclvm_cli
cp ./target/release/kclvm_cli $kclvm_install_dir/bin/kclvm_cli

# libkclvm_cli

# Darwin dylib
if [ -e $topdir/kclvm/target/release/libkclvm_cli_cdylib.dylib ]; then
    touch $kclvm_install_dir/bin/libkclvm_cli_cdylib.dylib
    rm $kclvm_install_dir/bin/libkclvm_cli_cdylib.dylib
    cp $topdir/kclvm/target/release/libkclvm_cli_cdylib.dylib $kclvm_install_dir/bin/libkclvm_cli_cdylib.dylib
fi
# Linux so
if [ -e $topdir/kclvm/target/release/libkclvm_cli_cdylib.so ]; then
    touch $kclvm_install_dir/bin/libkclvm_cli_cdylib.so
    rm $kclvm_install_dir/bin/libkclvm_cli_cdylib.so
    cp $topdir/kclvm/target/release/libkclvm_cli_cdylib.so $kclvm_install_dir/bin/libkclvm_cli_cdylib.so
fi
# Windows dll
if [ -e $topdir/kclvm/target/release/libkclvm_cli_cdylib.dll ]; then
    touch $kclvm_install_dir/bin/libkclvm_cli_cdylib.dll
    rm $kclvm_install_dir/bin/libkclvm_cli_cdylib.dll
    cp $topdir/kclvm/target/release/libkclvm_cli_cdylib.dll $kclvm_install_dir/bin/libkclvm_cli_cdylib.dll
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
cp $topdir/kclvm/target/release/libkclvm.a                        $kclvm_install_dir/lib/libkclvm_native.a

# Darwin dylib
if [ -e $topdir/kclvm/target/release/libkclvm.dylib ]; then
    touch $kclvm_install_dir/lib/libkclvm.dylib
    rm $kclvm_install_dir/lib/libkclvm.dylib
    cp $topdir/kclvm/target/release/libkclvm.dylib $kclvm_install_dir/lib/
    cp $topdir/kclvm/target/release/libkclvm.dylib $kclvm_install_dir/lib/libkclvm_native_shared.dylib
fi
# Linux so
if [ -e $topdir/kclvm/target/release/libkclvm.so ]; then
    touch $kclvm_install_dir/lib/libkclvm.so
    rm $kclvm_install_dir/lib/libkclvm.so
    cp $topdir/kclvm/target/release/libkclvm.so $kclvm_install_dir/lib/
    cp $topdir/kclvm/target/release/libkclvm.so $kclvm_install_dir/lib/libkclvm_native_shared.so
fi
# Windows dll
if [ -e $topdir/kclvm/target/release/libkclvm.dll ]; then
    touch $kclvm_install_dir/lib/libkclvm.dll
    rm $kclvm_install_dir/lib/libkclvm.dll
    cp $topdir/kclvm/target/release/libkclvm.dll $kclvm_install_dir/lib/
    cp $topdir/kclvm/target/release/libkclvm.dll $kclvm_install_dir/lib/libkclvm_native_shared.dll
fi


# WASM
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
cp $topdir/kclvm/target/wasm32-unknown-unknown/release/libkclvm.a $kclvm_install_dir/lib/libkclvm_wasm32.a
cp src/_kclvm_undefined_wasm.txt $kclvm_install_dir/lib/_kclvm_undefined_wasm.txt

cd $topdir/kclvm/capi
## Native
cargo build --release

# Darwin dylib
if [ -e $topdir/kclvm/target/release/libkclvm_capi.dylib ]; then
    touch $kclvm_install_dir/lib/libkclvm_capi.dylib
    rm $kclvm_install_dir/lib/libkclvm_capi.dylib
    cp $topdir/kclvm/target/release/libkclvm_capi.dylib $kclvm_install_dir/lib/
    cp $topdir/kclvm/target/release/libkclvm_capi.dylib $kclvm_install_dir/lib/libkclvm_capi.dylib
fi
# Linux so
if [ -e $topdir/kclvm/target/release/libkclvm_capi.so ]; then
    touch $kclvm_install_dir/lib/libkclvm_capi.so
    rm $kclvm_install_dir/lib/libkclvm_capi.so
    cp $topdir/kclvm/target/release/libkclvm_capi.so $kclvm_install_dir/lib/
    cp $topdir/kclvm/target/release/libkclvm_capi.so $kclvm_install_dir/lib/libkclvm_capi.so
fi
# Windows dll
if [ -e $topdir/kclvm/target/release/libkclvm_capi.dll ]; then
    touch $kclvm_install_dir/lib/libkclvm_capi.dll
    rm $kclvm_install_dir/lib/libkclvm_capi.dll
    cp $topdir/kclvm/target/release/libkclvm_capi.dll $kclvm_install_dir/lib/
    cp $topdir/kclvm/target/release/libkclvm_capi.dll $kclvm_install_dir/lib/libkclvm_capi.dll
fi


# Copy LLVM runtime and header
cd $topdir/kclvm/runtime
cp src/_kclvm.bc $kclvm_install_dir/include/_kclvm.bc
cp src/_kclvm.h  $kclvm_install_dir/include/_kclvm.h

cd $kclvm_install_dir/include

# build kclvm_plugin python module

cd $topdir/kclvm/plugin
kclvm setup.py install_lib

# Print the summary.
echo "================ Summary ================"
echo "  KCLVM is updated into $kclvm_install_dir"
