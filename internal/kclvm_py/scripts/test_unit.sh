#!/usr/bin/env bash

# Environment
if [ -f "/etc/os-release" ]; then
    source /etc/os-release
    os=$ID
else
    os=$(uname)
fi
topdir=$(realpath $(dirname $0)/../../../)
kclvm_source_dir="$topdir"

echo PATH=$PATH:$kclvm_source_dir/_build/dist/$os/kclvm/bin >> ~/.bash_profile
source ~/.bash_profile

export PATH=$PATH:$topdir/_build/dist/$os/kclvm/bin

# Install the dependency
kclvm -m pip install nose==1.3.7

# Unit test
cd $kclvm_source_dir/test/test_units/
kclvm -m nose
