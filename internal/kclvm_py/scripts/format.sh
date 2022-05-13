#!/usr/bin/env bash

# Stop on error.
set -e

root="$topdir"
kclvm_src="$topdir/internal/kclvm_py"

# 使用中文环境
#export LANGUAGE=zh_CN.utf-8

# or English Env
export LANGUAGE=en_US.utf-8

# Install black format tools
python3 -m pip install black==21.5b1 

# Run the black format
python3 -m black $kclvm_src --extend-exclude .*?_pb2.py\|lark_token.py

# Print the summary.
echo "================ Format Summary ================"
echo "black format done successfully in $root"
