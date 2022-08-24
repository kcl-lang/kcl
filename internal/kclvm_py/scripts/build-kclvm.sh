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
kclvm_source_dir="$topdir/internal/kclvm_py"

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
cd $kclvm_install_dir
mkdir -p bin
mkdir -p lib
cp $cpython_build_dir/bin/${py_exe_name} $kclvm_install_dir/bin/kclvm
cp -r $cpython_build_dir/lib/${py_lib_basename} $kclvm_install_dir/lib/

# Darwin dylib
if [ -e $cpython_build_dir/lib/lib${py_lib_basename}.dylib ]; then
    touch $kclvm_install_dir/lib/lib${py_lib_basename}.dylib
    rm $kclvm_install_dir/lib/lib${py_lib_basename}.dylib
    mv $cpython_build_dir/lib/lib${py_lib_basename}.dylib $kclvm_install_dir/lib/lib${py_lib_basename}.dylib
fi
# Linux so
if [ -e $cpython_build_dir/lib/lib${py_lib_basename}m.so.1.0 ]; then
    touch $kclvm_install_dir/lib/lib${py_lib_basename}.so
    rm $kclvm_install_dir/lib/lib${py_lib_basename}.so
    mv $cpython_build_dir/lib/lib${py_lib_basename}m.so.1.0 $kclvm_install_dir/lib/lib${py_lib_basename}.so
fi
# Windows dll
if [ -e $cpython_build_dir/lib/lib${py_lib_basename}.dll ]; then
    touch $kclvm_install_dir/lib/lib${py_lib_basename}.dll
    rm $kclvm_install_dir/lib/lib${py_lib_basename}.dll
    mv $cpython_build_dir/lib/lib${py_lib_basename}.dll $kclvm_install_dir/lib/lib${py_lib_basename}.dll
fi
cp -r $cpython_build_dir/include $kclvm_install_dir/

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

if [ -d $kclvm_install_dir/lib/${py_lib_basename}/kclvm ]; then
   rm -rf $kclvm_install_dir/lib/${py_lib_basename}/kclvm
fi
cp -r $kclvm_source_dir $kclvm_install_dir/lib/${py_lib_basename}
mv $kclvm_install_dir/lib/${py_lib_basename}/kclvm_py $kclvm_install_dir/lib/${py_lib_basename}/kclvm

# Get site-packages.
chmod +x $topdir/internal/kclvm_py/scripts/kcllib-install.sh
$topdir/internal/kclvm_py/scripts/kcllib-install.sh

# Install plugins
cp -rf $topdir/plugins $kclvm_install_dir/

set +x

# Print the summary.
echo "================ Summary ================"
echo "  KCLVM is installed into $kclvm_install_dir"
