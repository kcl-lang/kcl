#!/usr/bin/env bash

topdir=$PWD
# Environment
if [ -f "/etc/os-release" ]; then
    source /etc/os-release
    os=$ID
else
    os=$(uname)
fi

release_file="kclvm-$os-latest.tar.gz"
release_path="$topdir/_build"
package_dir="$topdir/_build/python_dist/$os"
install_dir="kclvm"

cd $package_dir
tar -czvf $release_file $install_dir

mv $package_dir/$release_file $release_path/$release_file

# Print the summary.
echo "================ Summary ================"
echo "  $release_path/$release_file has been created"
