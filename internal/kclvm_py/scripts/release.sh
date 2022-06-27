#!/usr/bin/env bash

kclvm_release_file="kclvm-$os-latest.tar.gz"
kclvm_release_path="$topdir/_build"
kclvm_package_dir="$topdir/_build/dist/$os"
kclvm_install_dir="kclvm"

rm $kclvm_release_file

cd $kclvm_package_dir
if [[ x"$os" == "xDarwin" ]]; then
    xattr -rc $kclvm_install_dir
fi
tar -czvf $kclvm_release_file $kclvm_install_dir

mv $kclvm_package_dir/$kclvm_release_file $kclvm_release_path/$kclvm_release_file

# Print the summary.
echo "================ Summary ================"
echo "  $kclvm_release_path/$kclvm_release_file has been created"
