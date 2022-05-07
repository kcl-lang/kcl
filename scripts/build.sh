#!/usr/bin/env bash

os=$os topdir=$topdir sslpath=$sslpath $topdir/scripts/build-cpython.sh
os=$os topdir=$topdir $topdir/scripts/build-kclvm.sh
