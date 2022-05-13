#!/usr/bin/env bash

topdir=$(realpath $(dirname $0)/../../../)
kclvm_install_dir="$topdir/_build/dist/$os/kclvm"
kclvm_source_dir="$topdir"

echo PATH=$PATH:$kclvm_source_dir/_build/dist/ubuntu/kclvm/bin >> ~/.bash_profile
source ~/.bash_profile

# Grammar test
cd $kclvm_source_dir/test/grammar
kclvm -m pytest -v -n 10
