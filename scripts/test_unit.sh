#!/usr/bin/env bash

topdir=$(realpath $(dirname $0)/../)
kclvm_install_dir="$topdir/_build/dist/$os/kclvm"
kclvm_source_dir="$topdir"
export PATH=$kclvm_install_dir/bin:$PATH

# Unit test
cd $kclvm_source_dir/test/test_units/
kclvm -m nose
