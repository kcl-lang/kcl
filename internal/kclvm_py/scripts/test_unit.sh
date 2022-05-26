#!/usr/bin/env bash

topdir=$(realpath $(dirname $0)/../../../)
kclvm_install_dir="$topdir/_build/dist/$os/kclvm"
kclvm_source_dir="$topdir"

export PYTHONPATH=$kclvm_install_dir/lib/site-packages

echo PATH=$PATH:$kclvm_source_dir/_build/dist/ubuntu/kclvm/bin >> ~/.bash_profile
source ~/.bash_profile

# Install the dependency
python3 -m pip install --target=$kclvm_install_dir/lib/site-packages -r $kclvm_install_dir/requirements.txt
python3 -m pip install --target=$kclvm_install_dir/lib/site-packages nose==1.3.7

# Unit test
cd $kclvm_source_dir/test/test_units/
python3 -m nose
