# Copyright 2021 The KCL Authors. All rights reserved.

FROM alpine:latest

# set timezone
RUN apk add --no-cache tzdata \
    && cp /usr/share/zoneinfo/Asia/Shanghai /etc/localtime \
    && echo 'Asia/Shanghai' >/etc/timezone

# update repositories and install required packages
RUN apk add --no-cache \
    make \
    wget \
    git \
    ca-certificates \
    clang \
    llvm \
    libffi-dev \
    go \
    python3 \
    python3-dev \
    py3-pip

# set environment variables
ENV GOPATH=/go \
    GOLANG_VERSION=1.20.5 \
    PATH="/root/.cargo/bin:${PATH}" \
    CARGO_NET_GIT_FETCH_WITH_CLI=true

# install rust and cargo
RUN wget -qO- https://sh.rustup.rs | sh -s -- -y \
    && echo 'source $HOME/.cargo/env' >> $HOME/.ashrc \
    && . $HOME/.cargo/env \
    && cargo version \
    && rustc --version

# install go tools
RUN go install golang.org/x/lint/golint@latest \
    && go install golang.org/x/tools/cmd/goimports@latest \
    && go install github.com/t-yuki/gocover-cobertura@latest \
    && go install github.com/jstemmer/go-junit-report@latest

RUN rm -rf /root/.cache/go-build \
    && rm -rf /go/pkg/mod \
    && rm -rf /go/pkg/sumdb \
    && rm -rf /var/cache/apk/*

WORKDIR /root

CMD ["ash"]
