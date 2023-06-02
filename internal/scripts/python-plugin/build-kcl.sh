#!/usr/bin/env bash

# Stop on error.
set -e

topdir=$PWD
# Environment
if [ -f "/etc/os-release" ]; then
    source /etc/os-release
    os=$ID
else
    os=$(uname)
fi

prepare_dirs () {
    cpython_build_dir="$topdir/_build/python_dist/$os/cpython"
    kclvm_packages_dir="$topdir/_build/packages"
    kcl_install_dir="$topdir/_build/python_dist/$os/kclvm"
    mkdir -p "$kcl_install_dir"
    mkdir -p "$kclvm_packages_dir"
}

prepare_dirs

# python exe name
py_exe_name="python3.7"
if [ -d "${cpython_build_dir}/lib/python3.9" ]; then
    py_exe_name="python3.9"
fi

# py_lib_basename: python3.x
py_lib_basename="python3.7"
if [ -d "${cpython_build_dir}/lib/python3.9" ]; then
    py_lib_basename="python3.9"
fi

# Perform the build process.
set -x

# Copy files from CPython.
cd $kcl_install_dir
mkdir -p bin
mkdir -p lib
cp $cpython_build_dir/bin/${py_exe_name} $kcl_install_dir/bin/kclvm
cp -r $cpython_build_dir/lib/${py_lib_basename} $kcl_install_dir/lib/

# Darwin dylib
if [ -e $cpython_build_dir/lib/lib${py_lib_basename}.dylib ]; then
    touch $kcl_install_dir/lib/lib${py_lib_basename}.dylib
    rm $kcl_install_dir/lib/lib${py_lib_basename}.dylib
    mv $cpython_build_dir/lib/lib${py_lib_basename}.dylib $kcl_install_dir/lib/lib${py_lib_basename}.dylib
fi
# Linux so
if [ -e $cpython_build_dir/lib/lib${py_lib_basename}m.so.1.0 ]; then
    touch $kcl_install_dir/lib/lib${py_lib_basename}.so
    rm $kcl_install_dir/lib/lib${py_lib_basename}.so
    mv $cpython_build_dir/lib/lib${py_lib_basename}m.so.1.0 $kcl_install_dir/lib/lib${py_lib_basename}.so
fi
# Windows dll
if [ -e $cpython_build_dir/lib/lib${py_lib_basename}.dll ]; then
    touch $kcl_install_dir/lib/lib${py_lib_basename}.dll
    rm $kcl_install_dir/lib/lib${py_lib_basename}.dll
    mv $cpython_build_dir/lib/lib${py_lib_basename}.dll $kcl_install_dir/lib/lib${py_lib_basename}.dll
fi
cp -r $cpython_build_dir/include $kcl_install_dir/

# Copy KCL Scripts.
scripts_dir="$topdir/internal/scripts/python-plugin/cli"
cp "$scripts_dir/kcl" $kcl_install_dir/bin/
cp "$scripts_dir/kcl-plugin" $kcl_install_dir/bin/
cp "$scripts_dir/kcl-doc" $kcl_install_dir/bin/
cp "$scripts_dir/kcl-test" $kcl_install_dir/bin/
cp "$scripts_dir/kcl-lint" $kcl_install_dir/bin/
cp "$scripts_dir/kcl-fmt" $kcl_install_dir/bin/
cp "$scripts_dir/kcl-vet" $kcl_install_dir/bin/
chmod +x $kcl_install_dir/bin/kcl
chmod +x $kcl_install_dir/bin/kcl-plugin
chmod +x $kcl_install_dir/bin/kcl-doc
chmod +x $kcl_install_dir/bin/kcl-test
chmod +x $kcl_install_dir/bin/kcl-lint
chmod +x $kcl_install_dir/bin/kcl-fmt
chmod +x $kcl_install_dir/bin/kcl-vet

if [ -d $kcl_install_dir/lib/${py_lib_basename}/kclvm ]; then
   rm -rf $kcl_install_dir/lib/${py_lib_basename}/kclvm
fi

# Get site-packages.
$kcl_install_dir/bin/kclvm -m pip install --upgrade -U kclvm

# Install plugins
cp -rf $topdir/plugins $kcl_install_dir/

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
    touch $kcl_install_dir/bin/libkclvm_cli_cdylib.$dll_extension
    rm $kcl_install_dir/bin/libkclvm_cli_cdylib.$dll_extension
    cp $topdir/kclvm/target/release/libkclvm_cli_cdylib.$dll_extension $kcl_install_dir/bin/libkclvm_cli_cdylib.$dll_extension
fi

# build kcl LSP server

cd $topdir/kclvm/tools/src/LSP
cargo build --release

touch $kcl_install_dir/bin/kcl-language-server
rm $kcl_install_dir/bin/kcl-language-server
cp $topdir/kclvm/target/release/kcl-language-server $kcl_install_dir/bin/kcl-language-server


cd $topdir/kclvm_cli
cargo build --release

touch $kcl_install_dir/bin/kclvm_cli
rm $kcl_install_dir/bin/kclvm_cli
cp ./target/release/kclvm_cli $kcl_install_dir/bin/kclvm_cli


# Copy kcl C API header
cd $topdir/kclvm/runtime
cp src/_kclvm.h  $kcl_install_dir/include/_kclvm.h

# build kcl plugin python module
cd $topdir/kclvm/plugin
cp ./kclvm_plugin.py $kcl_install_dir/lib/site-packages/
cp ./kclvm_runtime.py $kcl_install_dir/lib/site-packages/

cd $topdir
# Print the summary.
echo "================ Summary ================"
echo "  KCLVM is updated into $kcl_install_dir"
