#!/usr/bin/env bash

latesttag=$(git describe --tags)

kclvm_release_file="kclvm-$latesttag-$os.tar.gz"
kclvm_release_path="$topdir/_build"
kclvm_package_dir="$topdir/_build/dist/$os"
kclvm_install_dir="kclvm"

if [ x"$os" == x"Darwin" ]; then
    if [[ $(uname -m) == 'arm64' ]]; then
        kclvm_release_file="kclvm-$latesttag-$os-arm64.tar.gz"
    fi
fi

rm $kclvm_release_file

if [ -d $kclvm_package_dir/kclvm/lib/site-packages ]; then
   rm -rf $kclvm_package_dir/kclvm/lib/site-packages
fi

mkdir -p $kclvm_package_dir/kclvm/lib/site-packages
cp -r $kclvm_source_dir/kclvm_py $kclvm_package_dir/kclvm/lib/site-packages
mv $kclvm_package_dir/kclvm/lib/site-packages/kclvm_py $kclvm_package_dir/kclvm/lib/site-packages/kclvm

cd $kclvm_package_dir
tar -czvf $kclvm_release_file $kclvm_install_dir

mv $kclvm_package_dir/$kclvm_release_file $kclvm_release_path/$kclvm_release_file

# Print the summary.
echo "================ Summary ================"
echo "  $kclvm_release_path/$kclvm_release_file has been created"
