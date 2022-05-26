#!/usr/bin/env bash

# Stop on error.
set -e

prepare_dirs () {
    kclvm_source_dir="$topdir/internal/kclvm_py"
    kclvm_install_dir="$topdir/_build/dist/$os/kclvm"
    mkdir -p "$kclvm_install_dir/bin"
    mkdir -p "$kclvm_install_dir/lib/site-packages"
    mkdir -p "$kclvm_install_dir/include"
}

prepare_dirs

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

if [ -d $kclvm_install_dir/lib/site-packages/kclvm ]; then
   rm -rf $kclvm_install_dir/lib/site-packages/kclvm
fi
cp -r $kclvm_source_dir $kclvm_install_dir/lib/site-packages
mv $kclvm_install_dir/lib/site-packages/kclvm_py $kclvm_install_dir/lib/site-packages/kclvm

# copy pip requirements
cp -r $kclvm_source_dir/scripts/requirements.txt $kclvm_install_dir

# Install tools/clang
# if [ x"$os" == x"Darwin" ]; then
#     base_url="https://github.com/KusionStack/llvm-package-windows/releases/download/v12.0.1"
# 
#     clang12_darwin_url="$base_url/clang12-darwin.tar.gz"
#     clang12_darwin_arm64_url="$base_url/clang12-darwin-arm64.tar.gz"
# 
#     if [ ! -f $topdir/_build/clang12-darwin.tar.gz ]; then
#         curl -LJO $clang12_darwin_url --output $topdir/_build/clang12-darwin.tar.gz
#     fi
#     if [ ! -f $topdir/_build/clang12-darwin-arm64.tar.gz ]; then
#         curl -LJO $clang12_darwin_arm64_url --output $topdir/_build/clang12-darwin-arm64.tar.gz
#     fi
# 
#     if [ -d $kclvm_install_dir/tools ]; then
#         rm -rf $kclvm_install_dir/tools
#     fi
# 
#     mkdir -p $kclvm_install_dir/tools
#     if [[ $(uname -m) == 'arm64' ]]; then
#         tar -xf $topdir/_build/clang12-darwin-arm64.tar.gz -C $kclvm_install_dir/tools
#     else
#         tar -xf $topdir/_build/clang12-darwin.tar.gz -C $kclvm_install_dir/tools
#     fi
# fi

# Install plugins
cp -rf $topdir/plugins $kclvm_install_dir/

set +x

# Print the summary.
echo "================ Summary ================"
echo "  KCLVM is installed into $kclvm_install_dir"
