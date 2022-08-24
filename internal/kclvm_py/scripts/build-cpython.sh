#!/usr/bin/env bash

# Stop on error.
set -e

prepare_dirs () {
    cpython_build_dir="$topdir/_build/build/$os/cpython"
    mkdir -p "$cpython_build_dir"
    cpython_install_dir="$topdir/_build/dist/$os/cpython"
    mkdir -p "$cpython_install_dir"
}

# Switch configuration options.
config_option="Default"
if [ "$os" != "" ]; then
    config_option=$os
fi

# python version
py_ver_major="3"
py_ver_minor="7"
py_ver_micro="6"

for config in "$config_option"
do
    case $config in
        "Default" | "centos")
            config_envs="LANG=C.UTF-8"
            config_options="--enable-optimizations --with-ssl"
            echo "$REPLY: The configuration is $config: config_envs=$config_envs config_options=$config_options"
            break
            ;;
        "Darwin")
            if [ "$sslpath" == "" ]; then
                sslpath=$(brew --prefix openssl@1.1)
            fi

            py_ver_major="3"
            py_ver_minor="9"
            py_ver_micro="12"

            config_envs="LANG=C.UTF-8"
            config_options="--enable-optimizations --with-openssl=$sslpath --with-ssl-default-suites=python"
            echo "$REPLY: The configuration is $config: config_envs=$config_envs config_options=$config_options"
            break
            ;;
        "ubuntu" | "debian" | "Ubuntu" |"Debian" | "Static-Debian" | "Cood1-Debian" | "Cood1Shared-Debian")
            config_envs="CFLAGS=-Wno-coverage-mismatch"
            config_options="--enable-optimizations --with-ssl"
            echo "$REPLY: The configuration is $config: config_envs=$config_envs config_options=$config_options"
            break
            ;;
        *) echo "Invalid config option $REPLY:$config"
            exit 1
            break
            ;;
    esac
done

# py_ver_str="$(python3 -c 'import os; print(os.path.basename(os.path.dirname(os.__file__)))')"
py_ver_str="${py_ver_major}.${py_ver_minor}.${py_ver_micro}"

# wget python
mkdir -p $topdir/_build/3rdparty
wget -P  $topdir/_build/3rdparty "https://www.python.org/ftp/python/${py_ver_str}/Python-${py_ver_str}.tgz"
tar zxvf $topdir/_build/3rdparty/Python-${py_ver_str}.tgz -C $topdir/_build/3rdparty

prepare_dirs
prefix_option="--prefix=$cpython_install_dir"
cpython_source_dir="$topdir/_build/3rdparty/Python-${py_ver_str}"

# Perform the configuration/make/make install process.
set -x
cd $cpython_build_dir
eval $config_envs $cpython_source_dir/configure $prefix_option $config_options "--enable-shared"
eval $config_envs $cpython_source_dir/configure $prefix_option $config_options
# The make -j command may fail on some OS.
# make -j "$(nproc)"
make -j8 build_all
make -j8 altinstall
set +x

# Print the summary.
echo "================ Summary ================"
echo "  CPython is built into $cpython_build_dir"