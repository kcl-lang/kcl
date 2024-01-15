# Copyright 2021 The KCL Authors. All rights reserved.

FROM ubuntu:20.04

#RUN uname -a
#RUN cat /etc/os-release

RUN apt-get update

RUN apt-get install -y git wget curl
RUN apt-get install -y make gcc patch g++ swig
RUN apt-get install -y python-dev libffi-dev
# SSL module deps sed by python3
RUN apt-get install -y zlib1g-dev ncurses-dev build-essential libncurses5-dev libgdbm-dev libnss3-dev libssl-dev libreadline-dev libffi-dev

# python-3.9
RUN mkdir -p /root/download && cd /root/download \
  && wget http://npm.taobao.org/mirrors/python/3.9.10/Python-3.9.10.tgz \
  && tar -xzf Python-3.9.10.tgz \
  && cd Python-3.9.10 \
  && LANG=C.UTF-8 ./configure \
      --prefix=/usr/local/python3.9 \
      --enable-optimizations \
      --with-ssl \
  && make install
RUN ln -sf /usr/local/python3.9/bin/python3.9 /usr/bin/python3
RUN ln -sf /usr/local/python3.9/bin/python3.9 /usr/bin/python3.9

# rust
# https://www.rust-lang.org/tools/install
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN cargo version
RUN rustc --version

# wasm
RUN rustup target add wasm32-unknown-unknown

# clang12
RUN apt-get install -y clang-12 lld-12
RUN ln -sf /usr/bin/clang-12   /usr/bin/clang
RUN ln -sf /usr/bin/wasm-ld-12 /usr/bin/wasm-ld

# golang 1.20+
RUN mkdir -p /root/download && cd /root/download \
  && wget https://dl.google.com/go/go1.20.5.linux-amd64.tar.gz \
  && tar -zxvf go1.20.5.linux-amd64.tar.gz \
  && mv ./go /usr/local/go1.20.5
RUN ln -sf /usr/local/go1.20.5/bin/go /usr/bin/go
RUN rm -rf /root/download

ENV GOPATH=/go
ENV GOLANG_VERSION=1.20.5

RUN go install golang.org/x/lint/golint@latest
RUN go install golang.org/x/tools/cmd/goimports@latest
RUN go install honnef.co/go/tools/cmd/...@latest

RUN go install github.com/t-yuki/gocover-cobertura@latest
RUN go install github.com/jstemmer/go-junit-report@latest

RUN rm -rf /go/pkg/mod
RUN rm -rf /go/pkg/sumdb

RUN ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime
RUN echo 'Asia/Shanghai' >/etc/timezone

WORKDIR /root

CMD ["bash"]
