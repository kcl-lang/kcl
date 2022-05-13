#!/usr/bin/env bash

os=$os topdir=$topdir sslpath=$sslpath $topdir/internal/kclvm_py/scripts/build-cpython.sh
os=$os topdir=$topdir $topdir/internal/kclvm_py/scripts/build-kclvm.sh
