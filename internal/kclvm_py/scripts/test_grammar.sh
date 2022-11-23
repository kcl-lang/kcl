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

# Grammar test
cd $kclvm_source_dir/test/grammar
python3 -m pip install --upgrade pip
python3 -m pip install -r $topdir/internal/kclvm_py/scripts/requirements.txt
python3 -m pip install pytest pytest-xdist
python3 -m pytest -v -n 10
