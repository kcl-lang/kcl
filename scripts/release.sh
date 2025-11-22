#!/usr/bin/env bash

getSystemInfo() {
    arch=$(uname -m)
    case $arch in
        armv7*) arch="arm";;
        aarch64) arch="arm64";;
        x86_64) arch="amd64";;
    esac

    os=$(echo `uname`|tr '[:upper:]' '[:lower:]')
}

if [ -z "$version" ]; then
    version=$1
fi
if [ -z "$version" ]; then
    version='latest'
fi

getSystemInfo

echo "[info] os: $os"
echo "[info] arch: $arch"
echo "[info] version: $version"
release_file="kcl-core-$version-$os-$arch.tar.gz"
release_path="_build"
package_dir="_build/dist/$os"
install_dir="core"

cd $package_dir
tar -czvf $release_file $install_dir

# Print the summary.
echo "================ Summary ================"
echo "  $release_path/$release_file has been created"
