#!/usr/bin/env bash

# Stop on error.
set -e

root="$topdir"
kclvm_install_dir="$topdir/_build/dist/$os/kclvm"

# or English Env
export LANGUAGE=en_US.utf-8

# Install flake8 lint tools and run linting.
$kclvm_install_dir/bin/kclvm -m pip install flake8==4.0.0
$kclvm_install_dir/bin/kclvm -m flake8 --config ./.flake8 ./internal/kclvm_py

# Print the summary.
echo "================ Lint Summary ================"
echo "  Lint done successfully in $root"
