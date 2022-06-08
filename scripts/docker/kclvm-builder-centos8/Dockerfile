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

# Go 1.6
RUN yum -y install golang
RUN go version

# /usr/lib64/python3.9
RUN yum -y install python39-devel
RUN python3 -m pip install pytest

# golang apps
RUN go get golang.org/x/lint/golint
RUN go get golang.org/x/tools/cmd/goimports
#RUN go get honnef.co/go/tools/cmd/...

RUN go get github.com/t-yuki/gocover-cobertura
RUN go get github.com/jstemmer/go-junit-report

RUN rm -rf /go/pkg/mod
RUN rm -rf /go/pkg/sumdb

RUN ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime
RUN echo 'Asia/Shanghai' >/etc/timezone

RUN mkdir -p /root/.cargo && touch /root/.cargo/env

WORKDIR /root

CMD ["bash"]
