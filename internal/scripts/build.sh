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
    mkdir -p "$install_dir/lib/site-packages"
    mkdir -p "$install_dir/include"
}

prepare_dirs

# Perform the build process.
set -x

# Copy kcl scripts
cp "$topdir/internal/scripts/cli/kcl" $install_dir/bin/
cp "$topdir/internal/scripts/cli/kclvm" $install_dir/bin/
cp "$topdir/internal/scripts/cli/kcl-plugin" $install_dir/bin/
cp "$topdir/internal/scripts/cli/kcl-doc" $install_dir/bin/
cp "$topdir/internal/scripts/cli/kcl-test" $install_dir/bin/
cp "$topdir/internal/scripts/cli/kcl-lint" $install_dir/bin/
cp "$topdir/internal/scripts/cli/kcl-fmt" $install_dir/bin/
cp "$topdir/internal/scripts/cli/kcl-vet" $install_dir/bin/
chmod +x $install_dir/bin/kcl
chmod +x $install_dir/bin/kclvm
chmod +x $install_dir/bin/kcl-plugin
chmod +x $install_dir/bin/kcl-doc
chmod +x $install_dir/bin/kcl-test
chmod +x $install_dir/bin/kcl-lint
chmod +x $install_dir/bin/kcl-fmt
chmod +x $install_dir/bin/kcl-vet

if [ -d $install_dir/lib/site-packages/kclvm ]; then
   rm -rf $install_dir/lib/site-packages/kclvm
fi

# Install plugins
cp -rf $topdir/plugins $install_dir/

set +x

# build kcl

cd $topdir/kclvm
cargo build --release

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

# Copy libkclvm_cli lib

if [ -e $topdir/kclvm/target/release/libkclvm_cli_cdylib.$dll_extension ]; then
    touch $install_dir/bin/libkclvm_cli_cdylib.$dll_extension
    rm $install_dir/bin/libkclvm_cli_cdylib.$dll_extension
    cp $topdir/kclvm/target/release/libkclvm_cli_cdylib.$dll_extension $install_dir/bin/libkclvm_cli_cdylib.$dll_extension
fi

# build kcl LSP server

cd $topdir/kclvm/tools/src/LSP
cargo build --release

touch $install_dir/bin/kcl-language-server
rm $install_dir/bin/kcl-language-server
cp $topdir/kclvm/target/release/kcl-language-server $install_dir/bin/kcl-language-server


cd $topdir/kclvm_cli
cargo build --release

touch $install_dir/bin/kclvm_cli
rm $install_dir/bin/kclvm_cli
cp ./target/release/kclvm_cli $install_dir/bin/kclvm_cli


# Copy kcl C API header
cd $topdir/kclvm/runtime
cp src/_kclvm.h  $install_dir/include/_kclvm.h

# build kcl plugin python module
cd $topdir/kclvm/plugin
cp ./kclvm_plugin.py $install_dir/lib/site-packages/
cp ./kclvm_runtime.py $install_dir/lib/site-packages/

cd $topdir
# Print the summary.
echo "================ Summary ================"
echo "  KCLVM is updated into $install_dir"
