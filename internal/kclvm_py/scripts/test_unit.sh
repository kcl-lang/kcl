#!/usr/bin/env bash

topdir=$(realpath $(dirname $0)/../../../)
kclvm_install_dir="$topdir/_build/dist/$os/kclvm"
kclvm_source_dir="$topdir"

echo PATH=$PATH:$kclvm_source_dir/_build/dist/ubuntu/kclvm/bin >> ~/.bash_profile
source ~/.bash_profile

# Install the dependency
kclvm -m pip install nose==1.3.7

# Unit test
cd $kclvm_source_dir/test/test_units/
kclvm -m nose
