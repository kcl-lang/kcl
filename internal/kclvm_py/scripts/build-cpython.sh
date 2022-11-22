#!/usr/bin/env bash

# Stop on error.
set -e

prepare_dirs () {
    cpython_build_dir="$topdir/_build/build/$os/cpython"
    mkdir -p "$cpython_build_dir"
    cpython_install_dir="$topdir/_build/dist/$os/cpython"
    mkdir -p "$cpython_install_dir"
}

# Print the summary.
echo "================ Summary ================"
echo "  CPython is ignored!!!"
