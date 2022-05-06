# Copyright 2021 The KCL Authors. All rights reserved.

FROM reg.docker.inc.com/kusionstack/kclvm-builder

# macOS M1 --platform linux/amd64
# try fix "Problem with the SSL CA cert (path? access rights?)"
# https://issueexplorer.com/issue/docker/for-mac/5912
# https://access.redhat.com/articles/2050743
RUN touch /etc/sysconfig/64bit_strstr_via_64bit_strstr_sse2_unaligned

# rust
# https://www.rust-lang.org/tools/install
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN cargo version
RUN rustc --version

# wasm
RUN rustup target add wasm32-unknown-unknown

# clang7
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

# rpm -ql llvm-toolset-7.0-clang.x86_64
# /opt/rh/llvm-toolset-7.0/root/usr/lib64/libLLVM-7.so
ENV LD_LIBRARY_PATH="/opt/rh/llvm-toolset-7.0/root/usr/lib64:${LD_LIBRARY_PATH}"
ENV PATH="/opt/rh/llvm-toolset-7.0/root/usr/bin:${PATH}"

RUN ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime
RUN echo 'Asia/Shanghai' >/etc/timezone

# local cargo mirror

COPY ./crates.io-index /root/crates.io-index

RUN  mkdir -p /root/.cargo
COPY ./_cargo_config-kclvm.toml /root/.cargo/config

WORKDIR /root

CMD ["bash"]
