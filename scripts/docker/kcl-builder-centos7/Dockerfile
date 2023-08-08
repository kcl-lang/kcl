# Copyright 2021 The KCL Authors. All rights reserved.

FROM centos:centos7

# macOS M1 --platform linux/amd64
# try fix "Problem with the SSL CA cert (path? access rights?)"
# https://issueexplorer.com/issue/docker/for-mac/5912
# https://access.redhat.com/articles/2050743
RUN touch /etc/sysconfig/64bit_strstr_via_64bit_strstr_sse2_unaligned

# ---------------------------------------------------------------------------------
# Please note: The following steps are to install the dependency packages 
# needed to compile CPython for centos7, see the 
# [Python official website](https://devguide.python.org/setup/#install-dependencies) 
# for details. When the version of CPython used becomes higher, 
# please pay attention to update the installation dependencies. 
# ---------------------------------------------------------------------------------

# Some language environments and plug-ins related to development and compilation, 
# such as git, CPython compilation, etc.
RUN yum groupinstall -y "Development Tools"
# Compiler and tool chain required to compile CPython such as gcc, make, sqlite3, ctype, struct, etc.
RUN yum install -y gcc patch libffi-devel python-devel zlib-devel bzip2-devel ncurses-devel sqlite-devel 
RUN yum install -y libpcap-devel xz-devel readline-devel tk-devel gdbm-devel db4-deve
# Install the system libraries required by python3 for UNIX based systems
RUN yum -y install yum-utils
RUN yum-builddep -y python3
# The python zlib module dependency package is required when compiling the python source code, 
# in order to use the modules that require zlib, such as setuptools, etc.
RUN yum install -y zlib* 
# The python ssl module dependency package is required when compiling the python source code, 
# in order to use the modules that require ssl, such as pip3, twine, etc.
RUN yum install -y openssl-devel

# Install which
RUN yum install -y which

# Install wget
RUN yum install -y wget

# Install git-2.x
# RUN yum -y install https://packages.endpoint.com/rhel/7/os/x86_64/endpoint-repo-1.7-1.x86_64.rpm
# RUN yum -y install git

# rust
# https://www.rust-lang.org/tools/install
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN cargo version
RUN rustc --version

# wasm
RUN rustup target add wasm32-unknown-unknown

# Install clang7 and llvm7
# https://www.softwarecollections.org/en/scls/rhscl/llvm-toolset-7.0/
#
# 1. Install a package with repository for your system:
# On CentOS, install package centos-release-scl available in CentOS repository:
#   $ sudo yum install centos-release-scl
# 
# On RHEL, enable RHSCL repository for you system:
#   $ sudo yum-config-manager --enable rhel-server-rhscl-7-rpms
#
# 2. Install the collection:
#   $ sudo yum install llvm-toolset-7.0
#
# 3. Start using software collections:
#   $ scl enable llvm-toolset-7.0 bash

RUN yum -y install centos-release-scl
RUN yum-config-manager --enable rhel-server-rhscl-7-rpms
RUN yum -y install llvm-toolset-7.0
RUN yum -y install llvm-toolset-7.0\*
RUN scl enable llvm-toolset-7.0 bash

# Install gcc7
RUN yum -y install devtoolset-7*
RUN scl enable devtoolset-7 bash
RUN gcc -v

# rpm -ql llvm-toolset-7.0-clang.x86_64
# /opt/rh/llvm-toolset-7.0/root/usr/lib64/libLLVM-7.so
ENV LD_LIBRARY_PATH="/opt/rh/llvm-toolset-7.0/root/usr/lib64:${LD_LIBRARY_PATH}"
ENV PATH="/opt/rh/llvm-toolset-7.0/root/usr/bin:${PATH}"

RUN ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime
RUN echo 'Asia/Shanghai' >/etc/timezone

WORKDIR /root

CMD ["bash"]
