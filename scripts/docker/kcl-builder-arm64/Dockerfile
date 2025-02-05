# Copyright The KCL Authors. All rights reserved.
# Builder for ghcr.io/kcl-lang/kcl-builder-arm64 image

FROM arm64v8/ubuntu:20.04

#RUN uname -a
#RUN cat /etc/os-release

RUN apt-get update

RUN apt-get install -y curl make gcc git zlib1g-dev
RUN apt install -y pkg-config libssl-dev

# rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y --default-toolchain 1.83.0
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN cargo version
RUN rustc --version

# clang12
RUN apt-get install -y clang-12 lld-12
RUN ln -sf /usr/bin/clang-12   /usr/bin/clang
RUN ln -sf /usr/bin/wasm-ld-12 /usr/bin/wasm-ld

WORKDIR /root

CMD ["bash"]
