# Copyright 2021 The KCL Authors. All rights reserved.

FROM centos:centos8

# macOS M1 --platform linux/amd64
# try fix "Problem with the SSL CA cert (path? access rights?)"
# https://issueexplorer.com/issue/docker/for-mac/5912
# https://access.redhat.com/articles/2050743
RUN touch /etc/sysconfig/64bit_strstr_via_64bit_strstr_sse2_unaligned

# https://forketyfork.medium.com/centos-8-no-urls-in-mirrorlist-error-3f87c3466faa
RUN sed -i -e "s|mirrorlist=|#mirrorlist=|g" /etc/yum.repos.d/CentOS-*
RUN sed -i -e "s|#baseurl=http://mirror.centos.org|baseurl=http://vault.centos.org|g" /etc/yum.repos.d/CentOS-*

RUN yum -y install make
RUN yum -y install which
RUN yum -y install wget
RUN yum -y install git

# ca-certificates
RUN yum -y install ca-certificates

# rust-1.54.0
# cargo 1.54.0
# RUN yum -y install rust cargo rustfmt
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

ENV PATH="/root/.cargo/bin:${PATH}"
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN cargo version
RUN rustc --version

# clang-12
RUN yum -y install clang
RUN clang --version

# llvm-12
RUN yum -y install llvm-devel
RUN yum -y install libffi-devel
RUN ln -s /usr/lib64/libtinfo.so.6 /usr/lib64/libtinfo.so

# golang 1.19+
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
# RUN go install honnef.co/go/tools/cmd/...@latest

RUN go install github.com/t-yuki/gocover-cobertura@latest
RUN go install github.com/jstemmer/go-junit-report@latest

RUN rm -rf /go/pkg/mod
RUN rm -rf /go/pkg/sumdb

# /usr/lib64/python3.9
RUN yum -y install python39-devel
RUN python3 -m pip install pytest

RUN ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime
RUN echo 'Asia/Shanghai' >/etc/timezone

RUN mkdir -p /root/.cargo && touch /root/.cargo/env

WORKDIR /root

CMD ["bash"]
